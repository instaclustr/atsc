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
    compressor::Compressor,
    types,
    utils::{f64_to_u64, prev_power_of_two},
};
use log::debug;
use types::metric_tag::MetricTag;

pub mod utils;

/// Max Frame size, this can aprox. 36h of data at 1point/sec rate, a little more than 1 week at 1point/5sec
/// and 1 month (30 days) at 1 point/20sec.
/// This would be aprox. 1MB of Raw data (131072 * 64bits).
/// We wouldn't want to decompressed a ton of uncessary data, but for historical view of the data, looking into 1day/week/month at once is very reasonable
const MAX_FRAME_SIZE: usize = 131072; // 2^17
/// The Min frame size is one that allows our compressors potentially achieve 100x compression. Currently the most
/// limited one is the FFT compressor, that needs 3 frequencies at minimum, 3x100 = 300, next power of 2 is 512.
const MIN_FRAME_SIZE: usize = 512; // 2^9

// My idea here:
// 1. Clean data
// 2. Split into good sized chunks (aka power of 2)
// 3. Get each chunk into the compressor that it should go
// 3.1. Chunks should be at least of a size that it can allow a 100x compression for that given compressor (FFT is 512)
// 4. From the clean data and chunk sizes, assign an optimizer for each chunk
#[derive(Debug, Clone)]
pub struct OptimizerPlan {
    pub data: Vec<f64>,
    pub chunk_sizes: Vec<usize>,
    pub compressors: Vec<Compressor>,
}

impl OptimizerPlan {
    /// Creates an optimal data compression plan
    pub fn plan(data: &[f64]) -> Self {
        let c_data = OptimizerPlan::clean_data(data);
        let chunks = OptimizerPlan::get_chunks_sizes(c_data.len());
        let optimizer = OptimizerPlan::assign_compressor(&c_data, &chunks, None);
        OptimizerPlan {
            data: c_data,
            chunk_sizes: chunks,
            compressors: optimizer,
        }
    }

    /// Creates an optimal plan for compression for the data set provided bound by a given error
    pub fn plan_bounded(data: &[f64], max_error: f32) -> Self {
        // TODO: Check error limits
        let c_data = OptimizerPlan::clean_data(data);
        let chunks = OptimizerPlan::get_chunks_sizes(c_data.len());
        let optimizer = OptimizerPlan::assign_compressor(&c_data, &chunks, Some(max_error));
        OptimizerPlan {
            data: c_data,
            chunk_sizes: chunks,
            compressors: optimizer,
        }
    }

    /// Sets a given compressor for all data chunks
    pub fn set_compressor(&mut self, compressor: Compressor) {
        let new_compressors = vec![compressor; self.compressors.len()];
        self.compressors = new_compressors;
    }

    /// Removes NaN and infinite references from the data
    pub fn clean_data(wav_data: &[f64]) -> Vec<f64> {
        // Cleaning data, removing NaN, etc. This might reduce sample count
        wav_data
            .iter()
            .filter(|x| !(x.is_nan() || x.is_infinite()))
            .copied()
            .collect()
    }

    /// This function gets a length and returns a vector with the chunk sizes to feed to the different compressors
    /// A lot of assumptions go into selecting the chunk size, including:
    /// 1. Collection rate - It is not expected that the collection rate exceeds 1point sec (it is expected actually less)
    /// 2. Maximum compression achievable - A compressed frame as overhead and a minimum number of segments, small frames don't allow great compressions
    /// 3. FFT operates faster under power of 2
    fn get_chunks_sizes(mut len: usize) -> Vec<usize> {
        let mut chunk_sizes = Vec::<usize>::new();
        while len > 0 {
            match len {
                _ if len >= MAX_FRAME_SIZE => {
                    chunk_sizes.push(MAX_FRAME_SIZE);
                    len -= MAX_FRAME_SIZE;
                }
                _ if len <= MIN_FRAME_SIZE => {
                    chunk_sizes.push(len);
                    len = 0;
                }
                _ => {
                    let size = prev_power_of_two(len);
                    chunk_sizes.push(size);
                    len -= size;
                }
            }
        }
        chunk_sizes
    }

    /// Returns a vector with the data slice and the compressor associated
    pub fn get_execution(&self) -> Vec<(&Compressor, &[f64])> {
        let mut output = Vec::with_capacity(self.chunk_sizes.len());
        let mut s = 0;
        for (i, size) in self.chunk_sizes.iter().enumerate() {
            output.push((&self.compressors[i], &self.data[s..(s + *size)]));
            s += *size;
        }
        output
    }

    /// Walks the data, checks how much variability is in the data, and assigns a compressor based on that
    /// NOTE: Is this any good?
    fn get_compressor(data: &[f64]) -> Compressor {
        let _ = data.iter().map(|&f| f64_to_u64(f, 0));
        // For now, let's just return FFT
        Compressor::FFT
    }

    /// Assigns a compressor to a chunk of data
    fn assign_compressor(
        clean_data: &[f64],
        chunks: &[usize],
        max_error: Option<f32>,
    ) -> Vec<Compressor> {
        let mut selection = Vec::with_capacity(chunks.len());
        match max_error {
            Some(_err) => todo!(),
            None => {
                let mut s = 0;
                for size in chunks.iter() {
                    selection.push(OptimizerPlan::get_compressor(&clean_data[s..(s + *size)]));
                    s += *size;
                }
            }
        }
        selection
    }
}

/// This should look at the data and return an optimized dataset for a specific compressor,
/// If a compressor is hand picked, this should be skipped.
pub fn process_data(wav_data: &[f64], tag: &MetricTag) -> Vec<f64> {
    debug!("Tag: {:?} Len: {}", tag, wav_data.len());
    wav_data
        .iter()
        .filter(|x| !(x.is_nan() || x.is_infinite()))
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optimizer() {
        let fake_data = vec![12.23; 2049];
        let op = OptimizerPlan::plan(&fake_data);
        let plan_vec = op.get_execution();
        assert_eq!(plan_vec.len(), 2);
    }

    #[test]
    fn test_get_chunks_sizes() {
        let len_very_large: usize = 131072 * 3 + 1765;
        let len_small: usize = 31;
        let len_right_sized: usize = 2048;
        let len_some_size: usize = 12032;
        assert_eq!(
            OptimizerPlan::get_chunks_sizes(len_very_large),
            [131072, 131072, 131072, 1024, 512, 229]
        );
        assert_eq!(OptimizerPlan::get_chunks_sizes(len_small), [31]);
        assert_eq!(OptimizerPlan::get_chunks_sizes(len_right_sized), [2048]);
        assert_eq!(
            OptimizerPlan::get_chunks_sizes(len_some_size),
            [8192, 2048, 1024, 512, 256]
        );
    }

    #[test]
    fn assign_compressor() {
        let fake_data = vec![12.23; 132671];
        let chunks = OptimizerPlan::get_chunks_sizes(fake_data.len());
        let compressor_vec = OptimizerPlan::assign_compressor(&fake_data, &chunks, None);
        assert_eq!(compressor_vec.len(), 4);
    }
}
