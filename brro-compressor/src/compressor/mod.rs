pub mod noop;

pub enum Compressor {
    Noop,
    FFT,
    Wavelet,
    Static,
}

pub struct Compressor {
    
}