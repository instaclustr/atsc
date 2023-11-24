pub mod error;
pub mod writers;
pub mod readers;

pub const DECIMAL_PRECISION: u32 = 5;

// Is this the right place?
pub fn prev_power_of_two(n: usize) -> usize {
    // n = 0 gives highest_bit_set_idx = 0.
    let highest_bit_set_idx = 63 - (n|1).leading_zeros();
    // Binary AND of highest bit with n is a no-op, except zero gets wiped.
    (1 << highest_bit_set_idx) & n
}

/// Converts a float to u64 with a given precision
pub fn f64_to_u64(number: f64, precision: usize) -> u64 {
    // TODO: Panic on overflow
    if precision > 6 { panic!("Precision only available up to 6 digits!")}
    let mul = [1, 10, 100, 1_000, 10_000, 100_000, 1_000_000][precision];
    (number * mul as f64) as u64
}

pub fn round_f32(x: f32, decimals: u32) -> f64 {
    let y = 10i32.pow(decimals) as f64;
    (x as f64 * y).round() / y    
}

pub fn round_f64(x: f64, decimals: u32) -> f64 {
    let y = 10i32.pow(decimals) as f64;
    (x * y).round() / y

}

pub fn round_and_limit_f64(x: f64, min: f64, max: f64, decimals: u32) -> f64 {
    let y = 10i32.pow(decimals) as f64;
    let out = (x * y).round() / y;
    match out {
        _ if out < min => min,
        _ if out > max => max,
        _ => out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_and_limit_f64() {
        assert_eq!(round_and_limit_f64(3.,2.,4.,1), 3.0);
        assert_eq!(round_and_limit_f64(5.,2.,4.,1), 4.0);
        assert_eq!(round_and_limit_f64(1.,2.,4.,1), 2.0);
        assert_eq!(round_and_limit_f64(3.123452312,2.,4.,3), 3.123);
    }
}