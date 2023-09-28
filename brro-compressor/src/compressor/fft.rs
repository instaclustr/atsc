use bincode::{Decode, Encode};
use rustfft::{FftPlanner, num_complex::Complex};
use super::BinConfig;
use log::{error, debug, warn};

const CONSTANT_COMPRESSOR_ID: u8 = 15;

/// FFT Compressor. Applies FFT to a signal, picks the N best frequencies, discards the rest. Always LOSSY
#[derive(Encode, Decode, PartialEq, Debug)]
pub struct FFT {
    pub id: u8,
    pub frequencies: Vec<(i32, i64)>,
    pub residuals: Vec<(i32, i64)>,
}

impl FFT {
    /// Creates a new instance of the Constant compressor with the size needed to handle the worst case
    pub fn new(frame_size: usize) -> Self {
        debug!("FFT compressor");
        FFT {
            id: CONSTANT_COMPRESSOR_ID,
            frequencies: Vec::with_capacity(frame_size),
            residuals: Vec::with_capacity(frame_size),
        }
    }

    // TODO: Move this to the optimizer?
    fn f64_to_f32(x: f64) -> f32 {
        let y = x as f32;
        if !(x.is_finite() && y.is_finite()) {
            // PANIC? Error?
            error!("f32 overflow during conversion");
        }
        y
    }

    // TODO: This actually seems to makes sense here. Call it convert?
    fn optimize(data: &[f64]) -> Vec<Complex<f32>> {
        data.iter()
            .map(|x| Complex{re: FFT::f64_to_f32(*x), im: 0.0f32})
            .collect()
    }

    /// Compress data via FFT. 
    /// This picks a set of data, computes the FFT, and stores the most relevant frequencies, dropping
    /// the remaining ones.
    pub fn compress(&mut self, data: &[f64]) {
        // First thing, always try to get the data len as a power of 2. 
        let v = data.len();
        if !v.is_power_of_two() {
            warn!("Slow FFT, data segment is not a power of 2!");
        }
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(v);
        let mut buffer = FFT::optimize(data);
        fft.process(&mut buffer);
    }

    pub fn to_bytes(self) -> Vec<u8> {
        let config = BinConfig::get();
        bincode::encode_to_vec(self, config).unwrap()
    }
}