pub mod noop;

pub enum Compressor {
    Noop,
    FFT,
    Wavelet,
    Static,
    Polynomial,
}

pub struct CompressedBlock {
    
}