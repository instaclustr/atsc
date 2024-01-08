use crate::compressor::Compressor;
use bincode::{Decode, Encode};
use log::debug;
use std::mem::size_of_val;

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

    /// Run all compressor algorithms and pick the best one, mostly for auto
    pub fn compress_best(&mut self, data: &[f64], max_error: f32) {
        self.sample_count = data.len();
        // Eligible compressors for use
        let compressor_list = [
            Compressor::Constant,
            Compressor::FFT,
            Compressor::Polynomial,
        ];

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
