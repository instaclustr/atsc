/// This compressor is for static values, it allows some level of non static (1%)
use std::collections::HashMap;
use log::{info, debug};

const CONSTANT_COMPRESSOR_ID:u8 = 0;

/// This is a temporary implementation, other implementations (FFT, Polynomial) might provide the same result
/// as going throught the data anyway.
pub struct Constant {
    pub id: u8,
    pub constant: i64,
    pub error: f64,
    pub residuals: Vec<(i32,i64)>,
}

impl Constant {
    pub fn new() -> Self {
        debug!("Constant compressor");
        Constant { 
             id: CONSTANT_COMPRESSOR_ID,
             constant: 0,
             error: 0.0,
             residuals: Vec::new() }
    }

    /// This compressor is about having a single constant for the whole segment
    pub fn set_constant(&mut self, constant_value: i64) {
        self.constant = constant_value;
    }

    /// Adds an error point to the error point vector.
    /// It needs the sample number (where the error happened in the signal)
    /// and the error value.
    pub fn add_residual(&mut self, error_sample: i32, error_value: i64) {
        self.residuals.push((error_sample, error_value));
    }

    /// Currently the data is provided in f64, this compressor needs i64. So the optimizer needs
    /// to get this out for the compressor
    pub fn optimize(data: &[f64]) -> Vec<i64> {
        let out_vec = Vec::with_capacity(data.len());
        out_vec
    }

    /// Compresses the data. Walks the data array and sets one value as the constant.
    /// TODO: VERY INNEFICIENT, we walk the data, and then the resulting map 2x. Find opportunities to improve
    pub fn compress(&mut self, data: &[i64]) {
        // Map all the entries, the one with most hits, is the constant, the remaining, are residuals
        let mut seen_values = HashMap::new();
        for val in data {
            seen_values.entry(val).and_modify(|e| {*e += 1}).or_insert(1);
        }
        // Walk the map and pick the highest value as the constant, the others as residuals
        let mut constant = 0;
        let mut hit_count = 0;
        for (k,v) in seen_values {
            if v > hit_count { 
                constant = *k;
                hit_count = v;
            }
        }
        self.set_constant(constant);
        // Drain the map and set the residuals
        for (k, v) in seen_values.drain() {
            // Skip the constant value
            if k.eq(&constant) { continue; }
            // TODO: THIS IS WRONG! It needs a position!
            self.add_residual(v, *k);
        }

    }

    /// This function transforms the structure in a Binary stream to be appended to the frame
    pub fn to_bytes(self) -> Vec<u8> {
        // Use Bincode and flate2-rs?
        Vec::new()
    }
}

pub fn constant(data: &[f64]) -> Vec<u8> {
    Vec::new()
 }