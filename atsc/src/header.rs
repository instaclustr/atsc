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

use log::{debug, trace};
use std::panic;

/*  The current file version.
    On file read, compressors check the version and uncompress accordingly to that (of fail)
*/
const CURRENT_VERSION: u32 = 1;
#[derive(Debug, Clone)]
pub struct CompressorHeader {
    initial_segment: [u8; 4],
    pub version: u32,
    frame_count: u8,
}

fn verify_header_versions(version: u32) {
    let current_version = CURRENT_VERSION;
    trace!("Versions: c:{} h:{}", current_version, version);
    match current_version.cmp(&version) {
        std::cmp::Ordering::Less => panic!(
            "Can't decompress! File is version ({}) is higher than compressor version ({})!",
            version, current_version
        ),
        std::cmp::Ordering::Equal | std::cmp::Ordering::Greater => {
            debug!("File version: {}", version)
        }
    }
}

impl CompressorHeader {
    pub fn new() -> Self {
        CompressorHeader {
            initial_segment: *b"BRRO",
            version: CURRENT_VERSION,
            frame_count: 0,
        }
    }

    pub fn add_frame(&mut self) {
        self.frame_count += 1;
    }

    pub fn get_frame_count(&mut self) -> u8 {
        self.frame_count
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // Converts header to a byte vector, little endian
        let mut vec = Vec::new();
        // Add initial_segment
        vec.extend_from_slice(&self.initial_segment);
        // Add version (u32 as 4 bytes)
        vec.extend_from_slice(&self.version.to_le_bytes());
        // Add frame_count
        vec.push(self.frame_count);

        vec
    }

    pub fn from_bytes(data: [u8; 9]) -> Self {
        // Extract initial_segment
        let initial_segment = [data[0], data[1], data[2], data[3]];
        // Extract version (u32 from 4 bytes)
        let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        // Extract frame_count
        let frame_count = data[8];
        verify_header_versions(version);
        CompressorHeader {
            initial_segment,
            version,
            frame_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::compressor::Compressor;
    use crate::data::CompressedStream;
    use crate::header::CURRENT_VERSION;

    #[test]
    fn test_same_version_or_smaller() {
        let vector1 = vec![1.0; 1024];
        let mut cs = CompressedStream::new();
        cs.compress_chunk_with(&vector1, Compressor::Constant);
        let b = cs.to_bytes();
        // Read the first 8 Bytes, check version on the 5th to 8th byte
        let version_bytes: [u8; 4] = [b[4], b[5], b[6], b[7]];
        let version_number = u32::from_le_bytes(version_bytes);
        assert_eq!(version_number, CURRENT_VERSION);
    }

    #[test]
    #[should_panic(expected = "is higher than compressor version")]
    fn test_higher_version() {
        let vector1 = vec![1.0; 1024];
        let mut cs = CompressedStream::new();
        cs.header.version = 9;
        cs.compress_chunk_with(&vector1, Compressor::Constant);
        let b = cs.to_bytes();
        CompressedStream::from_bytes(&b);
    }
}
