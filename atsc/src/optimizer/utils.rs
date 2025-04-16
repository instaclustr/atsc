/*
Copyright 2024 NetApp, Inc.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use bincode::{Decode, Encode};
use log::debug;

#[derive(PartialEq, Debug, Clone, Copy, Encode, Decode)]
pub enum Bitdepth {
    F64,
    I32,
    I16,
    U8,
}
/// Data structure that holds statictical information about the data provided
pub struct DataStats {
    pub max: f64,
    pub max_loc: usize,
    pub min: f64,
    pub min_loc: usize,
    pub mean: f64,
    pub bitdepth: Bitdepth,
    pub fractional: bool,
}

impl DataStats {
    pub fn new(data: &[f64]) -> Self {
        // Statistical data stored
        let mut min: f64 = data[0];
        let mut min_loc = 0;
        let mut max: f64 = data[0];
        let mut max_loc = 0;
        let mut fractional = false;
        let mut mean: f64 = 0.0;
        let mut recommended_bitdepth = Bitdepth::F64;

        // Walk the data and perform the analysis
        for (i, value) in data.iter().enumerate() {
            let t_value = *value;
            mean += value;
            if split_n(t_value).1 != 0.0 {
                fractional = true;
            }
            if t_value > max {
                max = t_value;
                max_loc = i;
            };
            if t_value < min {
                min = t_value;
                min_loc = i;
            };
        }
        mean /= data.len() as f64;
        // Check max size of values
        // TODO: for very large numbers (i32 and i64), it might be ideal to detect the dc component
        // of the signal. And then remove it later
        let max_int = split_n(max).0; // This is the DC component
        let min_int = split_n(min).0;

        // Finding the bitdepth without the DC component
        if !fractional {
            recommended_bitdepth = DataStats::bitdepth(max_int, min_int);
        }
        debug!(
            "Recommended Bitdepth: {:?}, Fractional: {}",
            recommended_bitdepth, fractional
        );
        DataStats {
            max,
            max_loc,
            min,
            min_loc,
            mean,
            bitdepth: recommended_bitdepth,
            fractional,
        }
    }

    fn bitdepth(max_int: i64, min_int: i64) -> Bitdepth {
        // Check where those ints fall into
        let bitdepth = match max_int {
            _ if max_int <= u8::MAX.into() => 8,
            _ if max_int <= i16::MAX.into() => 16,
            _ if max_int <= i32::MAX.into() => 32,
            _ => 64,
        };

        let bitdepth_signed = match min_int {
            _ if min_int >= 0 && min_int <= u8::MAX.into() => 8,
            _ if min_int >= i16::MIN.into() => 16,
            _ if min_int >= i32::MIN.into() => 32,
            _ => 64,
        };

        match bitdepth.max(bitdepth_signed) {
            8 => Bitdepth::U8,
            16 => Bitdepth::I16,
            32 => Bitdepth::I32,
            _ => Bitdepth::F64,
        }
    }
}
fn split_n(x: f64) -> (i64, f64) {
    const FRACT_SCALE: f64 = 1.0 / (65536.0 * 65536.0 * 65536.0 * 65536.0); // 1_f64.exp(-64)
    const STORED_MANTISSA_DIGITS: u32 = f64::MANTISSA_DIGITS - 1;
    const STORED_MANTISSA_MASK: u64 = (1 << STORED_MANTISSA_DIGITS) - 1;
    const MANTISSA_MSB: u64 = 1 << STORED_MANTISSA_DIGITS;

    const EXPONENT_BITS: u32 = 64 - 1 - STORED_MANTISSA_DIGITS;
    const EXPONENT_MASK: u32 = (1 << EXPONENT_BITS) - 1;
    const EXPONENT_BIAS: i32 = (1 << (EXPONENT_BITS - 1)) - 1;

    let bits = x.to_bits();
    let is_negative = (bits as i64) < 0;
    let exponent = ((bits >> STORED_MANTISSA_DIGITS) as u32 & EXPONENT_MASK) as i32;

    let mantissa = (bits & STORED_MANTISSA_MASK) | MANTISSA_MSB;
    let mantissa = if is_negative {
        -(mantissa as i64)
    } else {
        mantissa as i64
    };

    let shl = exponent + (64 - f64::MANTISSA_DIGITS as i32 - EXPONENT_BIAS + 1);
    if shl <= 0 {
        let shr = -shl;
        if shr < 64 {
            // x >> 0..64
            let fraction = ((mantissa as u64) >> shr) as f64 * FRACT_SCALE;
            (0, fraction)
        } else {
            // x >> 64..
            (0, 0.0)
        }
    } else if shl < 64 {
        // x << 1..64
        let int = mantissa >> (64 - shl);
        let fraction = ((mantissa as u64) << shl) as f64 * FRACT_SCALE;
        (int, fraction)
    } else if shl < 128 {
        // x << 64..128
        let int = mantissa << (shl - 64);
        (int, 0.0)
    } else {
        // x << 128..
        (0, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_linear() {
        let data = vec![1.0, 1.0, 1.0];
        let stats = DataStats::new(&data);
        assert_eq!(stats.bitdepth, Bitdepth::U8);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 1.0);
        assert_eq!(stats.mean, 1.0);
        assert_eq!(stats.min_loc, 0);
        assert_eq!(stats.max_loc, 0);
        assert!(!stats.fractional);
    }

    #[test]
    fn test_stats_non_linear() {
        let data = vec![1.0, 4.0, 7.0];
        let stats = DataStats::new(&data);
        assert_eq!(stats.bitdepth, Bitdepth::U8);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 7.0);
        assert_eq!(stats.mean, 4.0);
        assert_eq!(stats.min_loc, 0);
        assert_eq!(stats.max_loc, 2);
        assert!(!stats.fractional);
    }

    #[test]
    fn test_stats_fract_non_linear() {
        let data = vec![1.5, 4.5, 9.0];
        let stats = DataStats::new(&data);
        assert_eq!(stats.bitdepth, Bitdepth::F64);
        assert_eq!(stats.min, 1.5);
        assert_eq!(stats.max, 9.0);
        assert_eq!(stats.mean, 5.0);
        assert_eq!(stats.min_loc, 0);
        assert_eq!(stats.max_loc, 2);
        assert!(stats.fractional);
    }
}
