/// IDW - Inverse Distance Weight
/// This could be a subset of polynomial, or a replacement altogether
/// The approach is very similar for both, but the methdo is different, and in some signals
/// IDW (theoriticaly) makes more sense.
/// So by having 2 we can check which one makes more sense (or keep both)

use super::BinConfig;
use bincode::{Decode, Encode};
use log::{debug, info};
use inverse_distance_weight::IDW;

const IDW_COMPRESSOR_ID: u8 = 3;

#[derive(Encode, Decode, PartialEq, Debug, Clone)]
pub struct Idw {
    /// Compressor ID
    pub id: u8,
    /// Stored frequencies
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

impl Idw {
    pub fn new(sample_count: usize, min: f64, max: f64) -> Self {
        debug!("Inverse Distance Weight compressor: min:{} max:{}", min, max);
        Idw {
            id: IDW_COMPRESSOR_ID,
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
         let points: Vec<f64> = (0..point_count).step_by(data_len/point_count).map(|f| f as f64).collect();
         // I need to extract the values for those points
         let values: Vec<f64> = points.iter().map(|&f| data[f as usize]).collect();
         // Calculate IDW - I only need this on decompression...
         // let idw = IDW::new(points, values);
         self.data_points = values;
    }

    /// Decompresses data
    pub fn decompress(data: &[u8]) -> Self {
        let config = BinConfig::get();
        let (idw, _) = bincode::decode_from_slice(data, config).unwrap();
        idw
    }

    pub fn to_bytes(self) -> Vec<u8> {
        let config = BinConfig::get();
        bincode::encode_to_vec(self, config).unwrap()
    }

    // TODO: Insert MIN and MAX
    pub fn to_data(&self, frame_size: usize) -> Vec<f64> {
        if self.max_value == self.min_value { 
            debug!("Same max and min, faster decompression!");
            return vec![self.max_value as f64; frame_size];
         }
        let data_len = self.data_points.len();
        // From the number of data points and the frame size, I can calculate the step and generate the points
        let point_count = data_len;
         // I can calculate the positions from here
         let points: Vec<f64> = (0..point_count).step_by(data_len/point_count).map(|f| f as f64).collect();
         let idw = IDW::new(points, self.data_points.clone());
         // Build the data
         (0..frame_size).map(|f| idw.evaluate(f as f64)).collect() 
    }
}

pub fn idw(data: &[f64]) -> Vec<u8> {
    info!("Initializing IDW Compressor");
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
    let mut c = Idw::new(data.len(), min, max);
    c.set_pos(pmin, pmax);
    // Convert the data
    c.compress(data);
    // Convert to bytes
    c.to_bytes()
}

/// Uncompress a IDW data
pub fn idw_to_data(sample_number: usize, compressed_data: &[u8]) -> Vec<f64> {
    let c = Idw::decompress(compressed_data);
    c.to_data(sample_number)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idw() {
        let vector1 = vec![1.0, 1.0, 1.0, 1.0, 2.0, 1.0, 1.0, 1.0, 3.0, 1.0, 1.0, 5.0];
        assert_eq!(idw(&vector1), [3, 1, 0, 0, 0, 0, 0, 0, 240, 63, 0, 0, 160, 64, 11, 0, 0, 128, 63, 0, 0]);
    }

    #[test]
    fn test_idw_compression() {
        let vector1 = vec![1.0, 2.0, 3.0, 4.0, 5.0, 1.0, 1.0, 1.0, 5.0, 1.0, 1.0, 5.0];
        let frame_size = vector1.len();
        let idw_data = idw(&vector1);
        let out = Idw::decompress(&idw_data).to_data(frame_size);
        assert_eq!(out, [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
    }
}