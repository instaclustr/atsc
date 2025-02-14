/*
Copyright 2024 NetApp, Inc.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use crate::compressor::{BinConfig, Compressor};
use crate::frame::CompressorFrame;
use crate::header::CompressorHeader;
//use bincode::{Decode, Encode};
use log::debug;

#[derive(Debug, Clone)]
pub struct CompressedStream {
    pub header: CompressorHeader,
    data_frames: Vec<CompressorFrame>,
}

impl CompressedStream {
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
        self.header.add_frame();
    }

    /// Compress a chunk of data with a specific compressor adding it as a new frame to the current stream
    pub fn compress_chunk_with(&mut self, chunk: &[f64], compressor: Compressor) {
        let mut compressor_frame = CompressorFrame::new(Some(compressor));
        compressor_frame.compress(chunk);
        compressor_frame.close();
        self.data_frames.push(compressor_frame);
        self.header.add_frame();
    }

    /// Compress a chunk of data with a specific compressor adding it as a new frame to the current stream
    pub fn compress_chunk_bounded_with(
        &mut self,
        chunk: &[f64],
        compressor: Compressor,
        max_error: f32,
        compression_speed: usize,
    ) {
        debug!(
            "Compressing chunk bounded with a max error of {}",
            max_error
        );
        let mut compressor_frame = CompressorFrame::new(Some(compressor));
        match compressor {
            // Auto means the frame will pick the best
            Compressor::Auto => compressor_frame.compress_best(chunk, max_error, compression_speed),
            _ => compressor_frame.compress_bounded(chunk, max_error),
        }
        compressor_frame.close();
        self.data_frames.push(compressor_frame);
        self.header.add_frame();
    }

    /// Transforms the whole CompressedStream into bytes to be written to a file
    pub fn to_bytes(self) -> Vec<u8> {
        let mut out = Vec::new();
        let config = BinConfig::get();
        out.extend_from_slice(&self.header.to_bytes());
        out.extend_from_slice(&bincode::encode_to_vec(self.data_frames, config).unwrap());
        out
    }

    /// Gets a binary stream and generates a Compressed Stream, at this point, anything inside the stream is
    /// still in the compressed state
    pub fn from_bytes(data: &[u8]) -> Self {
        let config = BinConfig::get();
        // Split the binary data into header and data
        let (binary_header, binary_data) = data.split_at(9);
        let header = CompressorHeader::from_bytes(
            binary_header
                .try_into()
                .expect("Header with incorrect length!"),
        );
        let (data_frames, _) = bincode::decode_from_slice(binary_data, config).unwrap();
        CompressedStream {
            header,
            data_frames,
        }
    }
    pub fn decompress(&self) -> Vec<f64> {
        self.data_frames
            .iter()
            .flat_map(|f| f.decompress())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_chunk() {
        let vector1 = vec![1.0, 1.0, 1.0, 1.0, 1.0];
        let mut cs = CompressedStream::new();
        cs.compress_chunk(&vector1);
        assert_eq!(cs.data_frames.len(), 1);
    }

    #[test]
    fn test_compress_chunk_with() {
        let vector1 = vec![1.0, 1.0, 1.0, 1.0, 1.0];
        let mut cs = CompressedStream::new();
        cs.compress_chunk_with(&vector1, Compressor::Constant);
        assert_eq!(cs.data_frames.len(), 1);
    }

    #[test]
    fn test_frame_count() {
        let vector1 = vec![1.0; 1024];
        let mut cs = CompressedStream::new();
        cs.compress_chunk_with(&vector1, Compressor::Constant);
        println!("{:?}", cs.header.get_frame_count());
        assert_eq!(
            usize::from(cs.header.get_frame_count()),
            cs.data_frames.len()
        );
    }

    #[test]
    fn test_to_bytes() {
        let vector1 = vec![1.0; 1024];
        let mut cs = CompressedStream::new();
        cs.compress_chunk_with(&vector1, Compressor::Constant);
        let b = cs.to_bytes();
        assert_eq!(
            b,
            [66, 82, 82, 79, 1, 0, 0, 0, 1, 1, 41, 251, 0, 4, 3, 3, 30, 3, 1]
        );
    }

    #[test]
    fn test_from_bytes() {
        let vector1 = vec![1.0; 1024];
        let mut cs = CompressedStream::new();
        cs.compress_chunk_with(&vector1, Compressor::Constant);
        let len = cs.data_frames.len();
        let b = cs.to_bytes();
        let cs2 = CompressedStream::from_bytes(&b);
        assert_eq!(len, cs2.data_frames.len());
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
}
