use log::{info, debug};
use super::BinConfig;
use bincode::{Decode, Encode};

// 250 to optimize bincode encoding, since it checks for <251 for u8
const NOOP_COMPRESSOR_ID:u8 = 250;
#[derive(Encode, Decode, PartialEq, Debug, Clone)]
pub struct Noop {
    pub id: u8,
    pub data: Vec<i64>,
}

impl Noop {
    pub fn new(frame_size: usize) -> Self {
        debug!("Noop compressor");
        Noop { id: NOOP_COMPRESSOR_ID, data: Vec::with_capacity(frame_size) }
    }

    /// Currently the data is provided in f64, this compressor needs i64. So the optimizer needs
    /// to get this out for the compressor
    /// TODO: Make this work decently, right now is only doing a cast (And maybe that is it?)
    pub fn optimize(data: &[f64]) -> Vec<i64> {
        let mut out_vec = Vec::with_capacity(data.len());
        for element in data {
            out_vec.push(*element as i64);
        }
        out_vec
    }

    /// "Compress"
    pub fn compress(&mut self, data: &[f64]) {
        self.data = Noop::optimize(data);
    }

    /// Receives a data stream and generates a Constant
    pub fn decompress(data: &Vec<u8>) -> Self {
        let config = BinConfig::get();
        match bincode::decode_from_slice(&data, config) {
            Ok((constant, _)) => constant,
            Err(e) => panic!("{e}")
        }
    }

    /// This function transforms the structure in a Binary stream to be appended to the frame
    pub fn to_bytes(&self) -> Vec<u8> {
        let config = BinConfig::get();
        bincode::encode_to_vec(self, config).unwrap()
    }

}

pub fn noop(data: &[f64]) -> Vec<u8> {
    info!("[Compressor] Initializing Noop Compressor");
    let mut c = Noop::new(data.len());
    c.compress(data);
    c.to_bytes()
 }

 #[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noop() {
        let vector1 = vec![1.0, 1.0, 1.0, 1.0, 1.0];
        assert_eq!(noop(&vector1), [250, 5, 2, 2, 2, 2, 2]);
    }
    
    #[test]
    fn test_compression() {
        let vector1 = vec![1.0, 1.0, 1.0, 1.0, 1.0];
        let mut c = Noop::new(vector1.len());
        c.compress(&vector1);
        let bin_data = c.to_bytes();
        let c2 = Noop::decompress(&bin_data);

        assert_eq!(c.clone(), c2);
    }
}