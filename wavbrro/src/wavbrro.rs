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

use rkyv::{Archive, Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::{error, fmt, io, result};

#[cfg(test)]
use crate::read::is_wavbrro_file;
use crate::write::write_wavbrro_file;

const MAX_CHUNK_SIZE: usize = 2048;
const FILE_HEADER_LEN: usize = 12;
const DEFAULT_MAX_INPUT_BYTES: usize = 256 * 1024 * 1024;
const DEFAULT_MAX_SAMPLES: usize = 131_072 * u8::MAX as usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadLimits {
    pub max_input_bytes: usize,
    pub max_samples: usize,
}

impl Default for ReadLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: DEFAULT_MAX_INPUT_BYTES,
            max_samples: DEFAULT_MAX_SAMPLES,
        }
    }
}

#[derive(Archive, Deserialize, Serialize, Debug, PartialEq, Clone)]
#[archive(
    // This will generate a PartialEq impl between our unarchived and archived
    // types:
    compare(PartialEq),
    // bytecheck can be used to validate your data if you want. To use the safe
    // API, you have to derive CheckBytes for the archived type:
    check_bytes,
)]
// Derives can be passed through to the generated type:
#[archive_attr(derive(Debug))]
pub struct WavBrro {
    // We can infer chunk count from here -> chunk count = ceil(sample_count/MAX_CHUNK_SIZE)
    pub sample_count: u32,
    // Bitdepth indicates the type of samples that is contained in the file
    // 0 -> u8, 1 -> i16, 2 -> i32, 3 -> i64, 4 -> f32, 5 -> f64
    // At version 0.1 only f64 is allowed, so any data will be converted to f64 and all data be output as f64
    pub bitdepth: u8,
    // Fixed size, of 2048 per chunk (Except last segment)
    pub chunks: Vec<Vec<f64>>,
}

impl Default for WavBrro {
    fn default() -> Self {
        Self::new()
    }
}

impl WavBrro {
    pub fn new() -> WavBrro {
        WavBrro {
            // It will duplicate with the header, but this allows to double check if the header matches.
            sample_count: 0,
            // Default to f64
            bitdepth: 5,
            chunks: Vec::new(),
        }
    }

    fn is_chunk_full(&self) -> bool {
        match self.chunks.last() {
            Some(c) => c.len() >= MAX_CHUNK_SIZE,
            None => true,
        }
    }

    fn create_chunk(&mut self) {
        // If I'm creating a chunk, I should probably flush the file?
        self.chunks.push(Vec::with_capacity(MAX_CHUNK_SIZE));
    }

    // Receives a slice of f64 and writes in it's internal structure
    fn from_slice(data: &[f64]) -> Self {
        let sample_count = data.len();
        WavBrro {
            sample_count: sample_count as u32,
            bitdepth: 5,
            chunks: data.chunks(MAX_CHUNK_SIZE).map(|s| s.into()).collect(),
        }
    }

    pub fn add_sample(&mut self, sample: f64) {
        if self.is_chunk_full() {
            self.create_chunk()
        }
        self.chunks.last_mut().unwrap().push(sample);
        self.sample_count += 1;
    }

    // TODO: This should be generic, but first implementation is going to be Vec f64
    // This consumes self!
    pub fn get_samples(self) -> Vec<f64> {
        self.chunks.into_iter().flatten().collect::<Vec<f64>>()
    }

    pub fn from_file(file_path: &Path) -> Result<Vec<f64>, Error> {
        Self::from_file_with_limits(file_path, ReadLimits::default())
    }

    pub fn from_file_with_limits(file_path: &Path, limits: ReadLimits) -> Result<Vec<f64>, Error> {
        let mut file = File::open(file_path)?;
        let metadata_len = usize::try_from(file.metadata()?.len()).unwrap_or(usize::MAX);
        if metadata_len > limits.max_input_bytes {
            return Err(Error::InputLimitExceeded {
                actual: metadata_len,
                limit: limits.max_input_bytes,
            });
        }
        if metadata_len < FILE_HEADER_LEN {
            return Err(Error::FormatError);
        }

        let mut header = [0_u8; FILE_HEADER_LEN];
        file.read_exact(&mut header)?;
        if &header[..4] != b"WBRO" || &header[8..] != b"WBRO" {
            return Err(Error::FormatError);
        }

        let body_capacity = metadata_len - FILE_HEADER_LEN;
        let mut body = rkyv::AlignedVec::with_capacity(body_capacity);
        let max_body_bytes = limits.max_input_bytes - FILE_HEADER_LEN;
        let mut buffer = [0_u8; 16 * 1024];
        loop {
            let remaining = max_body_bytes.saturating_sub(body.len());
            let read_limit = remaining.saturating_add(1).min(buffer.len());
            let read = file.read(&mut buffer[..read_limit])?;
            if read == 0 {
                break;
            }
            if read > remaining {
                return Err(Error::InputLimitExceeded {
                    actual: limits.max_input_bytes.saturating_add(1),
                    limit: limits.max_input_bytes,
                });
            }
            body.reserve_exact(read);
            body.extend_from_slice(&buffer[..read]);
        }

        let archived = validate_archived(&body, limits)?;
        archived_samples(archived)
    }

    // TODO: This will panic left and right, make it right
    pub fn to_file_with_data(file_path: &Path, data: &[f64]) {
        let wb = WavBrro::from_slice(data);
        let bytes = wb.to_bytes();
        write_wavbrro_file(file_path, &bytes);
    }

    // TODO: This will panic left and right, make it right
    pub fn to_file(&self, file_path: &Path) {
        let bytes = self.to_bytes();
        write_wavbrro_file(file_path, &bytes);
    }

    pub fn to_bytes(&self) -> rkyv::AlignedVec {
        rkyv::to_bytes::<_, 1024>(self).expect("Failed to serialize data!")
    }

    /// Validates and deserializes an archived WAVBRRO body.
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        Self::try_from_bytes_with_limits(bytes, ReadLimits::default())
    }

    pub fn try_from_bytes_with_limits(bytes: &[u8], limits: ReadLimits) -> Result<Self, Error> {
        if bytes.len() > limits.max_input_bytes {
            return Err(Error::InputLimitExceeded {
                actual: bytes.len(),
                limit: limits.max_input_bytes,
            });
        }
        let archived = validate_archived(bytes, limits)?;
        archived_to_owned(archived)
    }

    /// Compatibility wrapper that panics when the archived body is invalid.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self::try_from_bytes(bytes).expect("Failed to deserialize data!")
    }
}

fn validate_archived(bytes: &[u8], limits: ReadLimits) -> Result<&ArchivedWavBrro, Error> {
    let archived = rkyv::check_archived_root::<WavBrro>(bytes)
        .map_err(|error| Error::DeserializationError(error.to_string()))?;
    let sample_count = archived.sample_count as usize;
    if sample_count > limits.max_samples {
        return Err(Error::SampleLimitExceeded {
            actual: sample_count,
            limit: limits.max_samples,
        });
    }
    if archived.bitdepth != 5 {
        return Err(Error::InvalidMetadata(
            "bitdepth must be 5 (f64)".to_string(),
        ));
    }

    let chunks = &archived.chunks;
    if sample_count == 0 {
        if !chunks.is_empty() {
            return Err(Error::InvalidMetadata(
                "zero samples must have zero chunks".to_string(),
            ));
        }
        return Ok(archived);
    }
    let expected_chunks = ((sample_count - 1) / MAX_CHUNK_SIZE) + 1;
    if chunks.len() != expected_chunks {
        return Err(Error::InvalidMetadata(format!(
            "{sample_count} samples require {expected_chunks} chunks, found {}",
            chunks.len()
        )));
    }

    let mut total = 0_usize;
    for (index, chunk) in chunks.iter().enumerate() {
        let is_final = index + 1 == chunks.len();
        if !is_final && chunk.len() != MAX_CHUNK_SIZE {
            return Err(Error::InvalidMetadata(format!(
                "non-final chunk {index} contains {} samples instead of {MAX_CHUNK_SIZE}",
                chunk.len()
            )));
        }
        if is_final && !(1..=MAX_CHUNK_SIZE).contains(&chunk.len()) {
            return Err(Error::InvalidMetadata(format!(
                "final chunk contains {} samples",
                chunk.len()
            )));
        }
        total = total
            .checked_add(chunk.len())
            .ok_or_else(|| Error::InvalidMetadata("chunk sample sum overflowed".to_string()))?;
    }
    if total != sample_count {
        return Err(Error::InvalidMetadata(format!(
            "chunk lengths sum to {total}, declared sample count is {sample_count}"
        )));
    }
    Ok(archived)
}

fn archived_samples(archived: &ArchivedWavBrro) -> Result<Vec<f64>, Error> {
    let sample_count = archived.sample_count as usize;
    let mut samples = Vec::new();
    reserve(&mut samples, sample_count, "WAVBRRO samples")?;
    for chunk in archived.chunks.iter() {
        samples.extend(chunk.iter().copied());
    }
    Ok(samples)
}

fn archived_to_owned(archived: &ArchivedWavBrro) -> Result<WavBrro, Error> {
    let mut chunks = Vec::new();
    reserve(&mut chunks, archived.chunks.len(), "WAVBRRO chunks")?;
    for archived_chunk in archived.chunks.iter() {
        let mut chunk = Vec::new();
        reserve(&mut chunk, archived_chunk.len(), "WAVBRRO chunk samples")?;
        chunk.extend(archived_chunk.iter().copied());
        chunks.push(chunk);
    }
    Ok(WavBrro {
        sample_count: archived.sample_count,
        bitdepth: archived.bitdepth,
        chunks,
    })
}

fn reserve<T>(values: &mut Vec<T>, additional: usize, context: &'static str) -> Result<(), Error> {
    let requested = values
        .len()
        .checked_add(additional)
        .ok_or(Error::AllocationError {
            context,
            requested: usize::MAX,
        })?;
    values
        .try_reserve(additional)
        .map_err(|_| Error::AllocationError { context, requested })
}

// Error class is based on https://codeberg.org/ruuda/hound/src/branch/master given the similarities
// between the formats (WAV and WAVBRRO).
#[derive(Debug)]
pub enum Error {
    /// An IO error occured in the underlying reader or writer.
    IoError(io::Error),
    /// It's not WAVBRRO
    FormatError,
    /// The archived WAVBRRO body failed validation or deserialization.
    DeserializationError(String),
    /// The archived metadata is structurally valid but semantically invalid.
    InvalidMetadata(String),
    /// The encoded input exceeds the configured byte limit.
    InputLimitExceeded { actual: usize, limit: usize },
    /// The encoded input declares too many samples.
    SampleLimitExceeded { actual: usize, limit: usize },
    /// A checked output allocation could not be satisfied.
    AllocationError {
        context: &'static str,
        requested: usize,
    },
    /// The sample has more bits than the destination type.
    ///
    /// When iterating using the `samples` iterator, this means that the
    /// destination type (produced by the iterator) is not wide enough to hold
    /// the sample. When writing, this means that the sample cannot be written,
    /// because it requires more bits than the bits per sample specified.
    TooWide,
    /// The Sample format is not supported.
    Unsupported,
    /// The sample format is different than the destination format.
    ///
    /// When iterating using the `samples` iterator, this means the destination
    /// type (produced by the iterator) has a different sample format than the
    /// samples in the wav file.
    ///
    /// For example, this will occur if the user attempts to produce `i32`
    /// samples (which have a `SampleFormat::Int`) from a wav file that
    /// contains floating point data (`SampleFormat::Float`).
    InvalidSampleFormat,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        match *self {
            Error::IoError(ref err) => err.fmt(formatter),
            Error::FormatError => formatter.write_str("Wrong WAVBRRO file!"),
            Error::DeserializationError(ref error) => {
                write!(formatter, "Failed to deserialize WAVBRRO data: {error}")
            }
            Error::InvalidMetadata(ref error) => {
                write!(formatter, "Invalid WAVBRRO metadata: {error}")
            }
            Error::InputLimitExceeded { actual, limit } => {
                write!(
                    formatter,
                    "WAVBRRO input is {actual} bytes, above configured limit {limit}"
                )
            }
            Error::SampleLimitExceeded { actual, limit } => {
                write!(
                    formatter,
                    "WAVBRRO contains {actual} samples, above configured limit {limit}"
                )
            }
            Error::AllocationError { context, requested } => {
                write!(
                    formatter,
                    "Could not allocate capacity for {requested} items while reading {context}"
                )
            }
            Error::TooWide => {
                formatter.write_str("The sample has more bits than the destination type.")
            }
            Error::Unsupported => {
                formatter.write_str("The WAVBRRO format of the file is not supported.")
            }
            Error::InvalidSampleFormat => {
                formatter.write_str("The sample format differs from the destination format.")
            }
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IoError(err)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            // TODO: I don't know if this is actually the right way to do!
            Error::IoError(ref _err) => "IO Error",
            Error::DeserializationError(_) => "failed to deserialize WAVBRRO data",
            Error::InvalidMetadata(_) => "invalid WAVBRRO metadata",
            Error::InputLimitExceeded { .. } => "WAVBRRO input limit exceeded",
            Error::SampleLimitExceeded { .. } => "WAVBRRO sample limit exceeded",
            Error::AllocationError { .. } => "WAVBRRO allocation failed",
            Error::TooWide => "the sample has more bits than the destination type",
            Error::Unsupported => "the wave format of the file is not supported",
            Error::InvalidSampleFormat => "the sample format differs from the destination format",
            Error::FormatError => "the file is not of the WAVBRRO format",
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            Error::IoError(ref err) => Some(err),
            Error::DeserializationError(_) => None,
            Error::InvalidMetadata(_) => None,
            Error::InputLimitExceeded { .. } => None,
            Error::SampleLimitExceeded { .. } => None,
            Error::AllocationError { .. } => None,
            Error::TooWide => None,
            Error::Unsupported => None,
            Error::InvalidSampleFormat => None,
            Error::FormatError => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_invalid_metadata(wavbrro: WavBrro) {
        let bytes = wavbrro.to_bytes();
        assert!(matches!(
            WavBrro::try_from_bytes(&bytes),
            Err(Error::InvalidMetadata(_))
        ));
    }

    #[test]
    fn test_wavbrro() {
        let mut wb = WavBrro::new();
        wb.add_sample(1.0);
        assert_eq!(wb.sample_count, 1);
    }

    #[test]
    fn test_serialization() {
        let mut wb = WavBrro::new();
        wb.add_sample(1.0);
        assert_eq!(
            wb.to_bytes().as_slice(),
            &[
                0, 0, 0, 0, 0, 0, 240, 63, 248, 255, 255, 255, 1, 0, 0, 0, 248, 255, 255, 255, 1,
                0, 0, 0, 1, 0, 0, 0, 5, 0, 0, 0
            ]
        );
    }

    #[test]
    fn test_deserialization() {
        let mut wb = WavBrro::new();
        wb.add_sample(1.0);
        wb.add_sample(2.0);
        wb.add_sample(3.0);
        let bytes = wb.to_bytes();
        let wb2 = WavBrro::from_bytes(bytes.as_slice());
        assert_eq!(wb, wb2);
    }

    #[test]
    fn try_from_bytes_rejects_truncated_archive() {
        assert!(matches!(
            WavBrro::try_from_bytes(&[0]),
            Err(Error::DeserializationError(_))
        ));
    }

    #[test]
    fn archived_metadata_invariants_are_validated_before_deserialization() {
        assert_invalid_metadata(WavBrro {
            sample_count: 1,
            bitdepth: 4,
            chunks: vec![vec![1.0]],
        });
        assert_invalid_metadata(WavBrro {
            sample_count: 2,
            bitdepth: 5,
            chunks: vec![vec![1.0]],
        });
        assert_invalid_metadata(WavBrro {
            sample_count: 0,
            bitdepth: 5,
            chunks: vec![Vec::new()],
        });
        assert_invalid_metadata(WavBrro {
            sample_count: 1,
            bitdepth: 5,
            chunks: Vec::new(),
        });
        assert_invalid_metadata(WavBrro {
            sample_count: (MAX_CHUNK_SIZE + 1) as u32,
            bitdepth: 5,
            chunks: vec![vec![1.0; MAX_CHUNK_SIZE - 1], vec![2.0; 2]],
        });
        assert_invalid_metadata(WavBrro {
            sample_count: (MAX_CHUNK_SIZE + 1) as u32,
            bitdepth: 5,
            chunks: vec![vec![1.0; MAX_CHUNK_SIZE], Vec::new()],
        });
        assert_invalid_metadata(WavBrro {
            sample_count: MAX_CHUNK_SIZE as u32,
            bitdepth: 5,
            chunks: vec![vec![1.0; MAX_CHUNK_SIZE + 1]],
        });
    }

    #[test]
    fn byte_and_sample_limits_accept_exact_values_and_reject_n_plus_one() {
        let wavbrro = WavBrro::from_slice(&[1.0, 2.0]);
        let bytes = wavbrro.to_bytes();

        let exact = ReadLimits {
            max_input_bytes: bytes.len(),
            max_samples: 2,
        };
        assert_eq!(
            WavBrro::try_from_bytes_with_limits(&bytes, exact)
                .unwrap()
                .get_samples(),
            [1.0, 2.0]
        );

        assert!(matches!(
            WavBrro::try_from_bytes_with_limits(
                &bytes,
                ReadLimits {
                    max_input_bytes: bytes.len() - 1,
                    max_samples: 2,
                },
            ),
            Err(Error::InputLimitExceeded {
                actual,
                limit
            }) if actual == bytes.len() && limit == bytes.len() - 1
        ));
        assert!(matches!(
            WavBrro::try_from_bytes_with_limits(
                &bytes,
                ReadLimits {
                    max_input_bytes: bytes.len(),
                    max_samples: 1,
                },
            ),
            Err(Error::SampleLimitExceeded {
                actual: 2,
                limit: 1
            })
        ));
    }

    #[test]
    fn bounded_file_reader_accepts_exact_limit_and_ignores_legacy_size_text() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("legacy-size.wbro");
        let body = WavBrro::from_slice(&[1.0, 2.0]).to_bytes();
        let mut file = b"WBROABCDWBRO".to_vec();
        file.extend_from_slice(&body);
        std::fs::write(&path, &file).unwrap();

        assert_eq!(
            WavBrro::from_file_with_limits(
                &path,
                ReadLimits {
                    max_input_bytes: file.len(),
                    max_samples: 2,
                },
            )
            .unwrap(),
            [1.0, 2.0]
        );
        assert!(matches!(
            WavBrro::from_file_with_limits(
                &path,
                ReadLimits {
                    max_input_bytes: file.len() - 1,
                    max_samples: 2,
                },
            ),
            Err(Error::InputLimitExceeded { .. })
        ));
    }

    #[test]
    fn test_write_wavbrro() {
        // Create a temporary directory for the test
        let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
        let path = temp_dir.path().join("test.wbro");
        let mut wb = WavBrro::new();
        wb.add_sample(1.0);
        wb.add_sample(2.0);
        wb.add_sample(3.0);
        wb.to_file(&path);
        let result = is_wavbrro_file(&path);
        assert!(result.unwrap());
    }

    #[test]
    fn test_read_wavbrro() {
        let path = Path::new("test.wbro");
        let mut wb = WavBrro::new();
        wb.add_sample(1.0);
        wb.add_sample(2.0);
        wb.add_sample(3.0);
        wb.to_file(path);
        let data = WavBrro::from_file(path);
        assert_eq!(data.unwrap(), [1.0, 2.0, 3.0]);
        std::fs::remove_file(path).expect("Failed to remove temporary file");
    }
}
