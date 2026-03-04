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
use std::cmp::Ordering;
use std::mem::size_of_val;

const COMPRESSION_SPEED: [i32; 7] = [i32::MAX, 4096, 2048, 1024, 512, 256, 128];

/// This is the structure of a compressor frame
#[derive(Encode, Decode, Debug, Clone)]
pub struct CompressorFrame {
    /// The frame size in bytes,
    frame_size: usize,
    sample_count: usize,
    compressor: Compressor,
    /// Output from the compressor
    data: Vec<u8>,
}

impl CompressorFrame {
    fn compressor_tiebreak_rank(compressor: Compressor) -> u8 {
        match compressor {
            // Favor FFT when payload and error tie.
            Compressor::FFT => 0,
            Compressor::Polynomial => 1,
            _ => 2,
        }
    }

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
    // TODO this is probably wrong, so we have to use the write stream to dump the bytes written
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
        let compressor_list = [Compressor::FFT, Compressor::Polynomial, Compressor::RLE];
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
        } else {
            // For non-constant data, use sampled bound checks only as a shortlist and
            // always make final decisions on full payload size.
            let mut shortlist: Vec<Compressor> = compressor_list.to_vec();
            if self.sample_count >= data_sample {
                let sample_bound_hits: Vec<Compressor> = compressor_list
                    .iter()
                    .filter_map(|compressor| {
                        let sample_result = compressor
                            .get_compress_bounded_results(&data[0..data_sample], max_error as f64);
                        if sample_result.error <= max_error as f64 {
                            Some(*compressor)
                        } else {
                            None
                        }
                    })
                    .collect();

                if !sample_bound_hits.is_empty() {
                    shortlist = sample_bound_hits;
                }
            }

            // Run shortlisted compressors on full data and pick payload-first winner.
            let compressor_results: Vec<_> = shortlist
                .iter()
                .map(|compressor| {
                    (
                        compressor.get_compress_bounded_results(data, max_error as f64),
                        *compressor,
                    )
                })
                .collect();

            let mut eligible_results: Vec<_> = if compressor_results
                .iter()
                .any(|(result, _)| result.error <= max_error as f64)
            {
                compressor_results
                    .into_iter()
                    .filter(|(result, _)| result.error <= max_error as f64)
                    .collect()
            } else {
                // Ensure we always have at least one candidate.
                compressor_results
            };

            let (result, compressor) = eligible_results
                .drain(..)
                .min_by(|a, b| {
                    let by_payload = a.0.compressed_data.len().cmp(&b.0.compressed_data.len());
                    if by_payload != Ordering::Equal {
                        return by_payload;
                    }

                    let by_error = a.0.error.partial_cmp(&b.0.error).unwrap_or(Ordering::Equal);
                    if by_error != Ordering::Equal {
                        return by_error;
                    }

                    Self::compressor_tiebreak_rank(a.1).cmp(&Self::compressor_tiebreak_rank(b.1))
                })
                .unwrap();

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
