use crate::compressor::{self, Compressor};

/// This is the structure of a compressor frame
pub struct CompressorFrame{
    /// The frame size in Bytes,
    frame_size: i64,
    /// The maximum numeric value of the points in the frame
    max_value: i64,  
    /// The minimum numeric value of the points in the frame
    min_value: i64,  
    /// The compressor used in the current frame
    compressor: Compressor,
}