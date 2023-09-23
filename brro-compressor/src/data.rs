use crate::compressor::CompressedBlock;
use crate::frame::CompressorFrame; 
use crate::header::CompressorHeader;

pub struct DataFrame {
    frame_header: CompressorFrame,
    data_field: CompressedBlock,
}

pub struct CompressedStream {
    header: CompressorHeader,
    data_frames: Vec<DataFrame>,
}

impl CompressedStream {
    /// Creates an empty compressor stream
    pub fn new() -> Self {
        CompressedStream { 
            header: CompressorHeader::new(),
            data_frames: Vec::new(),
        }
    }
}

impl DataFrame {
    pub fn new() -> Self {
        DataFrame { frame_header: (), data_field: () }
    }
}