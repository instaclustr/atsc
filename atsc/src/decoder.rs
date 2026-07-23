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

use std::ops::Range;

use crate::{compressor::Compressor, data::CompressedStream, error::DecodeError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameInfo {
    pub index: usize,
    pub sample_offset: usize,
    pub sample_count: usize,
    pub compressor: Compressor,
    pub payload_bytes: usize,
}

#[derive(Debug, Default)]
pub struct Decoder {
    frame_output: Vec<f64>,
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

    pub fn decode_into(
        &mut self,
        stream: &CompressedStream,
        output: &mut Vec<f64>,
    ) -> Result<usize, DecodeError> {
        output.reserve(stream.sample_count());
        let initial_len = output.len();
        for frame in stream.frames() {
            output.extend(frame.try_decompress()?);
        }
        Ok(output.len() - initial_len)
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
        let frame_output = frame.try_decompress()?;
        let appended = frame_output.len();
        output.extend(frame_output);
        Ok(appended)
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

    pub fn decode_range_into(
        &mut self,
        stream: &CompressedStream,
        range: Range<usize>,
        output: &mut Vec<f64>,
    ) -> Result<usize, DecodeError> {
        let samples = stream.sample_count();
        if range.start > range.end || range.end > samples {
            return Err(DecodeError::RangeOutOfBounds {
                start: range.start,
                end: range.end,
                samples,
            });
        }

        let requested = range.end - range.start;
        if requested == 0 {
            return Ok(0);
        }

        output.reserve(requested);
        let initial_len = output.len();
        let mut frame_start = 0_usize;
        for frame in stream.frames() {
            let frame_end = frame_start
                .checked_add(frame.sample_count())
                .expect("compressed stream sample count overflowed usize");
            if frame_start == frame_end || frame_end <= range.start {
                frame_start = frame_end;
                continue;
            }
            if frame_start >= range.end {
                break;
            }

            self.frame_output = frame.try_decompress()?;
            let local_start = range.start.saturating_sub(frame_start);
            let local_end = range.end.min(frame_end) - frame_start;
            output.extend_from_slice(&self.frame_output[local_start..local_end]);
            frame_start = frame_end;
        }
        Ok(output.len() - initial_len)
    }
}
