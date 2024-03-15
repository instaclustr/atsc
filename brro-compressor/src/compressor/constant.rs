use crate::{compressor::CompressorResult, optimizer::utils::DataStats};

use super::BinConfig;
use bincode::{Decode, Encode};
use log::debug;

const CONSTANT_COMPRESSOR_ID: u8 = 30;

/// This is a temporary implementation, other implementations (FFT, Polynomial) might provide the same result
/// as going through the data anyway.
#[derive(Encode, Decode, PartialEq, Debug, Clone)]
pub struct Constant {
    pub id: u8,
    pub constant: f64,
    pub residuals: Vec<(i32, f64)>,
}

impl Constant {
    /// Creates a new instance of the Constant compressor with the size needed to handle the worst case
    pub fn new(sample_count: usize, constant_value: f64) -> Self {
        debug!("Constant compressor");
        Constant {
            id: CONSTANT_COMPRESSOR_ID,
            constant: constant_value,
            residuals: Vec::with_capacity(sample_count),
        }
    }

    /// This compressor is about having a single constant for the whole segment
    pub fn set_constant(&mut self, constant_value: f64) {
        self.constant = constant_value;
    }

    /// Adds a residual
    pub fn add_residual(&mut self, sample_number: i32, value: f64) {
        self.residuals.push((sample_number, value));
    }

    /// returns the error value
    pub fn get_residual_count(self) {
        self.residuals.len();
    }

    /// Receives a data stream and generates a Constant
    pub fn decompress(data: &[u8]) -> Self {
        let config = BinConfig::get();
        let (ct, _) = bincode::decode_from_slice(data, config).unwrap();
        ct
    }

    /// This function transforms the structure into a Binary stream
    pub fn to_bytes(&self) -> Vec<u8> {
        // Use Bincode and flate2-rs? Do this at the Stream Level?
        let config = BinConfig::get();
        bincode::encode_to_vec(self, config).unwrap()
    }

    /// Returns an array of data. It creates an array of data the size of the frame with a constant value
    /// and pushes the residuals to the right place.
    pub fn to_data(&self, frame_size: usize) -> Vec<f64> {
        let mut data = vec![self.constant; frame_size];
        for (i, v) in &self.residuals {
            data[*i as usize] = *v;
        }
        data
    }
}

pub fn constant_compressor(data: &[f64], stats: DataStats) -> CompressorResult {
    debug!("Initializing Constant Compressor. Error and Stats provided");
    // Initialize the compressor
    let c = Constant::new(data.len(), stats.min);
    // Convert to bytes
    CompressorResult::new(c.to_bytes(), 0.0)
}

pub fn constant_to_data(sample_number: usize, compressed_data: &[u8]) -> Vec<f64> {
    let c = Constant::decompress(compressed_data);
    c.to_data(sample_number)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant() {
        let vector1 = vec![1.0, 1.0, 1.0, 1.0, 1.0];
        assert_eq!(
            Constant::new(vector1.len(), DataStats::new(&vector1).min).to_bytes(),
            [30, 2, 0]
        );
    }

    #[test]
    fn test_compression() {
        let vector1 = vec![1.0, 1.0, 1.0, 1.0, 1.0];
        let mut c = Constant::new(vector1.len(), DataStats::new(&vector1).min).to_bytes();
        let c2 = constant_to_data(vector1.len(), &c);

        assert_eq!(vector1, c2);
    }
}
