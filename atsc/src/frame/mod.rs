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

use crate::{
    compressor::{Compressor, CompressorResult},
    decoder::Decoder,
    error::{DecodeError, EncodeError},
    optimizer::utils::DataStats,
};
use bincode::{Decode, Encode};
use log::debug;
use std::mem::size_of_val;

const COMPRESSION_SPEED: [usize; 7] = [usize::MAX, 4096, 2048, 1024, 512, 256, 128];
const AUTO_COMPRESSORS: [Compressor; 3] =
    [Compressor::FFT, Compressor::Polynomial, Compressor::RLE];

fn meets_error_bound(result: &CompressorResult, requested: f64) -> bool {
    requested.is_finite() && result.error.is_finite() && result.error <= requested
}

fn validate_error_bound(
    codec: Compressor,
    requested: f64,
    result: CompressorResult,
) -> Result<CompressorResult, EncodeError> {
    if meets_error_bound(&result, requested) {
        Ok(result)
    } else {
        Err(EncodeError::ErrorBoundNotMet {
            codec,
            requested,
            actual: result.error,
        })
    }
}

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
        self.try_compress_bounded(data, max_error)
            .expect("bounded frame compression failed");
    }

    pub fn try_compress_bounded(
        &mut self,
        data: &[f64],
        max_error: f32,
    ) -> Result<(), EncodeError> {
        if self.compressor == Compressor::Auto {
            return Err(EncodeError::UnsupportedCompressor {
                codec: self.compressor,
                operation: "forced bounded compression",
            });
        }

        let requested = f64::from(max_error);
        let result = self
            .compressor
            .get_compress_bounded_results(data, requested);
        let result = validate_error_bound(self.compressor, requested, result)?;

        self.sample_count = data.len();
        self.data = result.compressed_data;
        Ok(())
    }

    /// This function tries to detect the best compressor for use and apply it to the data size
    pub fn compress_best(&mut self, data: &[f64], max_error: f32, compression_speed: usize) {
        self.try_compress_best(data, max_error, compression_speed)
            .expect("automatic frame compression failed");
    }

    pub fn try_compress_best(
        &mut self,
        data: &[f64],
        max_error: f32,
        compression_speed: usize,
    ) -> Result<(), EncodeError> {
        // Speed factor limits the amount of data that is sampled to calculate the best compressor.
        // We need enough samples to do decent compression, minimum is 128 (2^7)
        let data_sample = *COMPRESSION_SPEED.get(compression_speed).ok_or(
            EncodeError::InvalidCompressionSpeed {
                index: compression_speed,
                max_index: COMPRESSION_SPEED.len() - 1,
            },
        )?;
        let requested = f64::from(max_error);
        // Do a statistical analysis of the data, let's see if we can pick a compressor out of this.
        let stats = DataStats::new(data);
        // Checking the statistical analysis and chose, if possible, a compressor
        // If the data is constant, well, constant frame
        let (compressor, result) = if stats.min == stats.max {
            let compressor = Compressor::Constant;
            let result = compressor.get_compress_bounded_results(data, requested);
            (
                compressor,
                validate_error_bound(compressor, requested, result)?,
            )
        } else {
            let mut candidates: Vec<_> = AUTO_COMPRESSORS.iter().copied().enumerate().collect();

            if data_sample != usize::MAX && data.len() >= data_sample {
                let sample = &data[..data_sample];
                candidates.sort_by_key(|(stable_rank, compressor)| {
                    let result = compressor.get_compress_bounded_results(sample, requested);
                    (
                        !meets_error_bound(&result, requested),
                        result.compressed_data.len(),
                        *stable_rank,
                    )
                });
            }

            let mut best: Option<(usize, Compressor, CompressorResult)> = None;
            let mut lowest_actual = f64::INFINITY;
            for (stable_rank, compressor) in candidates {
                let result = compressor.get_compress_bounded_results(data, requested);
                if result.error.is_finite() {
                    lowest_actual = lowest_actual.min(result.error);
                }
                if !meets_error_bound(&result, requested) {
                    continue;
                }

                let candidate_key = (result.compressed_data.len(), stable_rank);
                let replace = best
                    .as_ref()
                    .map(|(best_rank, _, best_result)| {
                        candidate_key < (best_result.compressed_data.len(), *best_rank)
                    })
                    .unwrap_or(true);
                if replace {
                    best = Some((stable_rank, compressor, result));
                }
            }

            let (_, compressor, result) = best.ok_or(EncodeError::ErrorBoundNotMet {
                codec: Compressor::Auto,
                requested,
                actual: lowest_actual,
            })?;
            (compressor, result)
        };

        self.sample_count = data.len();
        self.data = result.compressed_data;
        self.compressor = compressor;
        debug!("Auto Compressor Selection: {:?}", self.compressor);
        Ok(())
    }

    /// Decompresses a frame and returns the resulting data array
    pub fn decompress(&self) -> Vec<f64> {
        self.try_decompress()
            .expect("failed to decompress BRO frame")
    }

    pub fn try_decompress(&self) -> Result<Vec<f64>, DecodeError> {
        debug!(
            "Decompressing Frame. Size: {}, Samples: {}",
            self.frame_size, self.sample_count
        );
        self.compressor
            .try_decompress(self.sample_count, &self.data)
    }

    pub(crate) fn try_decompress_into(
        &self,
        decoder: &mut Decoder,
        output: &mut Vec<f64>,
    ) -> Result<usize, DecodeError> {
        debug!(
            "Decompressing Frame. Size: {}, Samples: {}",
            self.frame_size, self.sample_count
        );
        self.compressor
            .try_decompress_into(self.sample_count, &self.data, decoder, output)
    }

    pub(crate) fn sample_count(&self) -> usize {
        self.sample_count
    }

    pub(crate) fn compressor(&self) -> Compressor {
        self.compressor
    }

    pub(crate) fn payload_bytes(&self) -> usize {
        self.data.len()
    }
}
