use crate::compressor::Compressor;

/// This is the structure of a compressor frame
pub struct CompressorFrame{
    /// The frame size in Bytes,
    frame_size: i64,
    /// The compressor used in the current frame
    compressor: Compressor,
    /// Output from the compressor
    data: Vec<u8>,
}

impl CompressorFrame {
    /// For testing
    pub fn new() -> Self {
        CompressorFrame { 
            frame_size: 0,
            compressor: Compressor::Noop,
            data: Vec::new() }
    }
}