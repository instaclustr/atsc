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

use crate::optimizer::utils::DataStats;

use self::constant::{constant_compressor, constant_to_data};
use self::fft::{fft, fft_compressor, fft_to_data};
use self::noop::{noop, noop_to_data};
use self::polynomial::{polynomial, polynomial_allowed_error, to_data, PolynomialType};
use self::rle::{rle_compressor, rle_to_data};

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
            Compressor::RLE => rle_compressor(data),
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
            Compressor::RLE => rle_compressor(data).compressed_data,
            _ => todo!(),
        }
    }

    pub fn get_compress_bounded_results(&self, data: &[f64], max_error: f64) -> CompressorResult {
        let stats = DataStats::new(data);
        match self {
            Compressor::Noop => CompressorResult::new(noop(data), 0.0),
            Compressor::FFT => fft_compressor(data, max_error, stats),
            Compressor::Constant => constant_compressor(data, stats),
            Compressor::Polynomial => {
                polynomial_allowed_error(data, max_error, PolynomialType::Polynomial)
            }
            Compressor::Idw => polynomial_allowed_error(data, max_error, PolynomialType::Idw),
            _ => todo!(),
        }
    }

    pub fn decompress(&self, samples: usize, data: &[u8]) -> Vec<f64> {
        match self {
            Compressor::Noop => noop_to_data(samples, data),
            Compressor::FFT => fft_to_data(samples, data),
            Compressor::Constant => constant_to_data(samples, data),
            Compressor::Polynomial => to_data(samples, data),
            Compressor::Idw => to_data(samples, data),
            _ => todo!(),
        }
    }
}

pub struct BinConfig {
    config: Configuration,
}

impl BinConfig {
    pub fn get() -> Configuration {
        // Little endian and Variable int encoding
        config::standard()
    }
}
