pub mod noop;

pub enum Compressor {
    Noop,
    FFT,
    Wavelet,
    Static,
    Polynomial,
    TopBottom,
}

pub struct CompressedBlock {
    
}