use bincode::{Decode, Encode};
use log::debug;
use crate::compressor::{Compressor, BinConfig};
use crate::frame::CompressorFrame;
use crate::header::CompressorHeader;
use crate::frame::frames_manager::FramesManager;

#[derive(Encode, Decode, Debug, Clone)]
pub struct CompressedStream {
    header: CompressorHeader,
    frame_manager: FramesManager,
}

impl CompressedStream {
    /// Creates an empty compressor stream
    pub fn new() -> Self {
        CompressedStream {
            header: CompressorHeader::new(),
            frame_manager: FramesManager::new(),
        }
    }

    /// Compress a chunk of data adding it as a new frame to the current stream
    pub fn compress_chunk(&mut self, chunk: &[f64]) {
        let mut compressor_frame = CompressorFrame::new(None);
        compressor_frame.compress(chunk);
        compressor_frame.close();
        self.frame_manager.add_frame(compressor_frame);
    }

    /// Compress a chunk of data with a specific compressor adding it as a new frame to the current stream
    pub fn compress_chunk_with(&mut self, chunk: &[f64], compressor: Compressor) {
        let mut compressor_frame = CompressorFrame::new(Some(compressor));
        compressor_frame.compress(chunk);
        compressor_frame.close();
        self.frame_manager.add_frame(compressor_frame);
    }

    /// Compress a chunk of data with a specific compressor adding it as a new frame to the current stream
    pub fn compress_chunk_bounded_with(&mut self, chunk: &[f64], compressor: Compressor, max_error: f32) {
        debug!("Compressing chunk bounded with a max error of {}", max_error);
        let mut compressor_frame = CompressorFrame::new(Some(compressor));
        compressor_frame.compress_bounded(chunk, max_error);
        compressor_frame.close();
        self.frame_manager.add_frame(compressor_frame);
    }

    /// Transforms the whole CompressedStream into bytes to be written to a file
    pub fn to_bytes(self) -> Vec<u8> {
        // Will this chain encode??
        let config = BinConfig::get();
        bincode::encode_to_vec(self, config).unwrap()
    }

    /// Gets a binary stream and generates a Compressed Stream, at this point, anything inside the stream is
    /// still in the compressed state
    pub fn from_bytes(data: &[u8]) -> Self {
        let config = BinConfig::get();
        let (compressed_stream, _) = bincode::decode_from_slice(data, config).unwrap();
        compressed_stream
    }

    /// Decompresses all the frames and returns a vector with the data
    pub fn decompress(&self) -> Vec<f64> {
        self.frame_manager.get_all_frames().iter()
            .flat_map(|f| f.decompress())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use super::*;

    #[test]
    fn test_compress_chunk() {
        let vector1 = vec![1.0, 1.0, 1.0, 1.0, 1.0];
        let mut cs = CompressedStream::new();
        cs.compress_chunk(&vector1);
        assert_eq!(cs.frame_manager.get_all_frames().len(), 1);
    }

    #[test]
    fn test_compress_chunk_with() {
        let vector1 = vec![1.0, 1.0, 1.0, 1.0, 1.0];
        let mut cs = CompressedStream::new();
        cs.compress_chunk_with(&vector1, Compressor::Constant);
        assert_eq!(cs.frame_manager.get_all_frames().len(), 1);
    }

    #[test]
    fn test_to_bytes() {
        let vector1 = vec![1.0; 1024];
        let mut cs = CompressedStream::new();
        cs.compress_chunk_with(&vector1, Compressor::Constant);
        let b = cs.to_bytes();
        assert_eq!(b, [66, 82, 82, 79, 0, 1, 41, 251, 0, 4, 3, 3, 0, 2, 0, 1, 253, 184, 80, 252, 43, 206, 253, 150, 101, 41, 251, 0, 4, 3, 3, 0, 2, 0]);
    }

    #[test]
    fn test_from_bytes() {
        let vector1 = vec![1.0; 1024];
        let mut cs = CompressedStream::new();
        cs.compress_chunk_with(&vector1, Compressor::Constant);
        let len = cs.frame_manager.get_all_frames().len();
        let b = cs.to_bytes();
        let cs2 = CompressedStream::from_bytes(&b);
        assert_eq!(len, cs2.frame_manager.get_all_frames().len());
    }

    #[test]
    fn test_constant_decompression() {
        let vector1 = vec![1.0; 1024];
        let mut cs = CompressedStream::new();
        cs.compress_chunk_with(&vector1, Compressor::Constant);
        let b = cs.to_bytes();
        let cs2 = CompressedStream::from_bytes(&b);
        let out = cs2.decompress();
        assert_eq!(vector1, out);
    }

    #[test]
    fn test_constant_decompression_many_different_chunks() {
        let vector1 = vec![1.0; 1024];
        let vector2 = vec![5.0; 1024];
        let mut cs = CompressedStream::new();
        cs.compress_chunk_with(&vector1, Compressor::Constant);
        cs.compress_chunk_with(&vector2, Compressor::Constant);
        let b = cs.to_bytes();
        let cs2 = CompressedStream::from_bytes(&b);
        let out = cs2.decompress();
        assert_eq!([vector1, vector2].concat(), out);
    }

    #[test]
    fn test_many_same_chunks() {
        let vector1 = vec![1.0; 1];
        let vector2 = vec![1.0; 1];
        let mut cs = CompressedStream::new();
        cs.compress_chunk_with(&vector1, Compressor::Constant);
        cs.compress_chunk_with(&vector2, Compressor::Constant);
        assert_eq!(cs.frame_manager.get_all_frames().len(), 2);
    }

    #[test]
    fn test_constant_decompression_many_same_chunks() {
        let vector1 = vec![1.0; 1024];
        let vector2 = vec![1.0; 1024];
        let mut cs = CompressedStream::new();
        cs.compress_chunk_with(&vector1, Compressor::Constant);
        cs.compress_chunk_with(&vector2, Compressor::Constant);
        let b = cs.to_bytes();
        let cs2 = CompressedStream::from_bytes(&b);
        let out = cs2.decompress();
        assert_eq!([vector1, vector2].concat(), out);
    }
    #[test]
    fn ref_check_same_chunks() {
        let vector1 = vec![1.0; 1024];
        let vector2 = vec![1.0; 1024];
        let mut cs = CompressedStream::new();
        cs.compress_chunk_with(&vector1, Compressor::Constant);
        cs.compress_chunk_with(&vector2, Compressor::Constant);

        assert!(Arc::ptr_eq(&cs.frame_manager.get_all_frames()[0], &cs.frame_manager.get_all_frames()[1]));
    }
    #[test]
    fn ref_check_different_chunks() {
        let vector1 = vec![1.0; 1024];
        let vector2 = vec![5.0; 1024];
        let mut cs = CompressedStream::new();
        cs.compress_chunk_with(&vector1, Compressor::Constant);
        cs.compress_chunk_with(&vector2, Compressor::Constant);

        assert!(!Arc::ptr_eq(&cs.frame_manager.get_all_frames()[0], &cs.frame_manager.get_all_frames()[1]));
    }
}