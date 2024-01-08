use crate::optimizer::utils::{Bitdepth, DataStats};
use crate::utils::{error::calculate_error, round_and_limit_f64, round_f64, DECIMAL_PRECISION};

use super::{BinConfig, CompressorResult};
use bincode::{Decode, Encode};
use inverse_distance_weight::IDW;
use log::{debug, info, trace};
use splines::{Interpolation, Key, Spline};

const POLYNOMIAL_COMPRESSOR_ID: u8 = 0;
const IDW_COMPRESSOR_ID: u8 = 1;

#[derive(Encode, Decode, Default, Debug, Clone, PartialEq)]
pub enum PolynomialType {
    #[default]
    Polynomial = 0,
    Idw = 1,
}

#[derive(Encode, Decode, Default, Debug, Clone)]
pub enum Method {
    #[default]
    CatmullRom,
    Idw,
}

#[derive(PartialEq, Debug, Clone)]
pub struct Polynomial {
    /// Compressor ID
    pub id: PolynomialType,
    /// Stored Points
    pub data_points: Vec<f64>,
    pub min: f64,
    pub max: f64,
    /// What is the base step between points
    pub point_step: u8,
    /// Compression error
    pub error: Option<f64>,
    /// Target bitdepth
    pub bitdepth: Bitdepth,
}

impl Encode for Polynomial {
    fn encode<__E: ::bincode::enc::Encoder>(
        &self,
        encoder: &mut __E,
    ) -> Result<(), ::bincode::error::EncodeError> {
        Encode::encode(&self.id, encoder)?;
        Encode::encode(&self.bitdepth, encoder)?;
        match &self.bitdepth {
            Bitdepth::U8 => {
                debug!("Encoding as u8");
                let vec_u8: Vec<u8> = self.data_points.iter().map(|f| *f as u8).collect();
                Encode::encode(&vec_u8, encoder)?;
            }
            Bitdepth::I16 => {
                debug!("Encoding as i16");
                let vec_i16: Vec<i16> = self.data_points.iter().map(|f| *f as i16).collect();
                Encode::encode(&vec_i16, encoder)?;
            }
            Bitdepth::I32 => {
                debug!("Encoding as i32");
                let vec_i32: Vec<i32> = self.data_points.iter().map(|f| *f as i32).collect();
                Encode::encode(&vec_i32, encoder)?;
            }
            Bitdepth::F64 => {
                debug!("Encoding as f64");
                Encode::encode(&self.data_points, encoder)?;
            }
        }
        Encode::encode(&self.min, encoder)?;
        Encode::encode(&self.max, encoder)?;
        Encode::encode(&self.point_step, encoder)?;
        Ok(())
    }
}

impl Decode for Polynomial {
    fn decode<__D: ::bincode::de::Decoder>(
        decoder: &mut __D,
    ) -> Result<Self, ::bincode::error::DecodeError> {
        let id = Decode::decode(decoder)?;
        let bitdepth = Decode::decode(decoder)?;
        // Here is where the pig twists the tail
        let data_points: Vec<f64> = match bitdepth {
            Bitdepth::U8 => {
                debug!("Decoding as u8");
                let vec_u8: Vec<u8> = Decode::decode(decoder)?;
                vec_u8.iter().map(|f| *f as f64).collect()
            }
            Bitdepth::I16 => {
                debug!("Decoding as i16");
                let vec_i16: Vec<i16> = Decode::decode(decoder)?;
                vec_i16.iter().map(|f| *f as f64).collect()
            }
            Bitdepth::I32 => {
                debug!("Decoding as i32");
                let vec_i32: Vec<i32> = Decode::decode(decoder)?;
                vec_i32.iter().map(|f| *f as f64).collect()
            }
            Bitdepth::F64 => {
                debug!("Decoding as f64");
                let vec_f64: Vec<f64> = Decode::decode(decoder)?;
                vec_f64
            }
        };
        let min = Decode::decode(decoder)?;
        let max = Decode::decode(decoder)?;
        let point_step = Decode::decode(decoder)?;

        Ok(Self {
            id,
            bitdepth,
            data_points,
            min,
            max,
            point_step,
            error: None,
        })
    }
}

impl<'__de> ::bincode::BorrowDecode<'__de> for Polynomial {
    fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de>>(
        decoder: &mut __D,
    ) -> Result<Self, ::bincode::error::DecodeError> {
        let id = ::bincode::BorrowDecode::borrow_decode(decoder)?;
        let bitdepth = ::bincode::BorrowDecode::borrow_decode(decoder)?;
        // Here is where the pig twists the tail
        let data_points: Vec<f64> = match bitdepth {
            Bitdepth::U8 => {
                debug!("Decoding as u8");
                let vec_u8: Vec<u8> = ::bincode::BorrowDecode::borrow_decode(decoder)?;
                vec_u8.iter().map(|f| *f as f64).collect()
            }
            Bitdepth::I16 => {
                debug!("Decoding as i16");
                let vec_i16: Vec<i16> = ::bincode::BorrowDecode::borrow_decode(decoder)?;
                vec_i16.iter().map(|f| *f as f64).collect()
            }
            Bitdepth::I32 => {
                debug!("Decoding as i32");
                let vec_i32: Vec<i32> = ::bincode::BorrowDecode::borrow_decode(decoder)?;
                vec_i32.iter().map(|f| *f as f64).collect()
            }
            Bitdepth::F64 => {
                debug!("Decoding as f64");
                let vec_f64: Vec<f64> = ::bincode::BorrowDecode::borrow_decode(decoder)?;
                vec_f64
            }
        };
        let min = Decode::decode(decoder)?;
        let max = Decode::decode(decoder)?;
        let point_step = Decode::decode(decoder)?;

        Ok(Self {
            id,
            bitdepth,
            data_points,
            min,
            max,
            point_step,
            error: None,
        })
    }
}

impl Polynomial {
    pub fn new(
        sample_count: usize,
        min: f64,
        max: f64,
        ptype: PolynomialType,
        bitdepth: Bitdepth,
    ) -> Self {
        debug!(
            "Polynomial compressor: min:{} max:{}, Type: {:?}",
            min, max, ptype
        );
        Polynomial {
            id: ptype,
            min,
            max,
            data_points: Vec::with_capacity(sample_count),
            // Minimum step is always 1
            point_step: 1,
            error: None,
            bitdepth,
        }
    }

    fn locate_in_data_points(&self, point: f64) -> bool {
        self.data_points.iter().any(|&i| i == point)
    }

    fn get_method(&self) -> Method {
        match self.id {
            PolynomialType::Idw => Method::Idw,
            PolynomialType::Polynomial => Method::CatmullRom,
        }
    }

    pub fn compress_bounded(&mut self, data: &[f64], max_err: f64) {
        if self.max == self.min {
            debug!("Same max and min, we're done here!");
            return;
        }
        // TODO: Big one, read below
        // To reduce error we add more points to the polynomial, but, we also might add residuals
        // each residual is 1/data_lenght * 100% less compression, each jump is 5% less compression.
        // We can do the math and pick the one which fits better.
        let method = self.get_method();
        let data_len = data.len();
        let baseline_points = if 3 >= (data_len / 100) {
            3
        } else {
            data_len / 100
        };
        // Variables for the error control loop
        let mut current_err = max_err + 1.0;
        let mut jump: usize = 0;
        let mut iterations = 0;
        // Locking max target error precision to 0.1%
        let target_error = round_f64(max_err, 3);
        while target_error < round_f64(current_err, 4) {
            trace!(
                "Method: {:?} Iterations: {} Error: {} Target: {}",
                method,
                iterations,
                current_err,
                target_error
            );
            iterations += 1;
            self.compress_hinted(data, baseline_points + jump);
            let out_data = match method {
                Method::CatmullRom => self.polynomial_to_data(data_len),
                Method::Idw => self.idw_to_data(data_len),
            };
            trace!("Calculated Values: {:?}", out_data);
            trace!("Data Values: {:?}", data);
            current_err = calculate_error(data, &out_data);
            trace!("Current Err: {}", current_err);
            // Max iterations is 18 (We start at 10%, we can go to 95% and 1% at a time)
            match iterations {
                // We should always increase by 1 in worst case
                1..=17 => jump += (data_len / 10).max(1),
                18..=22 => jump += (data_len / 100).max(1),
                // No more jumping, but we landed right in the end
                _ if target_error > round_f64(current_err, 4) => break,
                // We can't hit the target, store everything
                _ => {
                    self.compress_hinted(data, data_len);
                    // if we store everything, there is no error
                    current_err = 0.0;
                    break;
                }
            }
            if self.data_points.len() == data_len {
                // Storing the whole thing anyway...
                // if we store everything, there is no error
                current_err = 0.0;
                break;
            }
        }
        self.error = Some(current_err);
        debug!(
            "Final Stored Data Lenght: {} Iterations: {}",
            self.data_points.len(),
            iterations
        );
    }

    pub fn compress_hinted(&mut self, data: &[f64], points: usize) {
        if self.max == self.min {
            debug!("Same max and min, we're done here!");
            return;
        }
        // The algorithm is simple, Select 10% of the data points, calculate the Polynomial based on those data points
        // Plus the max and min
        let data_len = data.len();
        // Instead of calculation, we use the provided count
        let point_count = points;
        // Step size
        let step = (data_len / point_count).max(1);
        // I can calculate the positions from here
        let mut points: Vec<f64> = (0..data_len).step_by(step).map(|f| f as f64).collect();
        // Pushing the last value if needed (and if data is not empty)
        if points.last() != Some(&(data_len as f64 - 1.)) {
            points.push(data_len as f64 - 1.);
        }
        // I need to extract the values for those points
        let values: Vec<f64> = points.iter().map(|&f| data[f as usize]).collect();

        debug!("Compressed Hinted Points: {:?}", points);
        debug!("Compressed Hinted Values: {:?}", values);

        self.data_points = values;
        self.point_step = step as u8;
    }

    // --- MANDATORY METHODS ---
    pub fn compress(&mut self, data: &[f64]) {
        let points = if 3 >= (data.len() / 100) {
            3
        } else {
            data.len() / 100
        };
        self.compress_hinted(data, points)
    }

    /// Decompresses data
    pub fn decompress(data: &[u8]) -> Self {
        let config = BinConfig::get();
        let (poly, _) = bincode::decode_from_slice(data, config).unwrap();
        poly
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let config = BinConfig::get();
        bincode::encode_to_vec(self, config).unwrap()
    }

    // --- END OF MANDATORY METHODS ---
    /// Since IDW and Polynomial are the same code everywhere, this function prepares the data
    /// to be used by one of the polynomial decompression methods
    fn get_positions(&self, frame_size: usize) -> Vec<usize> {
        let mut points = Vec::with_capacity(frame_size);
        for position_value in (0..frame_size).step_by(self.point_step as usize) {
            points.push(position_value);
        }
        // Always add the last position of the frame, if needed
        if points.last() != Some(&(frame_size - 1)) {
            points.push(frame_size - 1);
        }
        trace!("points {:?}", points);
        points
    }

    pub fn polynomial_to_data(&self, frame_size: usize) -> Vec<f64> {
        // Create the interpolation
        let points = self.get_positions(frame_size);
        let mut key_vec = Vec::with_capacity(points.len());
        for (current_key, (point, value)) in points.iter().zip(self.data_points.iter()).enumerate()
        {
            // CatmullRom needs at least 1 key behind and 2 ahead so this check.
            let interpolation = if current_key > 0 && points.len() - current_key > 2 {
                Interpolation::CatmullRom
            } else {
                Interpolation::Linear
            };
            key_vec.push(Key::new(*point as f64, *value, interpolation));
        }
        let spline = Spline::from_vec(key_vec);
        // Build the data
        // There is a problem with the spline calculation, that it might get a value for all positions. In those cases
        // we return the good value calculated. If that doesn't exist, we return the minimum value
        let mut out_vec = Vec::with_capacity(frame_size);
        let mut prev = self.min;
        for value in 0..frame_size {
            let spline_value = spline.clamped_sample(value as f64).unwrap_or(prev);
            prev = spline_value;
            out_vec.push(round_and_limit_f64(
                spline_value,
                self.min,
                self.max,
                DECIMAL_PRECISION,
            ));
        }
        out_vec
    }

    pub fn idw_to_data(&self, frame_size: usize) -> Vec<f64> {
        // IDW needs f64 for points :(
        let points = self
            .get_positions(frame_size)
            .iter()
            .map(|&f| f as f64)
            .collect();
        let idw = IDW::new(points, self.data_points.clone());
        // Build the data
        (0..frame_size)
            .map(|f| {
                round_and_limit_f64(
                    idw.evaluate(f as f64),
                    self.min,
                    self.max,
                    DECIMAL_PRECISION,
                )
            })
            .collect()
    }

    pub fn to_data(&self, frame_size: usize) -> Vec<f64> {
        if self.max == self.min {
            debug!("Same max and min, faster decompression!");
            return vec![self.max; frame_size];
        }
        match self.id {
            PolynomialType::Idw => self.idw_to_data(frame_size),
            PolynomialType::Polynomial => self.polynomial_to_data(frame_size),
        }
    }
}

pub fn polynomial(data: &[f64], p_type: PolynomialType) -> Vec<u8> {
    info!("Initializing Polynomial Compressor");
    let stats = DataStats::new(data);
    // Initialize the compressor
    let mut c = Polynomial::new(data.len(), stats.min, stats.max, p_type, stats.bitdepth);
    // Convert the data
    c.compress(data);
    // Convert to bytes
    c.to_bytes()
}

pub fn polynomial_allowed_error(
    data: &[f64],
    allowed_error: f64,
    p_type: PolynomialType,
) -> CompressorResult {
    info!("Initializing Polynomial Compressor");
    let stats = DataStats::new(data);
    // Initialize the compressor
    let mut c = Polynomial::new(data.len(), stats.min, stats.max, p_type, stats.bitdepth);
    // Convert the data
    c.compress_bounded(data, allowed_error);
    CompressorResult::new(c.to_bytes(), c.error.unwrap_or(0.0))
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
    fn test_polynomial_u8() {
        let vector1 = vec![1.0, 0.0, 1.0, 1.0, 2.0, 1.0, 1.0, 1.0, 3.0, 1.0, 1.0, 5.0];
        assert_eq!(
            polynomial(&vector1, PolynomialType::Polynomial),
            [0, 3, 4, 1, 2, 3, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 20, 64, 4]
        );
    }

    #[test]
    fn test_polynomial_i16() {
        let vector1 = vec![1.0, 0.0, 1.0, 1.0, 2.0, 1.0, 1.0, 1.0, 3.0, 1.0, 1.0, 500.0];
        assert_eq!(
            polynomial(&vector1, PolynomialType::Polynomial),
            [
                0, 2, 4, 2, 4, 6, 251, 232, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 127, 64,
                4
            ]
        );
    }

    #[test]
    fn test_polynomial_i32() {
        let vector1 = vec![
            40001.0, 40000.0, 40001.0, 40001.0, 40002.0, 40001.0, 40001.0, 40001.0, 40003.0,
            40001.0, 40001.0, 40005.0,
        ];
        assert_eq!(
            polynomial(&vector1, PolynomialType::Polynomial),
            [
                0, 1, 4, 252, 130, 56, 1, 0, 252, 132, 56, 1, 0, 252, 134, 56, 1, 0, 252, 138, 56,
                1, 0, 0, 0, 0, 0, 0, 136, 227, 64, 0, 0, 0, 0, 160, 136, 227, 64, 4
            ]
        );
    }

    #[test]
    fn test_polynomial_f64() {
        let vector1 = vec![1.1, 0.1, 1.1, 1.1, 2.0, 1.0, 1.0, 1.0, 3.0, 1.0, 1.0, 5.0];
        assert_eq!(
            polynomial(&vector1, PolynomialType::Polynomial),
            [
                0, 0, 4, 154, 153, 153, 153, 153, 153, 241, 63, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0,
                0, 0, 0, 8, 64, 0, 0, 0, 0, 0, 0, 20, 64, 154, 153, 153, 153, 153, 153, 185, 63, 0,
                0, 0, 0, 0, 0, 20, 64, 4
            ]
        );
    }

    #[test]
    fn test_polynomial_compression() {
        let vector1 = vec![
            1.0, 1.0, 1.0, 1.0, 2.0, 3.0, 5.0, 1.0, 2.0, 7.0, 1.0, 1.0, 1.0, 3.0, 1.0, 1.0, 5.0,
        ];
        let frame_size = vector1.len();
        let idw_data = polynomial(&vector1, PolynomialType::Polynomial);
        let out = Polynomial::decompress(&idw_data).to_data(frame_size);
        assert_eq!(
            out,
            [
                1.0, 1.4, 1.8, 2.2, 2.6, 3.0, 2.824, 2.392, 1.848, 1.336, 1.0, 1.0, 1.0, 1.0, 1.0,
                1.0, 5.0
            ]
        );
    }

    #[test]
    fn test_polynomial_linear_compression() {
        let vector1 = vec![
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
        ];
        let frame_size = vector1.len();
        let idw_data = polynomial(&vector1, PolynomialType::Polynomial);
        let out = Polynomial::decompress(&idw_data).to_data(frame_size);
        assert_eq!(
            out,
            [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0]
        );
    }

    #[test]
    fn test_to_allowed_error() {
        let vector1 = vec![
            1.0, 1.0, 1.0, 1.0, 2.0, 3.0, 5.0, 1.0, 2.0, 7.0, 1.0, 1.0, 1.0, 3.0, 1.0, 1.0, 5.0,
        ];
        let frame_size = vector1.len();
        let cr = polynomial_allowed_error(&vector1, 0.05, PolynomialType::Polynomial);
        let out = Polynomial::decompress(&cr.compressed_data).to_data(frame_size);
        let e = calculate_error(&vector1, &out);
        assert!(e <= 0.05);
    }

    #[test]
    fn test_idw() {
        let vector1 = vec![1.0, 0.0, 1.0, 1.0, 2.0, 1.0, 1.0, 1.0, 3.0, 1.0, 1.0, 5.0];
        assert_eq!(
            polynomial(&vector1, PolynomialType::Idw),
            [1, 3, 4, 1, 2, 3, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 20, 64, 4]
        );
    }

    #[test]
    fn test_idw_compression() {
        let vector1 = vec![
            1.0, 1.0, 1.0, 1.0, 2.0, 3.0, 5.0, 1.0, 2.0, 7.0, 1.0, 1.0, 1.0, 3.0, 1.0, 1.0, 5.0,
        ];
        let frame_size = vector1.len();
        let idw_data = polynomial(&vector1, PolynomialType::Idw);
        let out = Polynomial::decompress(&idw_data).to_data(frame_size);
        assert_eq!(
            out,
            [
                1.0, 1.13167, 1.62573, 2.32782, 2.83429, 3.0, 2.8335, 2.34163, 1.68979, 1.184, 1.0,
                1.18933, 1.64488, 1.9634, 1.77047, 1.0, 5.0
            ]
        );
    }

    #[test]
    fn test_idw_linear_compression() {
        let vector1 = vec![
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
        ];
        let frame_size = vector1.len();
        let idw_data = polynomial(&vector1, PolynomialType::Idw);
        let out = Polynomial::decompress(&idw_data).to_data(frame_size);
        assert_eq!(
            out,
            [
                1.0, 1.62873, 3.51429, 4.84995, 5.0, 5.40622, 7.05871, 8.64807, 9.0, 9.37719,
                11.18119, 12.0
            ]
        );
    }

    #[test]
    fn test_idw_to_allowed_error() {
        let vector1 = vec![
            1.0, 1.0, 1.0, 1.0, 2.0, 3.0, 5.0, 1.0, 2.0, 7.0, 1.0, 1.0, 1.0, 3.0, 1.0, 1.0, 5.0,
        ];
        let frame_size = vector1.len();
        let cr = polynomial_allowed_error(&vector1, 0.02, PolynomialType::Idw);
        let out = Polynomial::decompress(&cr.compressed_data).to_data(frame_size);
        let e = calculate_error(&vector1, &out);
        assert!(e <= 0.02);
    }

    #[test]
    fn test_line_polynomial() {
        let vector1 = vec![1.0, 1.0, 1.0, 1.0];
        assert_eq!(
            polynomial(&vector1, PolynomialType::Polynomial),
            [0, 3, 0, 0, 0, 0, 0, 0, 0, 240, 63, 0, 0, 0, 0, 0, 0, 240, 63, 1]
        );
    }

    #[test]
    fn test_line_idw() {
        let vector1 = vec![1.0, 1.0, 1.0, 1.0];
        assert_eq!(
            polynomial(&vector1, PolynomialType::Idw),
            [1, 3, 0, 0, 0, 0, 0, 0, 0, 240, 63, 0, 0, 0, 0, 0, 0, 240, 63, 1]
        );
    }
}
