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

use bincode::config::{self, Configuration};
use bincode::{Decode, Encode};

use crate::{
    decoder::Decoder, error::DecodeError, optimizer::utils::DataStats, utils::is_decomposable,
};

use self::constant::{constant_compressor, Constant};
use self::fft::{fft, fft_compressor, FFT};
use self::noop::{noop, Noop};
use self::polynomial::{polynomial, polynomial_allowed_error, Polynomial, PolynomialType};
use self::rle::{rle_compressor, IndexRLE};

const BINCODE_DECODE_LIMIT: usize = 256 * 1024 * 1024;
type BincodeDecodeConfig =
    Configuration<config::LittleEndian, config::Varint, config::Limit<BINCODE_DECODE_LIMIT>>;

pub mod constant;
pub mod fft;
pub mod noop;
pub mod polynomial;
pub mod rle;

#[derive(Encode, Decode, Default, Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum Compressor {
    #[default]
    Noop,
    FFT,
    Idw,
    Constant,
    Polynomial,
    Auto,
    RLE,
}

/// Struct to store the results of a compression round. Will be used to pick the best compressor.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct CompressorResult {
    pub compressed_data: Vec<u8>,
    pub error: f64,
}

impl CompressorResult {
    pub fn new(compressed_data: Vec<u8>, error: f64) -> Self {
        CompressorResult {
            compressed_data,
            error,
        }
    }
}

impl Compressor {
    pub fn compress(&self, data: &[f64]) -> Vec<u8> {
        let stats = DataStats::new(data);
        match self {
            Compressor::Noop => noop(data),
            Compressor::FFT => fft(data),
            Compressor::Constant => constant_compressor(data, stats).compressed_data,
            Compressor::Polynomial => polynomial(data, PolynomialType::Polynomial),
            Compressor::Idw => polynomial(data, PolynomialType::Idw),
            Compressor::RLE => rle_compressor(data, stats).compressed_data,
            _ => todo!(),
        }
    }

    pub fn compress_bounded(&self, data: &[f64], max_error: f64) -> Vec<u8> {
        let stats = DataStats::new(data);
        match self {
            Compressor::Noop => noop(data),
            Compressor::FFT => fft_compressor(data, max_error, stats).compressed_data,
            Compressor::Constant => constant_compressor(data, stats).compressed_data,
            Compressor::Polynomial => {
                polynomial_allowed_error(data, max_error, PolynomialType::Polynomial)
                    .compressed_data
            }
            Compressor::Idw => {
                polynomial_allowed_error(data, max_error, PolynomialType::Idw).compressed_data
            }
            Compressor::RLE => rle_compressor(data, stats).compressed_data,
            _ => todo!(),
        }
    }

    pub fn get_compress_bounded_results(&self, data: &[f64], max_error: f64) -> CompressorResult {
        let stats = DataStats::new(data);
        match self {
            Compressor::Noop => CompressorResult::new(noop(data), 0.0),
            Compressor::FFT => fft_compressor(data, max_error, stats),
            Compressor::Constant => constant_compressor(data, stats),
            Compressor::RLE => rle_compressor(data, stats),
            Compressor::Polynomial => {
                polynomial_allowed_error(data, max_error, PolynomialType::Polynomial)
            }
            Compressor::Idw => polynomial_allowed_error(data, max_error, PolynomialType::Idw),
            _ => todo!(),
        }
    }

    pub fn decompress(&self, samples: usize, data: &[u8]) -> Vec<f64> {
        self.try_decompress(samples, data)
            .expect("failed to decompress BRO frame")
    }

    pub fn try_decompress(&self, samples: usize, data: &[u8]) -> Result<Vec<f64>, DecodeError> {
        let mut decoder = Decoder::new();
        let mut output = Vec::with_capacity(samples);
        self.try_decompress_into(samples, data, &mut decoder, &mut output)?;
        Ok(output)
    }

    pub(crate) fn try_decompress_into(
        &self,
        samples: usize,
        data: &[u8],
        decoder: &mut Decoder,
        output: &mut Vec<f64>,
    ) -> Result<usize, DecodeError> {
        let initial_len = output.len();
        match self {
            Compressor::Noop => {
                let noop = Noop::try_decompress(data)?;
                if noop.data.len() != samples {
                    return Err(DecodeError::InvalidFrame {
                        codec: *self,
                        reason: format!(
                            "decoded {} values for {samples} declared samples",
                            noop.data.len()
                        ),
                    });
                }
                noop.append_to_data(samples, decoder, output);
            }
            Compressor::FFT => {
                let fft = FFT::try_decompress(data)?;
                let reconstructed_len =
                    fft_reconstructed_len(samples).ok_or_else(|| DecodeError::InvalidFrame {
                        codec: *self,
                        reason: format!(
                            "sample count {samples} overflows reconstructed FFT length"
                        ),
                    })?;
                if reconstructed_len == 0 && fft.max_value != fft.min_value {
                    return Err(DecodeError::InvalidFrame {
                        codec: *self,
                        reason: "non-constant FFT frame has zero reconstructed length".to_string(),
                    });
                }
                if let Some(position) = fft
                    .frequencies
                    .iter()
                    .map(|frequency| frequency.position())
                    .find(|position| *position >= reconstructed_len)
                {
                    return Err(DecodeError::InvalidFrame {
                        codec: *self,
                        reason: format!(
                            "frequency position {position} is outside reconstructed length {reconstructed_len}"
                        ),
                    });
                }
                fft.append_to_data(samples, decoder, output);
            }
            Compressor::Constant => {
                Constant::try_decompress(data)?.append_to_data(samples, decoder, output);
            }
            Compressor::Polynomial | Compressor::Idw => {
                let polynomial = Polynomial::try_decompress(data)?;
                let expected_type = match self {
                    Compressor::Polynomial => PolynomialType::Polynomial,
                    Compressor::Idw => PolynomialType::Idw,
                    _ => unreachable!(),
                };
                if polynomial.id != expected_type {
                    return Err(DecodeError::InvalidFrame {
                        codec: *self,
                        reason: format!(
                            "payload type {:?} does not match frame codec",
                            polynomial.id
                        ),
                    });
                }
                validate_polynomial_point_count(*self, samples, &polynomial)?;
                polynomial.append_to_data(samples, decoder, output);
            }
            Compressor::RLE => {
                let rle = IndexRLE::try_decompress(data)?;
                let mut starts = 0_usize;
                let mut has_zero = false;
                for start in rle.rle.iter().flat_map(|(_, starts)| starts) {
                    starts = starts
                        .checked_add(1)
                        .ok_or_else(|| DecodeError::InvalidFrame {
                            codec: *self,
                            reason: "run start count overflowed usize".to_string(),
                        })?;
                    has_zero |= *start == 0;
                    if *start >= samples {
                        return Err(DecodeError::InvalidFrame {
                            codec: *self,
                            reason: format!(
                                "run start {start} is outside {samples} declared samples"
                            ),
                        });
                    }
                }
                if !has_zero {
                    return Err(DecodeError::InvalidFrame {
                        codec: *self,
                        reason: "run starts do not include zero".to_string(),
                    });
                }
                if starts > samples {
                    return Err(DecodeError::InvalidFrame {
                        codec: *self,
                        reason: format!("{starts} run starts exceed {samples} declared samples"),
                    });
                }
                rle.append_to_data(samples, decoder, output);
            }
            Compressor::Auto => {
                return Err(DecodeError::InvalidFrame {
                    codec: *self,
                    reason: "Auto is not a stored frame codec".to_string(),
                });
            }
        }
        Ok(output.len() - initial_len)
    }
}

fn fft_reconstructed_len(samples: usize) -> Option<usize> {
    if samples < 128 {
        return Some(samples);
    }

    let mut reconstructed_len = samples.checked_add(1)?;
    while !is_decomposable(reconstructed_len) {
        reconstructed_len = reconstructed_len.checked_add(1)?;
    }
    Some(reconstructed_len)
}

fn validate_polynomial_point_count(
    codec: Compressor,
    samples: usize,
    polynomial: &Polynomial,
) -> Result<(), DecodeError> {
    if polynomial.min == polynomial.max {
        return Ok(());
    }
    if samples == 0 {
        return Err(DecodeError::InvalidFrame {
            codec,
            reason: "polynomial frame has zero samples".to_string(),
        });
    }

    let step = polynomial.point_step as usize;
    let last_position = samples - 1;
    let mut expected_points = last_position
        .checked_div(step)
        .and_then(|count| count.checked_add(1))
        .ok_or_else(|| DecodeError::InvalidFrame {
            codec,
            reason: "polynomial point count overflowed usize".to_string(),
        })?;
    if last_position % step != 0 {
        expected_points =
            expected_points
                .checked_add(1)
                .ok_or_else(|| DecodeError::InvalidFrame {
                    codec,
                    reason: "polynomial point count overflowed usize".to_string(),
                })?;
    }
    if polynomial.data_points.len() != expected_points {
        return Err(DecodeError::InvalidFrame {
            codec,
            reason: format!(
                "decoded {} points but {expected_points} are required for {samples} samples",
                polynomial.data_points.len()
            ),
        });
    }
    Ok(())
}

pub(crate) fn decode_payload<T: Decode>(
    data: &[u8],
    context: &'static str,
) -> Result<T, DecodeError> {
    let (decoded, consumed) = bincode::decode_from_slice(data, BinConfig::get_decode())
        .map_err(|source| DecodeError::Bincode { context, source })?;
    if consumed != data.len() {
        return Err(DecodeError::TrailingBytes {
            remaining: data.len() - consumed,
        });
    }
    Ok(decoded)
}

pub struct BinConfig {
    config: Configuration,
}

impl BinConfig {
    pub fn get() -> Configuration {
        // Little endian and Variable int encoding
        config::standard()
    }

    pub(crate) fn get_decode() -> BincodeDecodeConfig {
        config::standard().with_limit::<BINCODE_DECODE_LIMIT>()
    }
}
