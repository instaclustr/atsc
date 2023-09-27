use bincode::config::{self, Configuration};
use crate::compressor::constant::{constant};

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
            Compressor::Noop => todo!(),
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
        config::standard()
    }
}