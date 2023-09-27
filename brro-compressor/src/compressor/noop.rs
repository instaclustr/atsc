use log::{info, debug};

const NOOP_COMPRESSOR_ID:u8 = 256;

pub struct Noop {
    pub id: u8,
}

impl Noop {
    pub fn new() -> Self {
        debug!("Noop compressor");
        Noop { id: NOOP_COMPRESSOR_ID }
    }

    /// This function transforms the structure in a Binary stream to be appended to the frame
    pub fn to_bytes(self) -> Vec<u8> {
        // Use Bincode and flate2-rs?
        Vec::new()
    }

}

pub fn noop(data: &[f64]) -> Vec<u8> {
    Vec::new()
 }
