use bincode::config::{self, Configuration};
use self::constant::constant;
use self::fft::fft;
use self::noop::noop;

pub mod noop;
pub mod constant;
pub mod fft;

#[derive(Default)]
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
            Compressor::FFT => fft(data, 8, 0.0, 10.0), // TODO: Remove the placeholders
            Compressor::Constant => constant(data),
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