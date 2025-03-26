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

use crate::{
    compressor::CompressorResult,
    optimizer::utils::{Bitdepth, DataStats},
};

use super::BinConfig;
use bincode::{Decode, Encode};
use log::debug;
use std::collections::HashMap;

const RLE_COMPRESSOR_ID: u8 = 60;

/// Run Length Encoding compressor
#[derive(PartialEq, Debug, Clone)]
pub struct RLE {
    pub id: u8,
    pub rle: Vec<(f64, Vec<usize>)>,
    // For internal use only
    bitdepth: Bitdepth,
}

impl Encode for RLE {
    fn encode<__E: ::bincode::enc::Encoder>(
        &self,
        encoder: &mut __E,
    ) -> Result<(), ::bincode::error::EncodeError> {
        Encode::encode(&self.id, encoder)?;
        Encode::encode(&self.bitdepth, encoder)?;
        match &self.bitdepth {
            Bitdepth::U8 => {
                debug!("Encoding as u8");
                Encode::encode(&(self.constant as u8), encoder)?;
            }
            Bitdepth::I16 => {
                debug!("Encoding as i16");
                Encode::encode(&(self.constant as i16), encoder)?;
            }
            Bitdepth::I32 => {
                debug!("Encoding as i32");
                Encode::encode(&(self.constant as i32), encoder)?;
            }
            Bitdepth::F64 => {
                debug!("Encoding as f64");
                Encode::encode(&self.constant, encoder)?;
            }
        }
        Ok(())
    }
}

impl Decode for RLE {
    fn decode<__D: ::bincode::de::Decoder>(
        decoder: &mut __D,
    ) -> Result<Self, ::bincode::error::DecodeError> {
        let id = Decode::decode(decoder)?;
        let bitdepth = Decode::decode(decoder)?;
        let constant: f64 = match bitdepth {
            Bitdepth::U8 => {
                debug!("Decoding as u8");
                let const_u8: u8 = Decode::decode(decoder)?;
                const_u8 as f64
            }
            Bitdepth::I16 => {
                debug!("Decoding as i16");
                let const_i16: i16 = Decode::decode(decoder)?;
                const_i16 as f64
            }
            Bitdepth::I32 => {
                debug!("Decoding as i32");
                let const_i32: i32 = Decode::decode(decoder)?;
                const_i32 as f64
            }
            Bitdepth::F64 => {
                debug!("Decoding as f64");
                let const_f64: f64 = Decode::decode(decoder)?;
                const_f64
            }
        };

        Ok(Self {
            id,
            constant,
            bitdepth,
        })
    }
}

impl RLE {
    /// Creates a new instance of the RLE compressor with the size needed to handle the worst case
    /// The "trick" here is to use a Hashmap for faster performance during the encoding process.
    /// and then convert it to a Vec<(f64, Vec<usize>)> for optimal space usage.
    pub fn new(_sample_count: usize, data: &[f64], bitdepth: Bitdepth) -> Self {
        debug!("RLE compressor");
        // Capacity is 10% of the date lenght in the worst case scenario
        let capacity = (data.len() * 0.1) as usize;
        let mut encoded: HashMap<f64, Vec<usize>> = HashMap::with_capacity(capacity);
        let mut i = 0;
        let len = data.len();
    
        while i < len {
            let value = data[i];
            let mut count = 1;
    
            while i + 1 < len && data[i + 1] == value count += 1;
                i += 1;
            }
    
            encoded.entry(value).or_insert_with(Vec::new).push(i - count + 1);
            i += 1;
        }
    
        // Convert HashMap to Vec<(RLE, Vec<usize>)>
        let mut result: Vec<(f64, Vec<usize>)> = Vec::with_capacity(encoded.len());
        for (value, indices) in encoded {
            result.push((value, indices));
        }

        RLE {
            id: RLE_COMPRESSOR_ID,
            rle: result,
            bitdepth,
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
        let config = BinConfig::get();
        bincode::encode_to_vec(self, config).unwrap()
    }

    pub fn to_data(&self, frame_size: usize) -> Vec<f64> {
        let mut data = vec![0; frame_size];
        // Walk the structure and generate the data vec
        for (value, indices) in self.rle {
            for &start_index in indices {
                let mut i = start_index;
                while i < length && data[i] == 0 {
                    data[i] = *value;
                    i += 1;
                }
            }
        }
    }
}

pub fn rle_compressor(data: &[f64], stats: DataStats) -> CompressorResult {
    debug!("Initializing RLE Compressor. Error and Stats provided");
    let c = RLE::new(data, data, stats.bitdepth);
    CompressorResult::new(c.to_bytes(), 0.0)
}

pub fn rle_to_data(sample_number: usize, compressed_data: &[u8]) -> Vec<f64> {
    let c = RLE::decompress(compressed_data);
    c.to_data(sample_number)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rle_u8() {
        let vector1 = vec![1.0, 1.0, 1.0, 1.0,
                            2.0, 2.0,
                            1.0, 1.0, 1.0, 1.0,
                            2.0, 2.0,
                            3.0, 3.0, 3.0, 3.0, 3.0, 3.0,
                            1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let stats = DataStats::new(&vector1);
        assert_eq!(
            RLE::new(vector1.len(), &vector1, stats.bitdepth).to_bytes(),
            [30, 3, 1]
        );
    }

    #[test]
    fn test_rle_f64() {
        let vector1 = vec![1.23456, 1.23456, 1.23456, 1.23456, 1.23456];
        let stats = DataStats::new(&vector1);
        assert_eq!(
            RLE::new(vector1.len(), &vector1, stats.bitdepth).to_bytes(),
            [30, 0, 56, 50, 143, 252, 193, 192, 243, 63]
        );
    }

    #[test]
    fn test_compression() {
        let vector1 = vec![1.0, 1.0, 1.0, 1.0, 1.0];
        let stats = DataStats::new(&vector1);
        let c = RLE::new(vector1.len(), &vector1, stats.bitdepth).to_bytes();
        let c2 = rle_to_data(vector1.len(), &c);

        assert_eq!(vector1, c2);
    }
}
