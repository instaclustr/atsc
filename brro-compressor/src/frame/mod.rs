use std::mem::size_of_val;
use bincode::{Decode, Encode};
use crate::compressor::Compressor;

/// This is the structure of a compressor frame
#[derive(Encode, Decode, Debug, Clone)]
pub struct CompressorFrame{
    /// The frame size in bytes,
    frame_size: usize,
    /// The number of samples in this frame,
    samples: u32,
    /// The compressor used in the current frame
    compressor: Compressor,
    /// Output from the compressor
    data: Vec<u8>,
}

impl CompressorFrame {
    ///  Creates a compressor frame, if a compressor is provided, it forces that compressor, otherwise is selected 
    /// by the optimizer
    /// compressor: None to allow BRRO to chose, or force one
    pub fn new(provided_compressor: Option<Compressor>) -> Self {
        CompressorFrame { 
            frame_size: 0,
            samples: 0,
            compressor: provided_compressor.unwrap_or_default(),
            data: Vec::new() }
    }

    /// Calculates the size of the Frame and "closes it"
    // TODO this is probably wrong, so we have to use the write stream to dump the bytes writen
    pub fn close(&mut self) {
        let size = size_of_val(&self.samples)
            + size_of_val(&self.compressor)
            + size_of_val(&self.data)
            + size_of_val(&self.frame_size);
        self.frame_size = size;
    }

    /// Compress a data and stores the result in the frame
    pub fn compress(&mut self, data: &[f64]) {
        // TODO: Optimize here
        // self.compressor = optimizer_selection
        self.data = self.compressor.compress(data);
    }
}