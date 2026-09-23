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
    error::{DecodeError, EncodeError, validate_encode_input},
    optimizer::utils::DataStats,
};
use bincode::{BorrowDecode, Decode, Encode};
use log::debug;
use std::mem::size_of_val;

const COMPRESSION_SPEED: [usize; 7] = [usize::MAX, 4096, 2048, 1024, 512, 256, 128];
const AUTO_COMPRESSORS: [Compressor; 3] =
    [Compressor::FFT, Compressor::Polynomial, Compressor::RLE];

fn meets_error_bound(result: &CompressorResult, requested: f64) -> bool {
    requested.is_finite() && result.error.is_finite() && result.error <= requested
}

/// How a codec result that misses the requested error bound is handled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoundPolicy {
    /// Reject it with [`EncodeError::ErrorBoundNotMet`].
    Strict,
    /// Commit what v0.7 committed: the forced codec's result, Auto's
    /// sample-selected codec, or the smallest Auto candidate when no candidate
    /// meets the bound.
    BestEffort,
}

fn validate_error_bound(
    codec: Compressor,
    requested: f64,
    result: CompressorResult,
    policy: BoundPolicy,
) -> Result<CompressorResult, EncodeError> {
    if policy == BoundPolicy::BestEffort || meets_error_bound(&result, requested) {
        Ok(result)
    } else {
        Err(EncodeError::ErrorBoundNotMet {
            codec,
            requested,
            actual: result.error,
        })
    }
}

/// v0.7 picked the smallest candidate that met the bound on the sample prefix
/// and committed its full-frame result without re-checking the bound; it
/// panicked when no candidate met the bound on the sample.
fn v0_7_sampled_choice(sample: &[f64], requested: f64) -> Option<Compressor> {
    AUTO_COMPRESSORS
        .iter()
        .map(|compressor| {
            (
                compressor.get_compress_bounded_results(sample, requested),
                *compressor,
            )
        })
        .filter(|(result, _)| result.error <= requested)
        .min_by_key(|(result, _)| result.compressed_data.len())
        .map(|(_, compressor)| compressor)
}

fn keep_smaller(
    slot: &mut Option<(usize, Compressor, CompressorResult)>,
    stable_rank: usize,
    compressor: Compressor,
    result: CompressorResult,
) {
    let candidate_key = (result.compressed_data.len(), stable_rank);
    let replace = slot
        .as_ref()
        .map(|(best_rank, _, best_result)| {
            candidate_key < (best_result.compressed_data.len(), *best_rank)
        })
        .unwrap_or(true);
    if replace {
        *slot = Some((stable_rank, compressor, result));
    }
}

/// This is the structure of a compressor frame
#[derive(Encode, Decode, Debug, Clone)]
pub struct CompressorFrame {
    /// Opaque BRO v1 legacy metadata.
    ///
    /// Historical writers stored a host-layout estimate here, not a serialized
    /// frame length. Decoders intentionally ignore arbitrary values for wire
    /// compatibility; an authoritative byte length requires a BRO v2 field.
    frame_size: usize,
    sample_count: usize,
    compressor: Compressor,
    /// Output from the compressor
    data: Vec<u8>,
}

/// Wire-identical view of [`CompressorFrame`] whose payload borrows from the
/// input, so a forged payload length fails before any buffer is allocated.
#[derive(BorrowDecode)]
pub(crate) struct BorrowedFrame<'a> {
    frame_size: usize,
    sample_count: usize,
    compressor: Compressor,
    data: &'a [u8],
}

impl BorrowedFrame<'_> {
    pub(crate) fn sample_count(&self) -> usize {
        self.sample_count
    }
}

impl From<BorrowedFrame<'_>> for CompressorFrame {
    fn from(frame: BorrowedFrame<'_>) -> Self {
        CompressorFrame {
            frame_size: frame.frame_size,
            sample_count: frame.sample_count,
            compressor: frame.compressor,
            data: frame.data.to_vec(),
        }
    }
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
    ///
    /// Unlike [`Self::try_compress_bounded`], a missed error bound still stores
    /// the codec's bounded output, as v0.7 did. Other errors panic.
    pub fn compress_bounded(&mut self, data: &[f64], max_error: f32) {
        self.compress_bounded_with_policy(data, max_error, BoundPolicy::BestEffort)
            .expect("bounded frame compression failed");
    }

    pub fn try_compress_bounded(
        &mut self,
        data: &[f64],
        max_error: f32,
    ) -> Result<(), EncodeError> {
        self.compress_bounded_with_policy(data, max_error, BoundPolicy::Strict)
    }

    pub(crate) fn compress_bounded_with_policy(
        &mut self,
        data: &[f64],
        max_error: f32,
        policy: BoundPolicy,
    ) -> Result<(), EncodeError> {
        validate_encode_input(data)?;
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
        let result = validate_error_bound(self.compressor, requested, result, policy)?;

        self.sample_count = data.len();
        self.data = result.compressed_data;
        Ok(())
    }

    /// This function tries to detect the best compressor for use and apply it to the data size
    ///
    /// Unlike [`Self::try_compress_best`], this selects the codec as v0.7 did:
    /// with a sampling speed, from the sample prefix without re-checking the
    /// bound on the full frame; otherwise, or if nothing met the bound on the
    /// sample, the smallest candidate is stored when none meets the bound.
    /// Other errors panic.
    pub fn compress_best(&mut self, data: &[f64], max_error: f32, compression_speed: usize) {
        self.compress_best_with_policy(data, max_error, compression_speed, BoundPolicy::BestEffort)
            .expect("automatic frame compression failed");
    }

    pub fn try_compress_best(
        &mut self,
        data: &[f64],
        max_error: f32,
        compression_speed: usize,
    ) -> Result<(), EncodeError> {
        self.compress_best_with_policy(data, max_error, compression_speed, BoundPolicy::Strict)
    }

    pub(crate) fn compress_best_with_policy(
        &mut self,
        data: &[f64],
        max_error: f32,
        compression_speed: usize,
        policy: BoundPolicy,
    ) -> Result<(), EncodeError> {
        validate_encode_input(data)?;
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
        let sampled = data_sample != usize::MAX && data.len() >= data_sample;
        // Checking the statistical analysis and chose, if possible, a compressor
        // If the data is constant, well, constant frame
        let (compressor, result) = if stats.min == stats.max {
            let compressor = Compressor::Constant;
            let result = compressor.get_compress_bounded_results(data, requested);
            (
                compressor,
                validate_error_bound(compressor, requested, result, policy)?,
            )
        } else if let Some(compressor) = (policy == BoundPolicy::BestEffort && sampled)
            .then(|| v0_7_sampled_choice(&data[..data_sample], requested))
            .flatten()
        {
            (
                compressor,
                compressor.get_compress_bounded_results(data, requested),
            )
        } else {
            let mut candidates: Vec<_> = AUTO_COMPRESSORS.iter().copied().enumerate().collect();

            if policy == BoundPolicy::Strict && sampled {
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
            let mut smallest_above_bound: Option<(usize, Compressor, CompressorResult)> = None;
            let mut lowest_actual = f64::INFINITY;
            for (stable_rank, compressor) in candidates {
                let result = compressor.get_compress_bounded_results(data, requested);
                if result.error.is_finite() {
                    lowest_actual = lowest_actual.min(result.error);
                }
                if meets_error_bound(&result, requested) {
                    keep_smaller(&mut best, stable_rank, compressor, result);
                } else if policy == BoundPolicy::BestEffort {
                    keep_smaller(&mut smallest_above_bound, stable_rank, compressor, result);
                }
            }

            let (_, compressor, result) =
                best.or(smallest_above_bound)
                    .ok_or(EncodeError::ErrorBoundNotMet {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::catch_unwind;

    fn with_zeros() -> Vec<f64> {
        (0..256).map(|index| f64::from(index % 5)).collect()
    }

    #[test]
    fn infallible_forced_bounded_compression_commits_codec_output_for_undefined_error() {
        let data = with_zeros();
        let expected = Compressor::Polynomial.get_compress_bounded_results(&data, 0.03);
        assert!(!expected.error.is_finite(), "{}", expected.error);

        let mut strict = CompressorFrame::new(Some(Compressor::Polynomial));
        assert!(matches!(
            strict.try_compress_bounded(&data, 0.03),
            Err(EncodeError::ErrorBoundNotMet { .. })
        ));
        assert!(strict.data.is_empty());

        let mut frame = CompressorFrame::new(Some(Compressor::Polynomial));
        frame.compress_bounded(&data, 0.03);
        assert_eq!(frame.compressor, Compressor::Polynomial);
        assert_eq!(frame.sample_count, data.len());
        assert_eq!(frame.data, expected.compressed_data);
    }

    #[test]
    fn infallible_forced_bounded_compression_commits_codec_output_above_bound() {
        let data = [1.0, 2.0];
        let mut frame = CompressorFrame::new(Some(Compressor::Constant));
        frame.compress_bounded(&data, 0.03);

        assert_eq!(frame.compressor, Compressor::Constant);
        assert_eq!(frame.sample_count, data.len());
        assert_eq!(frame.data, Compressor::Constant.compress(&data));
    }

    #[test]
    fn infallible_auto_compression_commits_smallest_candidate_when_none_meets_bound() {
        let data = with_zeros();
        let (expected_result, expected_codec) = AUTO_COMPRESSORS
            .iter()
            .map(|codec| (codec.get_compress_bounded_results(&data, -1.0), *codec))
            .min_by_key(|(result, _)| result.compressed_data.len())
            .unwrap();

        for speed in [0, COMPRESSION_SPEED.len() - 1] {
            let mut strict = CompressorFrame::new(Some(Compressor::Auto));
            assert!(matches!(
                strict.try_compress_best(&data, -1.0, speed),
                Err(EncodeError::ErrorBoundNotMet { .. })
            ));
            assert!(strict.data.is_empty());

            let mut frame = CompressorFrame::new(Some(Compressor::Auto));
            frame.compress_best(&data, -1.0, speed);
            assert_eq!(frame.compressor, expected_codec);
            assert_eq!(frame.sample_count, data.len());
            assert_eq!(frame.data, expected_result.compressed_data);
        }
    }

    fn smooth_prefix_then_noise() -> Vec<f64> {
        (0..512_u32)
            .map(|index| {
                if index < 128 {
                    100.0 + (f64::from(index) / 16.0).sin()
                } else {
                    100.0 + f64::from(index.wrapping_mul(2_654_435_761) % 97)
                }
            })
            .collect()
    }

    #[test]
    fn infallible_sampled_auto_reproduces_v0_7_selection() {
        let data = smooth_prefix_then_noise();
        let speed = COMPRESSION_SPEED.len() - 1;
        let sample = &data[..COMPRESSION_SPEED[speed]];
        let requested = f64::from(0.03_f32);
        let (_, sampled_codec) = AUTO_COMPRESSORS
            .iter()
            .map(|codec| {
                (
                    codec.get_compress_bounded_results(sample, requested),
                    *codec,
                )
            })
            .filter(|(result, _)| result.error <= requested)
            .min_by_key(|(result, _)| result.compressed_data.len())
            .unwrap();

        let mut strict = CompressorFrame::new(Some(Compressor::Auto));
        strict.try_compress_best(&data, 0.03, speed).unwrap();
        assert_ne!(strict.compressor, sampled_codec);

        let mut frame = CompressorFrame::new(Some(Compressor::Auto));
        frame.compress_best(&data, 0.03, speed);
        assert_eq!(frame.compressor, sampled_codec);
        assert_eq!(frame.sample_count, data.len());
        assert_eq!(
            frame.data,
            sampled_codec
                .get_compress_bounded_results(&data, requested)
                .compressed_data
        );
    }

    #[test]
    fn infallible_frame_wrappers_still_panic_on_non_bound_errors() {
        let forced_auto = catch_unwind(|| {
            CompressorFrame::new(Some(Compressor::Auto)).compress_bounded(&[1.0], 0.03);
        });
        assert!(forced_auto.is_err());

        let non_finite = catch_unwind(|| {
            CompressorFrame::new(Some(Compressor::Auto)).compress_best(&[f64::NAN], 0.03, 0);
        });
        assert!(non_finite.is_err());

        let empty = catch_unwind(|| {
            CompressorFrame::new(Some(Compressor::Noop)).compress_bounded(&[], 0.03);
        });
        assert!(empty.is_err());
    }
}
