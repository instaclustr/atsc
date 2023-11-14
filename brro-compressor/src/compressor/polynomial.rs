use super::BinConfig;
use bincode::{Decode, Encode};
use log::{debug, info};
use splines::{Interpolation, Key, Spline};

const POLYNOMIAL_COMPRESSOR_ID: u8 = 2;

#[derive(Encode, Decode, PartialEq, Debug, Clone)]
pub struct Polynomial {
    /// Compressor ID
    pub id: u8,
    /// Stored frequencies
    pub spline_coef: Vec<(f64, f64, f64, f64)>,
    /// The maximum numeric value of the points in the frame
    pub max_value: f32,  
    /// The minimum numeric value of the points in the frame
    pub min_value: f32,  
    /// To reduce error
    pub residuals: Vec<(i32, i64)>,
}

impl Polynomial {
    pub fn new(sample_count: usize, min: f64, max: f64) -> Self {
        debug!("Polynomial compressor: min:{} max:{}", min, max);
        Polynomial {
            id: POLYNOMIAL_COMPRESSOR_ID,
            spline_coef: Vec::with_capacity(sample_count),
            /// The maximum numeric value of the points in the frame
            max_value: max as f32,  
            /// The minimum numeric value of the points in the frame
            min_value: min as f32,
            residuals: Vec::with_capacity(sample_count),
            }
    }

    // --- MANDATORY METHODS ---
    pub fn compress(&mut self, data: &[f64]) {
        
    }

    /// Decompresses data
    pub fn decompress(data: &[u8]) -> Self {
        let config = BinConfig::get();
        let (poly, _) = bincode::decode_from_slice(data, config).unwrap();
        poly
    }

    pub fn to_bytes(self) -> Vec<u8> {
        let config = BinConfig::get();
        bincode::encode_to_vec(self, config).unwrap()
    }

    pub fn to_data(&self, frame_size: usize) -> Vec<f64> {
        if self.max_value == self.min_value { 
            debug!("Same max and min, faster decompression!");
            return vec![self.max_value as f64; frame_size];
         }
        Vec::with_capacity(frame_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

}