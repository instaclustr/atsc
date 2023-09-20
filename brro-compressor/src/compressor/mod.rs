pub mod noop;

pub enum Compressor {
    Noop,
    FFT,
    Wavelet,
    Static,
}

impl Compressor {
    pub fn compress(self) {

    }

    pub fn decompress(self) {
        
    }
}