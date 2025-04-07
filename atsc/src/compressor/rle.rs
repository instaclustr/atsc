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
pub struct IndexRLE {
    /// Compressor ID
    pub id: u8,
    /// Index Based RLE. f64 is the value of the index, Vec<usize> is a vector where each value is the index of the value in the data
    /// A series of [1,1,1,1,2,1,1,1,1] would have 1 as a value with indexes at 0 and 5.
    pub rle: Vec<(f64, Vec<usize>)>,
    bitdepth: Bitdepth,
}

impl Encode for IndexRLE {
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

impl Decode for IndexRLE {
    fn decode<__D: ::bincode::de::Decoder>(
        decoder: &mut __D,
    ) -> Result<Self, ::bincode::error::DecodeError> {
        let id = Decode::decode(decoder)?;
        let bitdepth = Decode::decode(decoder)?;
        let rle: Vec<(f64, Vec<usize>)> = match bitdepth {
            Bitdepth::U8 => {
                debug!("Decoding as u8");
                let rle_u8: Vec<(u8, Vec<usize>)> = Decode::decode(decoder)?;
                rle_u8
                    .into_iter()
                    .map(|(value, indices)| (value as f64, indices))
                    .collect()
            }
            Bitdepth::I16 => {
                debug!("Decoding as i16");
                let rle_i16: Vec<(i16, Vec<usize>)> = Decode::decode(decoder)?;
                rle_i16
                    .into_iter()
                    .map(|(value, indices)| (value as f64, indices))
                    .collect()
            }
            Bitdepth::I32 => {
                debug!("Decoding as i32");
                let rle_i32: Vec<(i32, Vec<usize>)> = Decode::decode(decoder)?;
                rle_i32
                    .into_iter()
                    .map(|(value, indices)| (value as f64, indices))
                    .collect()
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

impl IndexRLE {
    // Helper functions to convert the RLE data to different types
    fn rle_as_u8(&self) -> Vec<(u8, Vec<usize>)> {
        let mut result: Vec<(u8, Vec<usize>)> = Vec::with_capacity(self.rle.len());
        for (value, indices) in &self.rle {
            result.push((*value as u8, indices.clone()));
        }
        result
    }

    fn rle_as_i16(&self) -> Vec<(i16, Vec<usize>)> {
        let mut result: Vec<(i16, Vec<usize>)> = Vec::with_capacity(self.rle.len());
        for (value, indices) in &self.rle {
            result.push((*value as i16, indices.clone()));
        }
        result
    }

    fn rle_as_i32(&self) -> Vec<(i32, Vec<usize>)> {
        let mut result: Vec<(i32, Vec<usize>)> = Vec::with_capacity(self.rle.len());
        for (value, indices) in &self.rle {
            result.push((*value as i32, indices.clone()));
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

            if i + 1 >= len || data[i + 1] != value {
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
        IndexRLE {
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
    let c = IndexRLE::new(data, stats.bitdepth);
    CompressorResult::new(c.to_bytes(), 0.0)
}

pub fn rle_to_data(sample_number: usize, compressed_data: &[u8]) -> Vec<f64> {
    let c = IndexRLE::decompress(compressed_data);
    c.to_data(sample_number)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_roundtrip(raw_data: &[f64], encoded: &[u8]) {
        let stats = DataStats::new(raw_data);
        let rle = IndexRLE::new(raw_data, stats.bitdepth);
        assert_eq!(rle.to_bytes(), encoded);

        let decoded = rle_to_data(raw_data.len(), encoded);
        assert_eq!(raw_data, decoded);
    }

    #[test]
    fn test_index_rle_beats_regular_rle() {
        // This test demonstrates a case where index-based RLE is more efficient than regular RLE.
        // Regular RLE would encode this as (value, count) pairs, which can be less efficient
        // when the data has sparse repetitions with large gaps between them.
        //

        let vector1 = vec![
            1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        let stats = DataStats::new(&vector1);
        let rle = IndexRLE::new(&vector1, stats.bitdepth);

        // Verify the encoded bytes are smaller than what regular RLE would produce.
        let encoded = rle.to_bytes();
        // RLE would do 7 pairs of (value, count) for vector1
        // Size: 7 runs * 2 bytes per run (with value and index encoded as u8) = 14 bytes
        let regular_rle_size = 16; // Regular RLE + 2 bytes from the Header
        assert!(encoded.len() < regular_rle_size);

        // Verify roundtrip correctness.
        let decoded = rle_to_data(vector1.len(), &encoded);
        assert_eq!(vector1, decoded);
    }

    #[test]
    fn test_for_constant() {
        assert_roundtrip(&[1.0; 512], &[60, 3, 1, 1, 1, 0]);
    }

    #[test]
    fn test_simple_validator() {
        assert_roundtrip(
            &[
                1.0, 2.0, 2.0, 3.0, 3.0, 3.0, 4.0, 4.0, 4.0, 4.0, 5.0, 5.0, 5.0, 5.0, 5.0,
            ],
            &[60, 3, 5, 1, 1, 0, 2, 1, 1, 3, 1, 3, 4, 1, 6, 5, 1, 10],
        );
    }

    #[test]
    fn test_rle_u8() {
        assert_roundtrip(
            &[
                1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 3.0, 3.0,
                3.0, 3.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
            ],
            &[60, 3, 3, 1, 3, 0, 6, 18, 2, 2, 4, 10, 3, 1, 12],
        );
    }

    #[test]
    fn test_rle_f64() {
        assert_roundtrip(
            &[1.23456, 1.23456, 1.23456, 1.23456, 1.23456],
            &[60, 0, 1, 56, 50, 143, 252, 193, 192, 243, 63, 1, 0],
        );
    }

    #[test]
    fn test_compression() {
        let vector1 = vec![
            1.0, 2.0, 2.0, 3.0, 3.0, 3.0, 4.0, 4.0, 4.0, 4.0, 5.0, 5.0, 5.0, 5.0, 5.0,
        ];
        let stats = DataStats::new(&vector1);
        let c = IndexRLE::new(&vector1, stats.bitdepth).to_bytes();
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
        let c = IndexRLE::new(&vector1, stats.bitdepth).to_bytes();
        let c2 = rle_to_data(vector1.len(), &c);
        assert_eq!(vector1, c2);
    }
}
