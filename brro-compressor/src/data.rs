use bincode::{Decode, Encode};
use crate::compressor::{Compressor, BinConfig};
use crate::frame::CompressorFrame;
use crate::header::CompressorHeader;

#[derive(Encode, Decode, Debug, Clone)]
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

    /// Compress a chunk of data adding it as a new frame to the current stream
    pub fn compress_chunk(&mut self, chunk: &[f64]) {
        let mut compressor_frame = CompressorFrame::new(None);
        compressor_frame.compress(chunk);
        compressor_frame.close();
        self.data_frames.push(compressor_frame);
    }

    /// Compress a chunk of data with a specific compressor adding it as a new frame to the current stream
    pub fn compress_chunk_with(&mut self, chunk: &[f64], compressor: Compressor) {
        let mut compressor_frame = CompressorFrame::new(Some(compressor));
        compressor_frame.compress(chunk);
        compressor_frame.close();
        self.data_frames.push(compressor_frame);

    }

    /// Transforms the whole CompressedStream into bytes to be written to a file
    pub fn to_bytes(self) -> Vec<u8> {
        let config = BinConfig::get();
        bincode::encode_to_vec(self, config).unwrap()
    }

    /// Gets a binary stream and generates a Compressed Stream
    pub fn from_bytes(data: &[u8]) -> Self {
        let config = BinConfig::get();
        match bincode::decode_from_slice(data, config) {
            Ok((compressed_stream, _)) => compressed_stream,
            Err(e) => panic!("{e}")
        }
    }
}
