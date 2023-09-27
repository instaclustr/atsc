use log::{info, debug};

const NOOP_COMPRESSOR_ID:u8 = 255;

pub struct Noop {
    pub id: u8,
    data: Vec<f64>,
}

impl Noop {
    pub fn new(frame_size: usize) -> Self {
        debug!("Noop compressor");
        Noop { id: NOOP_COMPRESSOR_ID, data: Vec::with_capacity(frame_size) }
    }

    /// "Compress"
    pub fn compress(&mut self, data: &[f64]) {
        self.data = data.to_vec();
    }

    /// This function transforms the structure in a Binary stream to be appended to the frame
    pub fn to_bytes(self) -> Vec<u8> {
        // Use Bincode and flate2-rs?
        Vec::new()
    }

}

pub fn noop(data: &[f64]) -> Vec<u8> {
    info!("[Compressor] Initializing Noop Compressor");
    let mut c = Noop::new(data.len());
    c.compress(data);
    c.to_bytes()
 }
