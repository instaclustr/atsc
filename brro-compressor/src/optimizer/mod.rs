// Lucas - Once the project is far enough along I strongly reccomend reenabling dead code checks
#![allow(dead_code)]

use median::Filter;
use log::debug;
use types::metric_tag::MetricTag;
use crate::types;

/// Max Frame size, this can aprox. 36h of data at 1point/sec rate, a little more than 1 week at 1point/5sec
/// and 1 month (30 days) at 1 point/20sec. 
/// This would be aprox. 1MB of Raw data (131072 * 64bits).
/// We wouldn't want to decompressed a ton of uncessary data, but for historical view of the data, looking into 1day/week/month at once is very reasonable
const MAX_FRAME_SIZE: usize = 131072; // 2^17
/// The Min frame size is one that allows our compressors potentially achieve 100x compression. Currently the most
/// limited one is the FFT compressor, that needs 3 frequencies at minimum, 3x100 = 300, next power of 2 is 512.
const MIN_FRAME_SIZE: usize = 512; // 2^9

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

/// This function gets a length and returns a vector with the chunk sizes to feed to the different compressors
/// A lot of assumptions go into selecting the chunk size, including:
/// 1. Collection rate - It is not expected that the collection rate exceeds 1point sec (it is expected actually less)
/// 2. Maximum compression achievable - A compressed frame as overhead and a minimum number of segments, small frames don't allow great compressions
/// 3. FFT operates faster under power of 2
fn get_chunks_sizes(len: usize) -> Vec<usize> {
    Vec::<usize>::with_capacity(MIN_FRAME_SIZE)
}

/// This should look at the data and return an optimized dataset for a specific compressor,
/// If a compressor is hand picked, this should be skipped.
pub fn process_data(wav_data: &Vec<f64>, tag: &MetricTag) -> Vec<f64> {
    // My idea here:
    // 1. Clean data
    // 2. Split into good sized chunks (aka power of 2)
    // 3. Get each chunk into the compressor that it should go
    // 3.1. Chunks should be at least of a size that it can allow a 100x compression for that given compressor (FFT is 512)
    let len = wav_data.len();
    if !len.is_power_of_two() {
        todo!()
    }
    // Cleaning data, removing NaN, etc. This might reduce sample count
    debug!("Tag: {:?} Len: {}", tag, wav_data.len());
    // Is len a power of 2? If not try to get the previous power of 2 
    wav_data.iter()
                .filter(|x| !(x.is_nan() || x.is_infinite()))
                .copied()
                .collect()
}