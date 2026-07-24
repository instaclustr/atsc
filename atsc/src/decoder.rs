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

use std::{fmt, iter::FusedIterator, mem, ops::Range, slice};

use crate::{
    compressor::Compressor,
    data::CompressedStream,
    error::{reserve_decode, DecodeError},
    frame::CompressorFrame,
};
use rustfft::{num_complex::Complex, FftPlanner};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameInfo {
    pub index: usize,
    pub sample_offset: usize,
    pub sample_count: usize,
    pub compressor: Compressor,
    pub payload_bytes: usize,
}

pub(crate) struct FrameInfoIter<'a> {
    frames: std::iter::Enumerate<slice::Iter<'a, CompressorFrame>>,
    sample_offset: usize,
}

impl<'a> FrameInfoIter<'a> {
    pub(crate) fn new(frames: &'a [CompressorFrame]) -> Self {
        Self {
            frames: frames.iter().enumerate(),
            sample_offset: 0,
        }
    }
}

impl Iterator for FrameInfoIter<'_> {
    type Item = FrameInfo;

    fn next(&mut self) -> Option<Self::Item> {
        let (index, frame) = self.frames.next()?;
        let info = FrameInfo {
            index,
            sample_offset: self.sample_offset,
            sample_count: frame.sample_count(),
            compressor: frame.compressor(),
            payload_bytes: frame.payload_bytes(),
        };
        self.sample_offset = self
            .sample_offset
            .checked_add(info.sample_count)
            .expect("compressed stream sample count overflowed usize");
        Some(info)
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        for _ in 0..n {
            self.next()?;
        }
        self.next()
    }

    fn last(self) -> Option<Self::Item> {
        let mut last = None;
        for info in self {
            last = Some(info);
        }
        last
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.frames.size_hint()
    }
}

impl ExactSizeIterator for FrameInfoIter<'_> {
    fn len(&self) -> usize {
        self.frames.len()
    }
}

impl FusedIterator for FrameInfoIter<'_> {}

fn append_transactionally(
    output: &mut Vec<f64>,
    append: impl FnOnce(&mut Vec<f64>) -> Result<(), DecodeError>,
) -> Result<usize, DecodeError> {
    let initial_len = output.len();
    match append(output) {
        Ok(()) => Ok(output.len() - initial_len),
        Err(error) => {
            output.truncate(initial_len);
            Err(error)
        }
    }
}

#[derive(Default)]
pub struct Decoder {
    fft_planner: Option<FftPlanner<f32>>,
    fft_buffer: Vec<Complex<f32>>,
    frame_output: Vec<f64>,
    rle_runs: Vec<(usize, f64)>,
}

impl fmt::Debug for Decoder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Decoder")
            .field("fft_planner_initialized", &self.fft_planner.is_some())
            .field("fft_buffer", &self.fft_buffer)
            .field("frame_output", &self.frame_output)
            .field("rle_runs", &self.rle_runs)
            .finish_non_exhaustive()
    }
}

impl Decoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn decode(&mut self, stream: &CompressedStream) -> Result<Vec<f64>, DecodeError> {
        let mut output = Vec::new();
        self.decode_into(stream, &mut output)?;
        Ok(output)
    }

    /// Appends the decoded stream to `output` and returns the number appended.
    ///
    /// If any frame fails to decode, `output` is restored to its original
    /// length before the error is returned.
    pub fn decode_into(
        &mut self,
        stream: &CompressedStream,
        output: &mut Vec<f64>,
    ) -> Result<usize, DecodeError> {
        append_transactionally(output, |output| {
            let samples = stream.try_sample_count()?;
            reserve_decode(output, samples, "full stream output")?;
            for frame in stream.frames() {
                frame.try_decompress_into(self, output)?;
            }
            Ok(())
        })
    }

    pub fn decode_frame_into(
        &mut self,
        stream: &CompressedStream,
        frame_index: usize,
        output: &mut Vec<f64>,
    ) -> Result<usize, DecodeError> {
        let frame = stream
            .frames()
            .get(frame_index)
            .ok_or(DecodeError::FrameOutOfBounds {
                index: frame_index,
                frames: stream.frame_count(),
            })?;
        append_transactionally(output, |output| {
            reserve_decode(output, frame.sample_count(), "single frame output")?;
            frame.try_decompress_into(self, output)?;
            Ok(())
        })
    }

    pub fn decode_range(
        &mut self,
        stream: &CompressedStream,
        range: Range<usize>,
    ) -> Result<Vec<f64>, DecodeError> {
        let mut output = Vec::new();
        self.decode_range_into(stream, range, &mut output)?;
        Ok(output)
    }

    /// Appends the requested sample range and returns the number appended.
    ///
    /// If the range is invalid or any intersecting frame fails to decode,
    /// `output` is restored to its original length before the error is
    /// returned.
    pub fn decode_range_into(
        &mut self,
        stream: &CompressedStream,
        range: Range<usize>,
        output: &mut Vec<f64>,
    ) -> Result<usize, DecodeError> {
        append_transactionally(output, |output| {
            let samples = stream.try_sample_count()?;
            if range.start > range.end || range.end > samples {
                return Err(DecodeError::RangeOutOfBounds {
                    start: range.start,
                    end: range.end,
                    samples,
                });
            }

            let requested = range.end - range.start;
            if requested == 0 {
                return Ok(());
            }

            reserve_decode(output, requested, "sample range output")?;
            let mut frame_start = 0_usize;
            for frame in stream.frames() {
                let frame_end = frame_start.checked_add(frame.sample_count()).ok_or(
                    DecodeError::AllocationFailed {
                        context: "stream sample metadata",
                        requested: usize::MAX,
                    },
                )?;
                if frame_start == frame_end || frame_end <= range.start {
                    frame_start = frame_end;
                    continue;
                }
                if frame_start >= range.end {
                    break;
                }

                let local_start = range.start.saturating_sub(frame_start);
                let local_end = range.end.min(frame_end) - frame_start;
                if local_start == 0 && local_end == frame.sample_count() {
                    frame.try_decompress_into(self, output)?;
                } else {
                    let mut frame_output = mem::take(&mut self.frame_output);
                    frame_output.clear();
                    let decode_result = frame.try_decompress_into(self, &mut frame_output);
                    self.frame_output = frame_output;
                    decode_result?;
                    output.extend_from_slice(&self.frame_output[local_start..local_end]);
                }
                frame_start = frame_end;
            }
            Ok(())
        })
    }

    pub(crate) fn fft_scratch(&mut self) -> (&mut FftPlanner<f32>, &mut Vec<Complex<f32>>) {
        (
            self.fft_planner.get_or_insert_with(FftPlanner::new),
            &mut self.fft_buffer,
        )
    }

    pub(crate) fn rle_scratch(&mut self) -> &mut Vec<(usize, f64)> {
        &mut self.rle_runs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(bytes: &[u8]) -> CompressedStream {
        CompressedStream::try_from_bytes(bytes).expect("fixture must parse")
    }

    #[test]
    fn fft_scratch_capacity_is_reused() {
        let stream = fixture(include_bytes!("../tests/fixtures/v1/fft.bro"));
        let mut decoder = Decoder::new();
        let mut output = Vec::new();
        assert!(decoder.fft_planner.is_none());

        decoder
            .decode_into(&stream, &mut output)
            .expect("first FFT decode must succeed");
        assert!(decoder.fft_planner.is_some());
        let first_capacity = decoder.fft_buffer.capacity();
        assert!(first_capacity > 0);

        output.clear();
        decoder
            .decode_into(&stream, &mut output)
            .expect("second FFT decode must succeed");
        assert_eq!(decoder.fft_buffer.capacity(), first_capacity);
    }

    #[test]
    fn rle_scratch_capacity_is_reused() {
        let stream = fixture(include_bytes!("../tests/fixtures/v1/rle.bro"));
        let mut decoder = Decoder::new();
        let mut output = Vec::new();

        decoder
            .decode_into(&stream, &mut output)
            .expect("first RLE decode must succeed");
        let first_capacity = decoder.rle_runs.capacity();
        assert!(first_capacity > 0);

        output.clear();
        decoder
            .decode_into(&stream, &mut output)
            .expect("second RLE decode must succeed");
        assert_eq!(decoder.rle_runs.capacity(), first_capacity);
    }
}
