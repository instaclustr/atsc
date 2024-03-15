use crate::{compressor::Compressor, optimizer::utils::DataStats};
use bincode::{Decode, Encode};
use log::debug;
use std::mem::size_of_val;

const COMPRESSION_SPEED: [i32; 7] = [i32::MAX, 4096, 2048, 1024, 512, 256, 128];

/// This is the structure of a compressor frame
#[derive(Encode, Decode, Debug, Clone)]
pub struct CompressorFrame {
    /// The frame size in bytes,
    frame_size: usize,
    /// The number of samples in this frame,
    sample_count: usize,
    /// The compressor used in the current frame
    compressor: Compressor,
    /// Output from the compressor
    data: Vec<u8>,
}

impl CompressorFrame {
    ///  Creates a compressor frame, if a compressor is provided, it forces that compressor, otherwise is selected
    /// by the optimizer
    /// compressor: None to allow BRRO to chose, or force one
    pub fn new(provided_compressor: Option<Compressor>) -> Self {
        CompressorFrame {
            frame_size: 0,
            sample_count: 0,
            compressor: provided_compressor.unwrap_or_default(),
            data: Vec::new(),
        }
    }

    /// Calculates the size of the Frame and "closes it"
    // TODO this is probably wrong, so we have to use the write stream to dump the bytes writen
    pub fn close(&mut self) {
        let size = size_of_val(&self.sample_count)
            + size_of_val(&self.compressor)
            + size_of_val(&self.data)
            + size_of_val(&self.frame_size);
        self.frame_size = size;
    }

    /// Compress a data and stores the result in the frame
    pub fn compress(&mut self, data: &[f64]) {
        self.sample_count = data.len();
        self.data = self.compressor.compress(data);
    }

    /// Compress a data and stores the result in the frame
    pub fn compress_bounded(&mut self, data: &[f64], max_error: f32) {
        self.sample_count = data.len();
        self.data = self.compressor.compress_bounded(data, max_error as f64);
    }

    /// This function tries to detect the best compressor for use and apply it to the data size
    pub fn compress_best(&mut self, data: &[f64], max_error: f32, compression_speed: usize) {
        self.sample_count = data.len();
        // Speed factor limits the amount of data that is sampled to calculate the best compressor.
        // We need enough samples to do decent compression, minimum is 128 (2^7)
        let data_sample = COMPRESSION_SPEED[compression_speed] as usize;
        // Eligible compressors for use
        let compressor_list = [
            Compressor::Constant,
            Compressor::FFT,
            Compressor::Polynomial,
        ];
        // Do a statistical analysis of the data, let's see if we can pick a compressor out of this.
        let stats = DataStats::new(data);
        // Checking the statistical analysis and chose, if possible, a compressor
        // If the data is constant, well, constant frame
        if stats.min == stats.max {
            self.compressor = Compressor::Constant;
            // Now do the full data compression
            self.data = self
                .compressor
                .get_compress_bounded_results(data, max_error as f64)
                .compressed_data;
        }
        // Any technique determine the best compressor seems to be slower than this one
        // Sample the dataset for a fast compressor run
        // Pick the best compression
        // Compress the full dataset that way
        if self.sample_count >= data_sample {
            let (_smallest_result, chosen_compressor) = compressor_list
                .iter()
                .map(|compressor| {
                    (
                        compressor
                            .get_compress_bounded_results(&data[0..data_sample], max_error as f64),
                        compressor,
                    )
                })
                .min_by_key(|x| x.0.compressed_data.len())
                .unwrap();
            self.compressor = *chosen_compressor;
            // Now do the full data compression
            self.data = self
                .compressor
                .get_compress_bounded_results(data, max_error as f64)
                .compressed_data;
        } else {
            // Run all the eligible compressors and choose smallest
            let (smallest_result, chosen_compressor) = compressor_list
                .iter()
                .map(|compressor| {
                    (
                        compressor.get_compress_bounded_results(data, max_error as f64),
                        compressor,
                    )
                })
                .min_by_key(|x| x.0.compressed_data.len())
                .unwrap();

            self.data = smallest_result.compressed_data;
            self.compressor = *chosen_compressor;
        }
        debug!("Auto Compressor Selection: {:?}", self.compressor);
    }

    /// Decompresses a frame and returns the resulting data array
    pub fn decompress(&self) -> Vec<f64> {
        debug!(
            "Decompressing Frame. Size: {}, Samples: {}",
            self.frame_size, self.sample_count
        );
        self.compressor.decompress(self.sample_count, &self.data)
    }
}
