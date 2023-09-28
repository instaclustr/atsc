use bincode::config::{self, Configuration};
use self::constant::constant;
use self::noop::noop;

pub mod noop;
pub mod constant;

pub enum Compressor {
    Noop,
    FFT,
    Wavelet,
    Constant,
    Polynomial,
    TopBottom,
}

impl Compressor {
    fn compress(&self, data: &[f64] ) -> Vec<u8> {
        match self {
            Compressor::Noop => noop(data),
            Compressor::FFT => todo!(),
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