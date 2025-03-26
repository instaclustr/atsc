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

const CONSTANT_COMPRESSOR_ID: u8 = 60;

/// Compressor frame for static data, stores the value and nothing else.
#[derive(PartialEq, Debug, Clone)]
pub struct Constant {
    pub id: u8,
    pub constant: f64,
    // For internal use only
    bitdepth: Bitdepth,
}

impl Encode for Constant {
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

impl Decode for Constant {
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
    /// Creates a new instance of the Constant compressor with the size needed to handle the worst case
    pub fn new(_sample_count: usize, constant_value: f64, bitdepth: Bitdepth) -> Self {
        debug!("Constant compressor");
        Constant {
            id: CONSTANT_COMPRESSOR_ID,
            constant: constant_value,
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

    /// Returns an array of data. It creates an array of data the size of the frame with a constant value
    /// and pushes the residuals to the right place.
    pub fn to_data(&self, frame_size: usize) -> Vec<f64> {
        let data = vec![self.constant; frame_size];
        data
    }
}

pub fn rle_compressor(data: &[f64], stats: DataStats) -> CompressorResult {
    debug!("Initializing Constant Compressor. Error and Stats provided");
    let c = Constant::new(data.len(), stats.min, stats.bitdepth);
    CompressorResult::new(c.to_bytes(), 0.0)
}

pub fn rle_to_data(sample_number: usize, compressed_data: &[u8]) -> Vec<f64> {
    let c = Constant::decompress(compressed_data);
    c.to_data(sample_number)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_u8() {
        let vector1 = vec![1.0, 1.0, 1.0, 1.0, 1.0];
        let stats = DataStats::new(&vector1);
        assert_eq!(
            Constant::new(vector1.len(), stats.min, stats.bitdepth).to_bytes(),
            [30, 3, 1]
        );
    }

    #[test]
    fn test_constant_f64() {
        let vector1 = vec![1.23456, 1.23456, 1.23456, 1.23456, 1.23456];
        let stats = DataStats::new(&vector1);
        assert_eq!(
            Constant::new(vector1.len(), stats.min, stats.bitdepth).to_bytes(),
            [30, 0, 56, 50, 143, 252, 193, 192, 243, 63]
        );
    }

    #[test]
    fn test_compression() {
        let vector1 = vec![1.0, 1.0, 1.0, 1.0, 1.0];
        let stats = DataStats::new(&vector1);
        let c = Constant::new(vector1.len(), stats.min, stats.bitdepth).to_bytes();
        let c2 = constant_to_data(vector1.len(), &c);

        assert_eq!(vector1, c2);
    }
}
