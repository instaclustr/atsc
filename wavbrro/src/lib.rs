pub read;
pub write;

fn split_f64_into_u16(bits: [u16; 4]) -> f64 {
    let u64_bits = (bits[0] as u64) |
        ((bits[1] as u64) << 16) |
        ((bits[2] as u64) << 32) |
        ((bits[3] as u64) << 48);
    
    f64::from_bits(u64_bits)
}

fn split_i64_into_i32(bits: [u32; 2]) -> i64 {
    let u64_bits = (bits[0] as u64) |
        ((bits[1] as u64) << 32);
    
    i64::from_bits(u64_bits)
}

fn join_u16_into_f64(bits: [u16; 4]) -> f64 {
    let u64_bits = (bits[0] as u64) |
        ((bits[1] as u64) << 16) |
        ((bits[2] as u64) << 32) |
        ((bits[3] as u64) << 48);
    
    f64::from_bits(u64_bits)
}

fn join_u32_into_i64(bits: [u32; 2]) -> i64 {
    let u64_bits = (bits[0] as u64) |
        ((bits[1] as u64) << 32);
    
    i64::from_bits(u64_bits)
}