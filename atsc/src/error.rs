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

use crate::compressor::Compressor;

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum EncodeError {
    #[error("compression input must not be empty")]
    EmptyInput,
    #[error("compression input at index {index} is not finite")]
    NonFiniteInput { index: usize },
    #[error("BRO v1 supports at most {limit} frames")]
    FrameLimitExceeded { limit: usize },
    #[error(
        "{codec:?} compression did not meet error bound {requested}: actual error was {actual}"
    )]
    ErrorBoundNotMet {
        codec: Compressor,
        requested: f64,
        actual: f64,
    },
    #[error("compression speed index {index} is invalid; maximum index is {max_index}")]
    InvalidCompressionSpeed { index: usize, max_index: usize },
    #[error("{codec:?} cannot be used for {operation}")]
    UnsupportedCompressor {
        codec: Compressor,
        operation: &'static str,
    },
}

pub(crate) fn validate_encode_input(data: &[f64]) -> Result<(), EncodeError> {
    if data.is_empty() {
        return Err(EncodeError::EmptyInput);
    }
    if let Some((index, _)) = data
        .iter()
        .enumerate()
        .find(|(_, value)| !value.is_finite())
    {
        return Err(EncodeError::NonFiniteInput { index });
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("BRO header requires 9 bytes, got {actual}")]
    TruncatedHeader { actual: usize },
    #[error("invalid BRO magic bytes: {found:?}")]
    InvalidMagic { found: [u8; 4] },
    #[error("BRO version {found} is newer than supported version {supported}")]
    UnsupportedVersion { found: u32, supported: u32 },
    #[error("input is {actual} bytes, above configured limit {limit}")]
    InputLimitExceeded { actual: usize, limit: usize },
    #[error("stream contains {actual} frames, above configured limit {limit}")]
    FrameLimitExceeded { actual: usize, limit: usize },
    #[error("stream contains {actual} samples, above configured limit {limit}")]
    SampleLimitExceeded { actual: usize, limit: usize },
    #[error("could not allocate capacity for {requested} items while decoding {context}")]
    AllocationFailed {
        context: &'static str,
        requested: usize,
    },
    #[error("header declares {header} frames but body contains {body}")]
    FrameCountMismatch { header: usize, body: usize },
    #[error("trailing bytes after BRO body: {remaining}")]
    TrailingBytes { remaining: usize },
    #[error("failed to decode {context}: {source}")]
    Bincode {
        context: &'static str,
        #[source]
        source: bincode::error::DecodeError,
    },
    #[error("invalid {codec:?} frame: {reason}")]
    InvalidFrame { codec: Compressor, reason: String },
    #[error("frame index {index} is outside stream frame count {frames}")]
    FrameOutOfBounds { index: usize, frames: usize },
    #[error("sample range {start}..{end} is outside stream length {samples}")]
    RangeOutOfBounds {
        start: usize,
        end: usize,
        samples: usize,
    },
}

pub(crate) fn reserve_decode<T>(
    values: &mut Vec<T>,
    additional: usize,
    context: &'static str,
) -> Result<usize, DecodeError> {
    let requested = values
        .len()
        .checked_add(additional)
        .ok_or(DecodeError::AllocationFailed {
            context,
            requested: usize::MAX,
        })?;
    values
        .try_reserve(additional)
        .map_err(|_| DecodeError::AllocationFailed { context, requested })?;
    Ok(requested)
}

#[derive(Debug, Clone, Copy)]
pub struct DecodeLimits {
    pub max_input_bytes: usize,
    pub max_frames: usize,
    pub max_samples: usize,
}

impl Default for DecodeLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: 256 * 1024 * 1024,
            max_frames: u8::MAX as usize,
            max_samples: 131_072 * u8::MAX as usize,
        }
    }
}
