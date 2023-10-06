// Lucas - Once the project is far enough along I strongly reccomend reenabling dead code checks
#![allow(dead_code)]

use median::Filter;
use log::debug;
use types::metric_tag::MetricTag;
use crate::types;

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


pub fn process_data(wav_data: &Vec<f64>, tag: &MetricTag) -> Vec<i64> {
    let mut _bitdepth = 64;
    let mut _dc_component: i64 = 0;
    let mut _fractional = true;

    debug!("Tag: {:?}", tag);
    let data = match tag {
        MetricTag::Other => Vec::new(),
        MetricTag::QuasiRandom => to_median_filter(wav_data),
        _ => {
            wav_data
                .iter()
                .map(|x| tag.from_float(*x))
                .collect()
        }
    };
    _fractional = false;
    data
}