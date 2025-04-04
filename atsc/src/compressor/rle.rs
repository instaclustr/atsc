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
use log::{debug, trace};
use std::collections::BTreeMap;

const RLE_COMPRESSOR_ID: u8 = 60;

/// Run Length Encoding compressor
#[derive(PartialEq, Debug, Clone)]
pub struct RLE {
    pub id: u8,
    pub rle: Vec<(f64, Vec<usize>)>,
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
                Encode::encode(&self.rle_as_u8(), encoder)?;
            }
            Bitdepth::I16 => {
                debug!("Encoding as i16");
                Encode::encode(&self.rle_as_i16(), encoder)?;
            }
            Bitdepth::I32 => {
                debug!("Encoding as i32");
                Encode::encode(&self.rle_as_i32(), encoder)?;
            }
            Bitdepth::F64 => {
                debug!("Encoding as f64");
                Encode::encode(&self.rle, encoder)?;
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
        let rle: Vec<(f64, Vec<usize>)> = match bitdepth {
            Bitdepth::U8 => {
                debug!("Decoding as u8");
                let rle_u8: Vec<(u8, Vec<usize>)> = Decode::decode(decoder)?;
                RLE::rle_u8_into(rle_u8)
            }
            Bitdepth::I16 => {
                debug!("Decoding as i16");
                let rle_i16: Vec<(i16, Vec<usize>)> = Decode::decode(decoder)?;
                RLE::rle_i16_into(rle_i16)
            }
            Bitdepth::I32 => {
                debug!("Decoding as i32");
                let rle_i32: Vec<(i32, Vec<usize>)> = Decode::decode(decoder)?;
                RLE::rle_i32_into(rle_i32)
            }
            Bitdepth::F64 => {
                debug!("Decoding as f64");
                let rle_f64: Vec<(f64, Vec<usize>)> = Decode::decode(decoder)?;
                rle_f64
            }
        };

        Ok(Self { id, rle, bitdepth })
    }
}

impl RLE {
    // Helper functions to convert the RLE data to different types
    fn rle_as_u8(&self) -> Vec<(u8, Vec<usize>)> {
        let mut result: Vec<(u8, Vec<usize>)> = Vec::new();
        for (value, indices) in &self.rle {
            result.push((*value as u8, indices.clone()));
        }
        result
    }

    fn rle_as_i16(&self) -> Vec<(i16, Vec<usize>)> {
        let mut result: Vec<(i16, Vec<usize>)> = Vec::new();
        for (value, indices) in &self.rle {
            result.push((*value as i16, indices.clone()));
        }
        result
    }

    fn rle_as_i32(&self) -> Vec<(i32, Vec<usize>)> {
        let mut result: Vec<(i32, Vec<usize>)> = Vec::new();
        for (value, indices) in &self.rle {
            result.push((*value as i32, indices.clone()));
        }
        result
    }

    pub fn rle_u8_into(rle_u8: Vec<(u8, Vec<usize>)>) -> Vec<(f64, Vec<usize>)> {
        let mut result: Vec<(f64, Vec<usize>)> = Vec::new();
        for (value, indices) in &rle_u8 {
            result.push((*value as f64, indices.clone()));
        }
        result
    }

    pub fn rle_i16_into(rle_i16: Vec<(i16, Vec<usize>)>) -> Vec<(f64, Vec<usize>)> {
        let mut result: Vec<(f64, Vec<usize>)> = Vec::new();
        for (value, indices) in &rle_i16 {
            result.push((*value as f64, indices.clone()));
        }
        result
    }

    pub fn rle_i32_into(rle_i32: Vec<(i32, Vec<usize>)>) -> Vec<(f64, Vec<usize>)> {
        let mut result: Vec<(f64, Vec<usize>)> = Vec::new();
        for (value, indices) in &rle_i32 {
            result.push((*value as f64, indices.clone()));
        }
        result
    }

    /// Creates a new instance of the index based RLE compressor with the size needed to handle the worst case
    /// The "trick" here is to use a BTreeMap for faster performance during the encoding process.
    /// and then convert it to a Vec<(f64, Vec<usize>)> for optimal space usage.
    /// NOTE: HashMap is faster, but we need to outcome to be always the same for the same input, which is not guaranteed
    /// with HashMap.
    pub fn new(data: &[f64], bitdepth: Bitdepth) -> Self {
        debug!("RLE compressor");
        // Capacity is 10% of the date lenght in the worst case scenario
        let len = data.len();
        let mut encoded: BTreeMap<u64, Vec<usize>> = BTreeMap::new();
        let mut i = 0;
        // This is where we found the first value of the current sequence
        let mut current_index: usize = 0;

        while i < len {
            let value = data[i];

            if i + 1 < len && data[i + 1] == value {
                i += 1;
                continue;
            } else if i + 1 >= len || data[i + 1] != value {
                trace!("Value Change! Storing value: {}", value);
                // Next value is different so store the current index
                // First check if we have the value in the map
                match encoded.get(&value.to_bits()) {
                    Some(indices) => {
                        trace!("Found value in map: {:?}", indices);
                        // We have the value in the map, so we need to add the current index
                        let mut indices = indices.clone();
                        indices.push(current_index);
                        encoded.insert(value.to_bits(), indices);
                    }
                    None => {
                        trace!("Not found value in map!");
                        // We don't have the value in the map, so we need to create a new entry
                        encoded.insert(value.to_bits(), vec![current_index]);
                    }
                }
                // Now we need to update the current index (it will start on the next value)
                current_index = i + 1;
            }
            i += 1;
        }
        trace!("Encoded: {:?}", encoded);
        // Convert HashMap to Vec<(RLE, Vec<usize>)>
        let mut result: Vec<(f64, Vec<usize>)> = Vec::with_capacity(encoded.len());
        for (value, indices) in encoded {
            result.push((f64::from_bits(value), indices));
        }
        trace!("Vector: {:?}", result);
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
        let mut data: Vec<f64> = vec![0.0; frame_size];

        // Optimize allocation, 1st: Calculate the total number of elements in the flattened vector
        let total_elements: usize = self.rle.iter().map(|(_, indices)| indices.len()).sum();

        // 2nd: Reserve the exact capacity for the flattened vector
        let mut flattened: Vec<(usize, f64)> = Vec::with_capacity(total_elements);

        // Flatten the RLE representation into a vector of (index, value) pairs
        for (value, indices) in &self.rle {
            for &index in indices.iter() {
                flattened.push((index, *value));
            }
        }

        // Sort the flattened vector by index
        flattened.sort_unstable_by_key(|&(index, _)| index);

        // Fill the sequence based on the sorted (index, value) pairs
        for i in 0..flattened.len() {
            let (start_index, value) = flattened[i];
            let end_index = if i + 1 < flattened.len() {
                flattened[i + 1].0
            } else {
                frame_size
            };
            for idx in data.iter_mut().take(end_index).skip(start_index) {
                *idx = value;
            }
        }
        data
    }
}

pub fn rle_compressor(data: &[f64], stats: DataStats) -> CompressorResult {
    debug!("Initializing RLE Compressor. Error and Stats provided");
    let c = RLE::new(data, stats.bitdepth);
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
    fn test_for_constant() {
        let vector1 = vec![1.0; 512];
        let stats = DataStats::new(&vector1);
        let rle = RLE::new(&vector1, stats.bitdepth);
        assert_eq!(rle.to_bytes(), [60, 3, 1, 1, 1, 0]);
    }

    #[test]
    fn test_simple_validator() {
        let vector1 = vec![
            1.0, 2.0, 2.0, 3.0, 3.0, 3.0, 4.0, 4.0, 4.0, 4.0, 5.0, 5.0, 5.0, 5.0, 5.0,
        ];
        let stats = DataStats::new(&vector1);
        let rle = RLE::new(&vector1, stats.bitdepth);
        assert_eq!(
            rle.to_bytes(),
            [60, 3, 5, 1, 1, 0, 2, 1, 1, 3, 1, 3, 4, 1, 6, 5, 1, 10]
        );
    }

    #[test]
    fn test_rle_u8() {
        let vector1 = vec![
            1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 3.0, 3.0, 3.0,
            3.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
        ];
        let stats = DataStats::new(&vector1);
        assert_eq!(
            RLE::new(&vector1, stats.bitdepth).to_bytes(),
            [60, 3, 3, 1, 3, 0, 6, 18, 2, 2, 4, 10, 3, 1, 12]
        );
    }

    #[test]
    fn test_rle_f64() {
        let vector1 = vec![1.23456, 1.23456, 1.23456, 1.23456, 1.23456];
        let stats = DataStats::new(&vector1);
        assert_eq!(
            RLE::new(&vector1, stats.bitdepth).to_bytes(),
            [60, 0, 1, 56, 50, 143, 252, 193, 192, 243, 63, 1, 0]
        );
    }

    #[test]
    fn test_compression() {
        let vector1 = vec![
            1.0, 2.0, 2.0, 3.0, 3.0, 3.0, 4.0, 4.0, 4.0, 4.0, 5.0, 5.0, 5.0, 5.0, 5.0,
        ];
        let stats = DataStats::new(&vector1);
        let c = RLE::new(&vector1, stats.bitdepth).to_bytes();
        let c2 = rle_to_data(vector1.len(), &c);
        assert_eq!(vector1, c2);
    }

    #[test]
    fn test_compression_2() {
        let vector1 = vec![
            1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 3.0, 3.0, 3.0,
            3.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
        ];
        let stats = DataStats::new(&vector1);
        let c = RLE::new(&vector1, stats.bitdepth).to_bytes();
        let c2 = rle_to_data(vector1.len(), &c);
        assert_eq!(vector1, c2);
    }
}
