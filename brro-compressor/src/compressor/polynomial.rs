use crate::utils::{DECIMAL_PRECISION, error::error_smape, round_and_limit_f64};

use super::BinConfig;
use bincode::{Decode, Encode};
use inverse_distance_weight::IDW;
use log::{debug, info, trace};
use splines::{Interpolation, Key, Spline};

const POLYNOMIAL_COMPRESSOR_ID: u8 = 2;
const IDW_COMPRESSOR_ID: u8 = 3;

#[derive(Encode, Decode, Default, Debug, Clone)]
pub enum Method {
    #[default]
    CatmullRom,
    Idw,
}

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
    pub fn new(sample_count: usize, min: f64, max: f64, idw: bool) -> Self {
        debug!("Polynomial compressor: min:{} max:{}, idw: {}", min, max, idw);
        Polynomial {
            id: if idw { IDW_COMPRESSOR_ID } else { POLYNOMIAL_COMPRESSOR_ID },
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

    fn get_method(&self) -> Method {
        match self.id {
            IDW_COMPRESSOR_ID => Method::Idw,
            POLYNOMIAL_COMPRESSOR_ID => Method::CatmullRom,
            _ => panic!("Unknown Compressor method!")
        }
    }

    pub fn compress_bounded(&mut self, data: &[f64], max_err: f64) {
        // TODO: Big one, read below
        // To reduce error we add more points to the polynomial, but, we also might add residuals
        // each residual is 1/data_lenght * 100% less compression, each jump is 5% less compression. 
        // We can do the math and pick the one which fits better. 
        let method = self.get_method();
        let data_len = data.len();
        let baseline_points = if 3 >= (data_len/100) { 3 } else { data_len/100 };
        
        // Variables for the error control loop
        let mut current_err = max_err + 1.0;
        let mut jump: usize = 0;
        let mut iterations = 0;
        
        while ((max_err * 1000.0) as i32) < ((current_err * 1000.0) as i32) {
            iterations += 1;
            self.compress_hinted(data, baseline_points+jump);
            let out_data = match method {
                Method::CatmullRom => self.polynomial_to_data(data_len),
                Method::Idw => self.idw_to_data(data_len)
            };
            trace!("Calculated Values: {:?}", out_data);
            trace!("Data Values: {:?}", data);
            current_err = error_smape(data, &out_data);
            trace!("Current Err: {}", current_err);
            // Max iterations is 18 (We start at 10%, we can go to 95% and 1% at a time)
            match iterations {
                1..=17 => jump += baseline_points/2,
                18..=22 => jump += baseline_points/10,
                _ => break
            }
            if self.data_points.len() == data_len {
                // Storing the whole thing anyway...
                break;
            }
        }
    } 

    pub fn compress_hinted(&mut self, data: &[f64], points: usize) {
        if self.max_value == self.min_value { 
            debug!("Same max and min, we're done here!");
            return
         }
        // The algorithm is simple, Select 10% of the data points, calculate the Polynomial based on those data points
        // Plus the max and min
        let data_len = data.len();
        // Instead of calculation, we use the provided count
        let point_count = points;
        // I can calculate the positions from here
        let mut points: Vec<f64> = (0..data_len).step_by(data_len/point_count).map(|f| f as f64).collect();
        points.push((data_len-1) as f64);
        // I need to extract the values for those points
        let mut values: Vec<f64> = points.iter().map(|&f| data[f as usize]).collect();
        
        debug!("Points: {:?}", points);
        debug!("Values: {:?}", values);

        // I need to insert MIN and MAX only if they don't belong to the values already.
        let mut prev_pos = points[0];
        for (array_position, position_value) in points.iter().enumerate() {
            if self.min_position > (prev_pos.round() as usize) && self.min_position < (position_value.round() as usize) {
                // We have to insert here
                values.insert(array_position, self.min_value as f64);
            }
            if self.max_position > (prev_pos.round() as usize) && self.max_position < (position_value.round() as usize) {
                // We have to insert here
                values.insert(array_position, self.max_value as f64);
                // And we are done
            }
            prev_pos = *position_value;
        }

        self.data_points = values; 
    }

    // --- MANDATORY METHODS ---
    pub fn compress(&mut self, data: &[f64]) {
        if self.max_value == self.min_value { 
            debug!("Same max and min, we're done here!");
            return
         }
        // The algorithm is simple, Select 10% of the data points, calculate the Polynomial based on those data points
        // Plus the max and min
        let data_len = data.len();
        // The minimum is a 3 point interpolation, otherwise, 1% of the data is used
        let point_count = if 3 >= (data_len/100) { 3 } else { data_len/100 };
        // I can calculate the positions from here
        let mut points: Vec<f64> = (0..data_len).step_by(data_len/point_count).map(|f| f as f64).collect();
        // Also we always use the last point
        points.push((data_len-1) as f64);
        // I need to extract the values for those points
        let mut values: Vec<f64> = points.iter().map(|&f| data[f as usize]).collect();
        // I need to insert MIN and MAX only if they don't belong to the values already.
        let mut prev_pos = points[0];
        for (array_position, position_value) in points.iter().enumerate() {
            if self.min_position > (prev_pos.round() as usize) && self.min_position < (position_value.round() as usize) {
                // We have to insert here
                values.insert(array_position, self.min_value as f64);
            }
            if self.max_position > (prev_pos.round() as usize) && self.max_position < (position_value.round() as usize) {
                // We have to insert here
                values.insert(array_position, self.max_value as f64);
                // And we are done
            }
            prev_pos = *position_value;
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

    /// Since IDW and Polynomial are the same code everywhere, this function prepares the data
    /// to be used by one of the polynomial decompression methods
    fn get_positions(&self, frame_size: usize) -> Vec<f64> {
        // How many points I should have
        let point_count = if 3 >= (frame_size/100) { 3 } else { frame_size/100 };
        // If they differ, it means I added either max and/or min
        let point_dif = self.data_points.len() - point_count;
        // I can calculate the positions from here
        let mut points: Vec<f64> = (0..frame_size).step_by(frame_size/point_count).map(|f| f as f64).collect();
        // Also we always use the last point
        points.push((frame_size-1) as f64);
        // Insert the position of min and max
        debug!("Points diff: {}", point_dif);
        if point_dif > 0 {
            let mut prev_pos = points[0];
            for (array_position, position_value) in points.clone().iter().enumerate() {
                if self.min_position > (prev_pos.round() as usize) && self.min_position < (position_value.round() as usize) {
                    // We have to insert here
                    points.insert(array_position, self.min_position as f64);
                }
                if self.max_position > (prev_pos.round() as usize) && self.max_position < (position_value.round() as usize) {
                    // We have to insert here
                    points.insert(array_position, self.max_position as f64);
                }
                prev_pos = *position_value;
            }
        }
        debug!("{} {}", points.len(), self.data_points.len());
        debug!("Points: {:?}", points);
        debug!("Out Values: {:?}", self.data_points);
        points
    }

    pub fn polynomial_to_data(&self, frame_size: usize) -> Vec<f64> {
        if self.max_value == self.min_value { 
            debug!("Same max and min, faster decompression!");
            return vec![self.max_value as f64; frame_size];
         }
        // Create the interpolation
        let points = self.get_positions(frame_size);
        let mut key_vec = Vec::with_capacity(points.len());
        for (current_key, (point, value)) in points.iter().zip(self.data_points.iter()).enumerate() {
            // CatmullRom needs at least 1 key behind and 2 ahead so this check.
            let interpolation = 
                if current_key > 0 && points.len() - current_key > 2 { Interpolation::CatmullRom }
                else { Interpolation::Linear };
            key_vec.push(Key::new(*point, *value, interpolation));
        }
        let spline = Spline::from_vec(key_vec);
        // Build the data
        // TODO: It gives values below minimum
        (0..frame_size)
        .map(|f| round_and_limit_f64(spline.clamped_sample(f as f64).unwrap(), self.min_value.into(), self.max_value.into(), DECIMAL_PRECISION))
        .collect() 
    }

    pub fn idw_to_data(&self, frame_size: usize) -> Vec<f64> {
        let points = self.get_positions(frame_size);
        let idw = IDW::new(points, self.data_points.clone());
        // Build the data
        // TODO: It gives values below minimum
        (0..frame_size)
        .map(|f| round_and_limit_f64(idw.evaluate(f as f64), self.min_value.into(), self.max_value.into(), DECIMAL_PRECISION))
        .collect() 
    }

    pub fn to_data(&self, frame_size: usize) -> Vec<f64> {
        match self.id {
            IDW_COMPRESSOR_ID => self.idw_to_data(frame_size),
            POLYNOMIAL_COMPRESSOR_ID => self.polynomial_to_data(frame_size),
            _ => panic!("Unknown compressor method!")
        }
    }

}

pub fn polynomial(data: &[f64], idw: bool) -> Vec<u8> {
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
    let mut c = Polynomial::new(data.len(), min, max, idw);
    c.set_pos(pmin, pmax);
    // Convert the data
    c.compress(data);
    // Convert to bytes
    c.to_bytes()
}

pub fn polynomial_allowed_error(data: &[f64], allowed_error: f64, idw: bool) -> Vec<u8> {
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
    let mut c = Polynomial::new(data.len(), min, max, idw);
    c.set_pos(pmin, pmax);
    // Convert the data
    c.compress_bounded(data, allowed_error);
    // Convert to bytes
    c.to_bytes()
}

/// Uncompress 
pub fn to_data(sample_number: usize, compressed_data: &[u8]) -> Vec<f64> {
    let c = Polynomial::decompress(compressed_data);
    c.to_data(sample_number)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_polynomial() {
        let vector1 = vec![1.0, 0.0, 1.0, 1.0, 2.0, 1.0, 1.0, 1.0, 3.0, 1.0, 1.0, 5.0];
        assert_eq!(polynomial(&vector1, false), [2, 5, 0, 0, 0, 0, 0, 0, 240, 63, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 8, 64, 0, 0, 0, 0, 0, 0, 20, 64, 0, 0, 160, 64, 11, 0, 0, 0, 0, 1, 0]);
    }

    #[test]
    fn test_polynomial_compression() {
        let vector1 = vec![1.0, 1.0, 1.0, 1.0, 2.0, 3.0, 5.0, 1.0, 2.0, 7.0, 1.0, 1.0, 1.0, 3.0, 1.0, 1.0, 5.0];
        let frame_size = vector1.len();
        let idw_data = polynomial(&vector1, false);
        let out = Polynomial::decompress(&idw_data).to_data(frame_size);
        assert_eq!(out, [1.0, 1.4, 1.8, 2.2, 2.6, 3.0, 4.075, 5.53333, 6.725, 7.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 5.0]);
    }

    #[test]
    fn test_polynomial_linear_compression() {
        let vector1 = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0];
        let frame_size = vector1.len();
        let idw_data = polynomial(&vector1, false);
        let out = Polynomial::decompress(&idw_data).to_data(frame_size);
        assert_eq!(out, [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0]);
    }

    #[test]
    fn test_to_allowed_error() {
        let vector1 = vec![1.0, 1.0, 1.0, 1.0, 2.0, 3.0, 5.0, 1.0, 2.0, 7.0, 1.0, 1.0, 1.0, 3.0, 1.0, 1.0, 5.0];
        let frame_size = vector1.len();
        let compressed_data = polynomial_allowed_error(&vector1, 0.02, false);
        let out = Polynomial::decompress(&compressed_data).to_data(frame_size);
        let e = error_smape(&vector1, &out);
        assert!(e <= 0.02);
    }

    #[test]
    fn test_idw_compression() {
        let vector1 = vec![1.0, 1.0, 1.0, 1.0, 2.0, 3.0, 5.0, 1.0, 2.0, 7.0, 1.0, 1.0, 1.0, 3.0, 1.0, 1.0, 5.0];
        let frame_size = vector1.len();
        let idw_data = polynomial(&vector1, true);
        let out = Polynomial::decompress(&idw_data).to_data(frame_size);
        assert_eq!(out, [1.0, 1.21502, 1.89444, 2.63525, 2.97975, 3.0, 3.21181, 4.10753, 5.44851, 7.0, 1.0, 2.23551, 2.70348, 2.5293, 1.92317, 1.0, 5.0]);
    }

    #[test]
    fn test_idw_linear_compression() {
        let vector1 = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0];
        let frame_size = vector1.len();
        let idw_data = polynomial(&vector1, true);
        let out = Polynomial::decompress(&idw_data).to_data(frame_size);
        assert_eq!(out, [1.0, 1.62873, 3.51429, 4.84995, 5.0, 5.40622, 7.05871, 8.64807, 9.0, 9.37719, 11.18119, 12.0]);
    }

    #[test]
    fn test_idw_to_allowed_error() {
        let vector1 = vec![1.0, 1.0, 1.0, 1.0, 2.0, 3.0, 5.0, 1.0, 2.0, 7.0, 1.0, 1.0, 1.0, 3.0, 1.0, 1.0, 5.0];
        let frame_size = vector1.len();
        let compressed_data = polynomial_allowed_error(&vector1, 0.02, true);
        let out = Polynomial::decompress(&compressed_data).to_data(frame_size);
        let e = error_smape(&vector1, &out);
        assert!(e <= 0.02);
    }
}