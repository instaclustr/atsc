// Lucas - Once the project is far enough along I strongly reccomend reenabling dead code checks
#![allow(dead_code)]

use median::Filter;
use log::debug;
use types::metric_tag::MetricTag;
use crate::{types, utils::prev_power_of_two, compressor::Compressor};

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
struct OptimizerPlan {
    pub data: Vec<f64>,
    pub chunk_sizes: Vec<usize>,
    pub compressors: Vec<Compressor>,
}

impl OptimizerPlan {
    pub fn create_plan(data: Vec<f64>) -> Self {
        let c_data = OptimizerPlan::clean_data(&data);
        let chunks = OptimizerPlan::get_chunks_sizes(c_data.len());
        let optimizer = OptimizerPlan::assign_compressor(&c_data, &chunks, None);
        OptimizerPlan { data: c_data,
                        chunk_sizes: chunks,
                        compressors: optimizer }
    }

    pub fn create_plan_bounded(data: Vec<f64>, max_error: f32) -> Self {
        let c_data = OptimizerPlan::clean_data(&data);
        let chunks = OptimizerPlan::get_chunks_sizes(c_data.len());
        let optimizer = OptimizerPlan::assign_compressor(&c_data, &chunks, Some(max_error));
        OptimizerPlan { data: c_data,
                        chunk_sizes: chunks,
                        compressors: optimizer }
    }

    /// Removes NaN and infinite references from the data
    pub fn clean_data(wav_data: &Vec<f64>) -> Vec<f64> {
        // Cleaning data, removing NaN, etc. This might reduce sample count
        wav_data.iter()
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
                },
                _ if len <= MIN_FRAME_SIZE => {
                    chunk_sizes.push(len);
                    len = 0;
                },
                _ => {
                    let size = prev_power_of_two(len);
                    chunk_sizes.push(size);
                    len -= size;
                }
            }
        }
        chunk_sizes
    }

    /// Assigns a compressor to a chunk of data
    fn assign_compressor(clean_data: &Vec<f64>, chunks: &Vec<usize>, max_error: Option<f32>) -> Vec<Compressor> {
        let selection = Vec::with_capacity(chunks.len());
        match max_error {
            Some(err) => todo!(),
            None => return selection,
        }
    }

}

impl MetricTag {
    #[allow(clippy::wrong_self_convention)]
    fn from_float(&self, x: f64) -> i64 {
        match self {
            MetricTag::Other => {
                0
            }
            MetricTag::NotFloat | MetricTag::QuasiRandom => {
                x as i64
            }
            MetricTag::Percent(y) => {
                to_multiply_and_truncate(x, *y)
            }
            MetricTag::Duration(y) => {
                to_multiply_and_truncate(x, *y)
            }
            MetricTag::Bytes(y) => {
                (x as i64) / (*y as i64)
            }
        }
    }
}

/// Converts a float via multiplication and truncation
fn to_multiply_and_truncate(number: f64, mul: i32) -> i64 {
    (number * mul as f64) as i64
}

fn to_median_filter(data: &Vec<f64>) -> Vec<i64> {
    let mut filtered = Vec::with_capacity(data.len());
    // 10minutes of data
    let mut filter = Filter::new(50);
    for point in data {
        let point_int = MetricTag::QuasiRandom.from_float(*point);
        let median = filter.consume(point_int);
        filtered.push(median)
    }
    filtered
}

/// This should look at the data and return an optimized dataset for a specific compressor,
/// If a compressor is hand picked, this should be skipped.
pub fn process_data(wav_data: &Vec<f64>, tag: &MetricTag) -> Vec<f64> {
    debug!("Tag: {:?} Len: {}", tag, wav_data.len());
    wav_data.iter()
        .filter(|x| !(x.is_nan() || x.is_infinite()))
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_chunks_sizes() {
        let len_very_large: usize = 131072 * 3 + 1765;
        let len_small: usize = 31;
        let len_right_sized: usize = 2048;
        let len_some_size: usize = 12032;
        assert_eq!(OptimizerPlan::get_chunks_sizes(len_very_large), [131072, 131072, 131072, 1024, 512, 229]);
        assert_eq!(OptimizerPlan::get_chunks_sizes(len_small), [31]);
        assert_eq!(OptimizerPlan::get_chunks_sizes(len_right_sized), [2048]);
        assert_eq!(OptimizerPlan::get_chunks_sizes(len_some_size), [8192, 2048, 1024, 512, 256]);
    }
}