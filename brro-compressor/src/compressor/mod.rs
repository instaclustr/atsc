use bincode::config::{self, Configuration};
use bincode::{Decode, Encode};

use self::constant::{constant, constant_to_data};
use self::fft::{fft, fft_to_data, fft_allowed_error};
use self::noop::{noop, noop_to_data};

pub mod noop;
pub mod constant;
pub mod fft;

#[derive(Encode, Decode, Default, Debug, Clone)]
pub enum Compressor {
    #[default]
    Noop,
    FFT,
    Wavelet,
    Constant,
    Polynomial,
    TopBottom,
}

impl Compressor {
    pub fn compress(&self, data: &[f64] ) -> Vec<u8> {
        match self {
            Compressor::Noop => noop(data),
            Compressor::FFT => fft(data),
            Compressor::Constant => constant(data),
            _ => todo!(),
        }
    }

    pub fn compress_bounded(&self, data: &[f64], max_error: f64 ) -> Vec<u8> {
        match self {
            Compressor::Noop => noop(data),
            Compressor::FFT => fft_allowed_error(data, max_error),
            Compressor::Constant => constant(data),
            _ => todo!(),
        }
    }

    pub fn decompress(&self, samples: usize, data: &[u8] ) -> Vec<f64> {
        match self {
            Compressor::Noop => noop_to_data(samples, data),
            Compressor::FFT => fft_to_data(samples, data),
            Compressor::Constant => constant_to_data(samples, data),
            _ => todo!()
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