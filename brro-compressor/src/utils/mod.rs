pub mod reader;
pub mod writer;
pub mod error;
mod file_type_detector;

// Is this the right place?
pub fn prev_power_of_two(n: usize) -> usize {
    // n = 0 gives highest_bit_set_idx = 0.
    let highest_bit_set_idx = 63 - (n|1).leading_zeros();
    // Binary AND of highest bit with n is a no-op, except zero gets wiped.
    (1 << highest_bit_set_idx) & n
}