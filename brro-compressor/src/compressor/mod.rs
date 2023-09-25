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

pub struct CompressedBlock {
    
}

impl Compressor {
    fn compress(&self, data: &[f64] ) -> Vec<f64> {
        match self {
            Compressor::Noop => data.to_vec(),
            Compressor::FFT => todo!(),
            Compressor::Constant => constant(data),
            _ => todo!(),
        }
    }
}