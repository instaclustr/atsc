use crate::frame::CompressorFrame;
use crate::header::CompressorHeader;

pub struct CompressedStream {
    header: CompressorHeader,
    data_frames: Vec<CompressorFrame>,
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
