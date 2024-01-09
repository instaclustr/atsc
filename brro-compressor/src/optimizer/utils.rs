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
    // Max value
    pub max: f64,
    // Max value location in the array
    pub max_loc: usize,
    // Min value
    pub min: f64,
    // Min value location in the array
    pub min_loc: usize,
    // Bitdepth that this data can be
    pub bitdepth: Bitdepth,
    pub fractional: bool,
}

impl DataStats {
    pub fn new(data: &[f64]) -> Self {
        let mut min: f64 = data[0];
        let mut min_loc = 0;
        let mut max: f64 = data[0];
        let mut max_loc = 0;
        let mut fractional = false;
        let mut recommended_bitdepth = Bitdepth::F64;
        for (i, value) in data.iter().enumerate() {
            let t_value = *value;
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
        // Check max size of values
        // For very large numbers (i32 and i64), it might be ideal to detect the dc component
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

fn as_i8(value: f64) -> i8 {
    split_n(value).0 as i8
}

fn as_i16(value: f64) -> i16 {
    split_n(value).0 as i16
}

fn as_i32(value: f64) -> i32 {
    split_n(value).0 as i32
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

fn analyze_data(data: &Vec<f64>) -> (i32, i64, bool) {
    let mut min: f64 = 0.0;
    let mut max: f64 = 0.0;
    let mut fractional = false;
    for value in data {
        let t_value = *value;
        if split_n(t_value).1 != 0.0 {
            fractional = true;
        }
        if t_value > max {
            max = t_value
        };
        if t_value < min {
            min = t_value
        };
    }
    // Check max size of values
    // For very large numbers (i32 and i64), it might be ideal to detect the dc component
    // of the signal. And then remove it later
    let max_int = split_n(max).0; // This is the DC component
    let min_int = split_n(min).0;

    // Finding the bitdepth without the DC component
    let recommended_bitdepth = find_bitdepth(max_int - min_int, min_int);
    debug!(
        "Recommended Bitdepth: {}, Fractional: {}",
        recommended_bitdepth, fractional
    );
    (recommended_bitdepth, min_int, fractional)
}

fn find_bitdepth(max_int: i64, min_int: i64) -> i32 {
    // Check where those ints fall into
    let bitdepth = match max_int {
        _ if max_int <= u8::MAX.into() => 8,
        _ if max_int <= i16::MAX.into() => 16,
        _ if max_int <= i32::MAX.into() => 32,
        _ => 64,
    };

    let bitdepth_signed = match min_int {
        _ if min_int == 0 => 8,
        _ if min_int >= i16::MIN.into() => 16,
        _ if min_int >= i32::MIN.into() => 32,
        _ => 64,
    };

    bitdepth.max(bitdepth_signed)
}
