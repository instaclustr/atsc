/*
Copyright 2024 NetApp, Inc.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

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

    /// Compress a vsri index
    pub fn compress_vsri(&mut self, data: &[i32]) {
        self.sample_count = data.len();
        self.data = self.compressor.compress_vsri(data);
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
        let compressor_list = [Compressor::FFT, Compressor::Polynomial];
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
        } else if self.sample_count >= data_sample {
            // Any technique determine the best compressor seems to be slower than this one
            // Sample the dataset for a fast compressor run
            // Pick the best compression
            // Compress the full dataset that way
            let (_smallest_result, chosen_compressor) = compressor_list
                .iter()
                .map(|compressor| {
                    (
                        compressor
                            .get_compress_bounded_results(&data[0..data_sample], max_error as f64),
                        compressor,
                    )
                })
                .filter(|(result, _)| result.error <= max_error as f64)
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
            let compressor_results: Vec<_> = compressor_list
                .iter()
                .map(|compressor| {
                    (
                        compressor.get_compress_bounded_results(data, max_error as f64),
                        *compressor,
                    )
                })
                .collect();

            #[allow(
                clippy::neg_cmp_op_on_partial_ord,
                reason = "we need to exactly negate `result.error < max_error`, we can't apply de morgans to the expression due to NaN values"
            )]
            let best_compressor = if compressor_results
                .iter()
                .all(|(result, _)| !(result.error <= max_error as f64))
            {
                // To ensure we always have at least one result,
                // if all results are above the max error just pick the smallest.
                compressor_results
                    .into_iter()
                    .min_by_key(|x| x.0.compressed_data.len())
            } else {
                compressor_results
                    .into_iter()
                    .filter(|(result, _)| result.error <= max_error as f64)
                    .min_by_key(|x| x.0.compressed_data.len())
            };

            let (result, compressor) = best_compressor.unwrap();
            self.data = result.compressed_data;
            self.compressor = compressor;
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
