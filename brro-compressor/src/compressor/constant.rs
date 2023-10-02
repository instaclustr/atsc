use std::collections::HashMap;
use bincode::{Decode, Encode};
use super::BinConfig;
use log::{info, debug};

const CONSTANT_COMPRESSOR_ID: u8 = 0;

/// This is a temporary implementation, other implementations (FFT, Polynomial) might provide the same result
/// as going through the data anyway.
#[derive(Encode, Decode, PartialEq, Debug, Clone)]
pub struct Constant {
    pub id: u8,
    pub constant: i64,
    pub residuals: Vec<(i32, i64)>,
}

impl Constant {
    /// Creates a new instance of the Constant compressor with the size needed to handle the worst case
    pub fn new(frame_size: usize) -> Self {
        debug!("Constant compressor");
        Constant {
            id: CONSTANT_COMPRESSOR_ID,
            constant: 0,
            residuals: Vec::with_capacity(frame_size),
        }
    }

    /// This compressor is about having a single constant for the whole segment
    pub fn set_constant(&mut self, constant_value: i64) {
        self.constant = constant_value;
    }

    /// Adds a residual
    pub fn add_residual(&mut self, sample_number: i32, value: i64) {
        self.residuals.push((sample_number, value));
    }

    /// returns the error value
    pub fn get_error(self){
        self.residuals.len();
    }

    /// Currently the data is provided in f64, this compressor needs i64. So the optimizer needs
    /// to get this out for the compressor
    /// TODO: Make this work decently, right now is only doing a cast (And maybe that is it?)
    pub fn optimize(data: &[f64]) -> Vec<i64> {
        let mut out_vec = Vec::with_capacity(data.len());
        for element in data {
            out_vec.push(*element as i64);
        }
        out_vec
    }

    /// Compresses the data. Walks the data array and sets one value as the constant.
    /// Performance consideration, we do O(3*n) in the worst case, best case is O(n). 
    pub fn compress(&mut self, data: &[i64]) {
        // Count occurrences of each value in the data
        let mut seen_values = HashMap::new();
        for &val in data {
            *seen_values.entry(val).or_insert(0) += 1;
        }

        // Find the constant and residuals in a single iteration
        let mut constant = 0;
        let mut hit_count = 0;

        for (&val, &count) in &seen_values {
            // This count is bigger than the previous, so, until now, it is the biggest count (and the constant)
            if count > hit_count {
                constant = val;
                hit_count = count;
            }
        }
        self.set_constant(constant);
        // if there is more than 1 element in the map, we have to walk the initial array
        if seen_values.len() > 1 {
            // Walk the initial array (again) and push anything not matching the constant to the residuals
            self.residuals = data.iter().enumerate()
                            .filter(|&(_,v)| *v != constant)
                            .map(|(k,v)| (k.try_into().unwrap(), *v))
                            .collect();
        }
    }

    /// Receives a data stream and generates a Constant
    pub fn decompress(data: &Vec<u8>) -> Self {
        let config = BinConfig::get();
        match bincode::decode_from_slice(&data, config) {
            Ok((constant, _)) => constant,
            Err(e) => panic!("{e}")
        }
    }

    /// This function transforms the structure into a Binary stream
    pub fn to_bytes(&self) -> Vec<u8> {
        // Use Bincode and flate2-rs? Do this at the Stream Level?
        let config = BinConfig::get();
        bincode::encode_to_vec(self, config).unwrap()
    }

    /// Returns an array of data. It creates an array of data the size of the frame with a constant value
    /// and pushes the residuals to the right place.
    pub fn to_data(&self) -> Vec<f64> {
        Vec::new()
    }

}

pub fn constant(data: &[f64]) -> Vec<u8> {
    info!("[Compressor] Initializing Constant Compressor");
    // Initialize the compressor
    let mut c = Constant::new(data.len());
    // Convert the data via the optimizer
    c.compress(&Constant::optimize(data));
    // Convert to bytes
    c.to_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant() {
        let vector1 = vec![1.0, 1.0, 1.0, 1.0, 1.0];
        assert_eq!(constant(&vector1), [0, 2, 0]);
    }

    #[test]
    fn test_compression() {
        let vector1 = vec![1.0, 1.0, 1.0, 1.0, 1.0];
        let mut c = Constant::new(vector1.len());
        c.compress(&Constant::optimize(&vector1));
        let bin_data = c.to_bytes();
        let c2 = Constant::decompress(&bin_data);
        
        assert_eq!(bin_data, [0, 2, 0]);
        assert_eq!(c.clone(), c2);
    }

    #[test]
    fn test_define_constant() {
        let vector1 = vec![1.0, 1.0, 1.0, 1.0, 1.0];
        let mut c = Constant::new(vector1.len());
        c.compress(&Constant::optimize(&vector1));

        assert!(c.constant == 1);
    }

    #[test]
    fn test_define_residuals() {
        let vector1 = vec![1.0, 2.0, 1.0, 1.0, 3.0];
        let mut c = Constant::new(vector1.len());
        c.compress(&Constant::optimize(&vector1));

        assert!(c.constant == 1);
        assert_eq!(c.residuals, vec![(1,2),(4,3)]);
    }
}