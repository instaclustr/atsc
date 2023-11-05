pub mod frames_manager;

use std::mem::size_of_val;
use bincode::{Decode, Encode};
use log::debug;
use crate::compressor::Compressor;

/// This is the structure of a compressor frame
#[derive(Encode, Decode, Debug, Clone, Hash, PartialEq)]
pub struct CompressorFrame{
    /// The frame size in bytes,
    frame_size: usize,
    /// The number of samples in this frame,
    sample_count: usize,
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
            sample_count: 0,
            compressor: provided_compressor.unwrap_or_default(),
            data: Vec::new() }
    }

    /// Calculates the size of the Frame and "closes it"
    // TODO this is probably wrong, so we have to use the write stream to dump the bytes writen
    pub fn close(&mut self) {
        let size = size_of_val(&self.sample_count)
            + size_of_val(&self.compressor)
            + size_of_val(&self.data)
            + size_of_val(&self.frame_size);
        self.frame_size = size;
    }

    /// Compress a data and stores the result in the frame
    pub fn compress(&mut self, data: &[f64]) {
        self.sample_count = data.len();
        self.data = self.compressor.compress(data);
    }

    /// Compress a data and stores the result in the frame
    pub fn compress_bounded(&mut self, data: &[f64], max_error: f32) {
        self.sample_count = data.len();
        self.data = self.compressor.compress_bounded(data, max_error as f64);
    }

    /// Decompresses a frame and returns the resulting data array
    pub fn decompress(&self) -> Vec<f64> {
        debug!("Decompressing Frame. Size: {}, Samples: {}", self.frame_size, self.sample_count);
        self.compressor.decompress(self.sample_count, &self.data)
    }

}