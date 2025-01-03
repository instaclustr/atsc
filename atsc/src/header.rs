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

use std::panic;

use bincode::{Decode, Encode};
use log::{debug, trace};
use version_compare::{compare, Cmp};

#[derive(Encode, Debug, Clone)]
pub struct CompressorHeader {
    initial_segment: [u8; 4],
    pub version: String,
    // We should go unsigned
    frame_count: i16,
}

impl Decode for CompressorHeader {
    fn decode<__D: ::bincode::de::Decoder>(
        decoder: &mut __D,
    ) -> Result<Self, ::bincode::error::DecodeError> {
        let header = Self {
            initial_segment: Decode::decode(decoder)?,
            version: Decode::decode(decoder)?,
            frame_count: Decode::decode(decoder)?,
        };
        let current_version = env!("CARGO_PKG_VERSION").to_string();
        trace!("Versions: c:{} h:{}", current_version, header.version);
        match compare(current_version.clone(), header.version.clone()) {
            Ok(Cmp::Lt) => panic!(
                "Can't decompress! File is version ({}) is higher than compressor version ({})!",
                header.version, current_version
            ),
            Ok(Cmp::Eq | Cmp::Gt) => debug!("File version: {}", header.version),
            _ => panic!("Wrong version number!"),
        }
        Ok(header)
    }
}

impl<'__de> ::bincode::BorrowDecode<'__de> for CompressorHeader {
    fn borrow_decode<__D: ::bincode::de::BorrowDecoder<'__de>>(
        decoder: &mut __D,
    ) -> Result<Self, ::bincode::error::DecodeError> {
        let header = Self {
            initial_segment: bincode::BorrowDecode::borrow_decode(decoder)?,
            version: bincode::BorrowDecode::borrow_decode(decoder)?,
            frame_count: bincode::BorrowDecode::borrow_decode(decoder)?,
        };
        let current_version = env!("CARGO_PKG_VERSION").to_string();
        trace!("Versions: c:{} h:{}", current_version, header.version);
        match compare(current_version.clone(), header.version.clone()) {
            Ok(Cmp::Lt) => panic!(
                "Can't decompress! File is version ({}) is higher than compressor version ({})!",
                header.version, current_version
            ),
            Ok(Cmp::Eq | Cmp::Gt) => debug!("File version: {}", header.version),
            _ => panic!("Wrong version number!"),
        }
        Ok(header)
    }
}

impl CompressorHeader {
    pub fn new() -> Self {
        const VERSION: &str = env!("CARGO_PKG_VERSION");
        CompressorHeader {
            initial_segment: *b"BRRO",
            version: VERSION.to_string(),
            // We have to limit the bytes of the header
            frame_count: 0,
        }
    }

    pub fn add_frame(&mut self) {
        self.frame_count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compressor::Compressor;
    use crate::data::CompressedStream;

    #[test]
    fn test_same_version() {
        let vector1 = vec![1.0; 1024];
        let mut cs = CompressedStream::new();
        cs.compress_chunk_with(&vector1, Compressor::Constant);
        let b = cs.to_bytes();
        let cs2 = CompressedStream::from_bytes(&b);
        assert_eq!(
            compare(env!("CARGO_PKG_VERSION").to_string(), cs2.header.version),
            Ok(Cmp::Eq)
        );
    }

    #[test]
    fn test_smaller_version() {
        let vector1 = vec![1.0; 1024];
        let mut cs = CompressedStream::new();
        cs.header.version = "0.1.0".to_owned();
        cs.compress_chunk_with(&vector1, Compressor::Constant);
        let b = cs.to_bytes();
        let cs2 = CompressedStream::from_bytes(&b);
        assert_eq!(
            compare(env!("CARGO_PKG_VERSION").to_string(), cs2.header.version),
            Ok(Cmp::Gt)
        );
    }

    #[test]
    #[should_panic(expected = "is higher than compressor version")]
    fn test_higher_version() {
        let vector1 = vec![1.0; 1024];
        let mut cs = CompressedStream::new();
        cs.header.version = "9.9.9".to_owned();
        cs.compress_chunk_with(&vector1, Compressor::Constant);
        let b = cs.to_bytes();
        CompressedStream::from_bytes(&b);
    }
}
