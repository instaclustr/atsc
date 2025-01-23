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

use crate::compressor::CompressorResult;

use super::BinConfig;
use bincode::{Decode, Encode};
use lib_vsri::vsri::Vsri;
use log::debug;

const VSRI_COMPRESSOR_ID: u8 = 249;

/// Compressor frame for static data, stores the value and nothing else.
#[derive(Debug, Clone, Encode, Decode)]
pub struct VSRI {
    pub id: u8,
    pub vsri: Vsri,
}

impl VSRI {
    /// Creates a new instance of the Constant compressor with the size needed to handle the worst case
    pub fn new() -> Self {
        debug!("Constant compressor");
        VSRI {
            id: VSRI_COMPRESSOR_ID,
            vsri: Vsri::new("placeholder"),
        }
    }

    /// Receives a data stream and generates a Constant
    pub fn decompress(data: &[u8]) -> Self {
        let config = BinConfig::get();
        let (ct, _) = bincode::decode_from_slice(data, config).unwrap();
        ct
    }

    /// This function transforms the structure into a Binary stream
    pub fn to_bytes(&self) -> Vec<u8> {
        // Use Bincode and flate2-rs? Do this at the Stream Level?
        let config = BinConfig::get();
        bincode::encode_to_vec(self, config).unwrap()
    }

    /// Returns an array of data. It creates an array of data the size of the frame with a constant value
    /// and pushes the residuals to the right place.
    pub fn to_data(&self) -> Vec<i32> {
        self.vsri.get_all_timestamps()
    }
}

pub fn vsri_compressor(data: &[i32]) -> CompressorResult {
    debug!("Initializing VSRI Compressor. Error and Stats provided");
    // Initialize the compressor
    let mut c = VSRI::new();
    for ts in data {
        c.vsri.update_for_point(*ts).unwrap();
    }
    // Convert to bytes
    CompressorResult::new(c.to_bytes(), 0.0)
}

pub fn vsri_compressor_bytes(data: &[i32]) -> Vec<u8> {
    debug!("Initializing VSRI Compressor. Error and Stats provided");
    // Initialize the compressor
    let mut c = VSRI::new();
    for ts in data {
        c.vsri.update_for_point(*ts).unwrap();
    }
    // Convert to bytes
    c.to_bytes()
}

pub fn vsri_to_data(compressed_data: &[u8]) -> Vec<i32> {
    let c = VSRI::decompress(compressed_data);
    c.to_data()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vsri_simple() {
        let timestamps = vec![
            1729606100, 1729606120, 1729606140, 1729606160, 1729606180, 1729606200, 1729606220,
            1729606240, 1729606260,
        ];
        let vsri = vsri_compressor(&timestamps);
        let out = vsri_to_data(&vsri.compressed_data);
        assert_eq!(timestamps, out);
    }

    #[test]
    fn test_vsri_several_segments() {
        let timestamps = vec![
            1729606100, 1729606120, 1729606140, 1729606160, 1729606180, 1729606200, 1729606220,
            1729606260, 1729606360, 1729606460, 1729606560, 1729606660, 1729606760, 1729606860,
            1729606881, 1729606882, 1729606883, 1729606884, 1729606885, 1729606886, 1729606887,
        ];
        let vsri = vsri_compressor(&timestamps);
        let out = vsri_to_data(&vsri.compressed_data);
        assert_eq!(timestamps, out);
    }
}
