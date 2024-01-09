use super::BinConfig;
use bincode::{Decode, Encode};
use log::{debug, info};
use std::collections::HashMap;

const CONSTANT_COMPRESSOR_ID: u8 = 30;

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
    pub fn new(sample_count: usize) -> Self {
        debug!("Constant compressor");
        Constant {
            id: CONSTANT_COMPRESSOR_ID,
            constant: 0,
            residuals: Vec::with_capacity(sample_count),
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
    pub fn get_error(self) {
        self.residuals.len();
    }

    /// This method optimizes the data conversion from f64 to i64.
    pub fn optimize(data: &[f64]) -> Vec<i64> {
        data.iter().map(|&x| x.round() as i64).collect()
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
            self.residuals = data
                .iter()
                .enumerate()
                .filter(|&(_, v)| *v != constant)
                .map(|(k, v)| (k.try_into().unwrap(), *v))
                .collect();
        }
        debug!(
            "Compressed {} elements into {} elements!",
            data.len(),
            self.residuals.len() + 1
        );
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
    pub fn to_data(&self, frame_size: usize) -> Vec<i64> {
        let mut data = vec![self.constant; frame_size];
        for (i, v) in &self.residuals {
            data[*i as usize] = *v;
        }
        data
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

pub fn constant_to_data(sample_number: usize, compressed_data: &[u8]) -> Vec<f64> {
    let c = Constant::decompress(compressed_data);
    let out_i64 = c.to_data(sample_number);
    out_i64.iter().map(|&x| x as f64).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant() {
        let vector1 = vec![1.0, 1.0, 1.0, 1.0, 1.0];
        assert_eq!(constant(&vector1), [30, 2, 0]);
    }

    #[test]
    fn test_compression() {
        let vector1 = vec![1.0, 1.0, 1.0, 1.0, 1.0];
        let mut c = Constant::new(vector1.len());
        c.compress(&Constant::optimize(&vector1));
        let bin_data = c.to_bytes();
        let c2 = Constant::decompress(&bin_data);

        assert_eq!(bin_data, [30, 2, 0]);
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
        assert_eq!(c.residuals, vec![(1, 2), (4, 3)]);
    }

    #[test]
    fn test_to_data() {
        let vector1 = vec![1.0, 2.0, 1.0, 1.0, 3.0];
        let frame_size = vector1.len();
        let compressed_data = constant(&vector1);
        let out = constant_to_data(frame_size, &compressed_data);
        assert_eq!(vector1, out);
    }

    #[test]
    fn test_optimize() {
        let vector1 = vec![1.1, 2.5, 3.8, 4.2, 5.7];
        let optimized_data = Constant::optimize(&vector1);
        assert_eq!(optimized_data, vec![1, 3, 4, 4, 6]);
    }
}
