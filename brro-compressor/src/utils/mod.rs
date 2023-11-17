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
    let out = (x as f64 * y).round() / y;
    out
}

pub fn round_f64(x: f64, decimals: u32) -> f64 {
    let y = 10i32.pow(decimals) as f64;
    let out = (x * y).round() / y;
    out
}