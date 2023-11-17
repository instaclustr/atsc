use crate::utils::{round_f64, DECIMAL_PRECISION};

use super::BinConfig;
use bincode::{Decode, Encode};
use log::{debug, info};
use splines::{Interpolation, Key, Spline};

const POLYNOMIAL_COMPRESSOR_ID: u8 = 2;

#[derive(Encode, Decode, PartialEq, Debug, Clone)]
pub struct Polynomial {
    /// Compressor ID
    pub id: u8,
    /// Stored Points
    pub data_points: Vec<f64>,
    /// The maximum numeric value of the points in the frame
    pub max_value: f32,
    pub max_position: usize,
    /// The minimum numeric value of the points in the frame
    pub min_value: f32,  
    pub min_position: usize,
    /// To reduce error, is it worth it?
    pub residuals: Vec<(i32, i64)>,
}

impl Polynomial {
    pub fn new(sample_count: usize, min: f64, max: f64) -> Self {
        debug!("Polynomial compressor: min:{} max:{}", min, max);
        Polynomial {
            id: POLYNOMIAL_COMPRESSOR_ID,
            data_points: Vec::with_capacity(sample_count),
            /// The maximum numeric value of the points in the frame
            max_value: max as f32,  
            /// The minimum numeric value of the points in the frame
            min_value: min as f32,
            min_position: 0,
            max_position: 0,
            residuals: Vec::with_capacity(sample_count),
            }
    }

    pub fn set_pos(&mut self, pmin: usize, pmax: usize){
        self.min_position = pmin;
        self.max_position = pmax;
    }

    fn locate_in_data_points(&self, point: f64) -> bool {
        self.data_points.iter().any(|&i| i==point)
    }

    // --- MANDATORY METHODS ---
    pub fn compress(&mut self, data: &[f64]) {
        if self.max_value == self.min_value { 
            debug!("Same max and min, we're done here!");
            return
         }
        // The algorithm is simple, Select 10% of the data points, calculate the IDW based on those data points
        // Plus the max and min
        let data_len = data.len();
        // The minimum is a 3 point interpolation, otherwise, 1% of the data is used
        let point_count = if 3 >= (data_len/100) { 3 } else { data_len/100 };
        // I can calculate the positions from here
        let points: Vec<f64> = (0..data_len).step_by(data_len/point_count).map(|f| f as f64).collect();
        // I need to extract the values for those points
        let mut values: Vec<f64> = points.iter().map(|&f| data[f as usize]).collect();
        
        // I need to insert MIN and MAX only if they don't belong to the values already.
        if self.min_position as f64 > *points.last().unwrap() {
            // The position is behind the last point, so add it there
            values.push(self.min_value as f64)
        } else if !values.iter().any(|&i| i==self.min_value as f64) {
            // Do the position exists already?
            debug!("Min Inserted {} at index {}", self.min_value, self.min_position);
            values.insert(self.min_position, self.min_value as f64);
        }
        if self.max_position as f64 > *points.last().unwrap() {
            // The position is behind the last point, so add it there
            values.push(self.max_value as f64)
        } else if !values.iter().any(|&i| i==self.max_value as f64)  {
            debug!("Max Inserted {} at index {}", self.max_value, self.max_position);
            values.insert(self.max_position, self.max_value as f64);
        }
        debug!("Points: {:?}", points);
        debug!("Values: {:?}", values);
        self.data_points = values; 
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
        // How many points I should have
        let point_count = if 3 >= (frame_size/100) { 3 } else { frame_size/100 };
        // If they differ, it means I added either max and/or min
        let point_dif = self.data_points.len() - point_count;
        // I can calculate the positions from here
        let mut points: Vec<f64> = (0..frame_size).step_by(frame_size/point_count).map(|f| f as f64).collect();
        // Insert the position of min and max
        debug!("Points diff: {}", point_dif);
        if point_dif > 0 {
            if self.min_position as f64 > *points.last().unwrap() {
                // The position is behind the last point, so add it there
                points.push(self.min_position as f64)
            } else if !self.locate_in_data_points(self.min_value as f64) {
                // Do the position exists already?
                points.insert(self.min_position, self.min_position as f64);
            }
            if self.max_position as f64 > *points.last().unwrap() {
                // The position is behind the last point, so add it there
                points.push(self.max_position as f64)
            } else if !self.locate_in_data_points(self.max_value as f64)  {
                points.insert(self.max_position, self.max_position as f64);
            }
        }
        debug!("{} {}", points.len(), self.data_points.len());
        // Create the interpolation
        let mut key_vec = Vec::with_capacity(points.len());
        let mut current_key = 0;
        for (point, value) in points.iter().zip(self.data_points.iter()) {
            // CatmullRom needs at least 1 key behind and 2 ahead so this check.
            let interpolation = 
                if current_key > 0 && points.len() - current_key > 2 { Interpolation::CatmullRom }
                else { Interpolation::Linear };
            key_vec.push(Key::new(*point, *value, interpolation));
            current_key += 1;
        }
        let spline = Spline::from_vec(key_vec);
        // Build the data
        (0..frame_size).map(|f| round_f64(spline.clamped_sample(f as f64).unwrap(), DECIMAL_PRECISION)).collect() 
    }
}

pub fn polynomial(data: &[f64]) -> Vec<u8> {
    info!("Initializing Polynomial Compressor");
    let mut min = data[0];
    let mut max = data[0];
    let mut pmin = 0;
    let mut pmax = 0;
    // For these one we need to store where the min and max happens on the data, not only their values
    for (position, value) in data.iter().enumerate(){
        if value > &max { max = *value;  pmax = position;};
        if value < &min { min = *value;  pmin = position; };
    }
    // Initialize the compressor
    let mut c = Polynomial::new(data.len(), min, max);
    c.set_pos(pmin, pmax);
    // Convert the data
    c.compress(data);
    // Convert to bytes
    c.to_bytes()
}

/// Uncompress a FFT data
pub fn polynomial_to_data(sample_number: usize, compressed_data: &[u8]) -> Vec<f64> {
    let c = Polynomial::decompress(compressed_data);
    c.to_data(sample_number)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_polynomial() {
        let vector1 = vec![1.0, 0.0, 1.0, 1.0, 2.0, 1.0, 1.0, 1.0, 3.0, 1.0, 1.0, 5.0];
        assert_eq!(polynomial(&vector1), [2, 5, 0, 0, 0, 0, 0, 0, 240, 63, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 8, 64, 0, 0, 0, 0, 0, 0, 20, 64, 0, 0, 160, 64, 11, 0, 0, 0, 0, 1, 0]);
    }

    #[test]
    fn test_polynomial_compression() {
        let vector1 = vec![1.0, 2.0, 3.0, 4.0, 5.0, 1.0, 1.0, 1.0, 6.0, 0.0, 1.0, 9.0];
        let frame_size = vector1.len();
        let idw_data = polynomial(&vector1);
        let out = Polynomial::decompress(&idw_data).to_data(frame_size);
        assert_eq!(out, [1.0, 2.0, 3.0, 4.0, 5.0, 5.69531, 6.3125, 6.52344, 6.0, 0.0, 4.5, 9.0]);
    }

    #[test]
    fn test_polynomial_linear_compression() {
        let vector1 = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0];
        let frame_size = vector1.len();
        let idw_data = polynomial(&vector1);
        let out = Polynomial::decompress(&idw_data).to_data(frame_size);
        assert_eq!(out, [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0]);
    }
}