/// This compressor is for static values, it allows some level of non static (1%)
use log::{info, debug};

const CONSTANT_COMPRESSOR_ID:u8 = 0;

pub struct Constant {
    pub id: u8,
    pub constant: f64,
    pub error: f64,
    pub error_points: Vec<(i32,f64)>,
}

impl Constant {
    pub fn new() -> Self {
        Constant { 
             id: CONSTANT_COMPRESSOR_ID,
             constant: 0.0,
             error: 0.0,
             error_points: Vec::new() }
    }

    /// This compressor is about having a single constant for the whole segment
    pub fn set_constant(&mut self, constant_value: f64) {
        self.constant = constant_value;
    }

    /// Adds an error point to the error point vector.
    /// It needs the sample number (where the error happened in the signal)
    /// and the error value.
    pub fn add_error_point(&mut self, error_sample: i32, error_value: f64) {
        self.error_points.push((error_sample, error_value));
    }

    /// Compresses the data. Walks the data array and sets one value as 
    pub fn compress(&mut self, data: &[f64]) {
        let constant_count = 0;
        let error_count = 0;
        for val in data {
            
        }

    }
}

pub fn constant(data: &[f64]) -> Vec<f64> {
    Vec::new()
 }