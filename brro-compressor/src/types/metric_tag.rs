use median::Filter;

#[derive(Debug)]
pub enum MetricTag {
    Percent(i32),
    // If it is a percent reduce significant digits to 2
    Duration(i32),
    // if it is a duration reduce precision to 1 microsecond
    NotFloat,
    // A metric that has a float representation but shouldn't (Eg. Precision is not needed)
    QuasiRandom,
    // A metric that exhibits a quasi random sample behavior. (E.g. Network deltas, heap memory)
    Bytes(i32),
    // Data that is in bytes... Make it MB, or KB
    Other, // Everything else
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
                Self::to_multiply_and_truncate(x, *y)
            }
            MetricTag::Duration(y) => {
                Self::to_multiply_and_truncate(x, *y)
            }
            MetricTag::Bytes(y) => {
                (x as i64) / (*y as i64)
            }
        }
    }

    /// Converts a float via multiplication and truncation
    fn to_multiply_and_truncate(number: f64, mul: i32) -> i64 {
        (number * mul as f64) as i64
    }

    fn to_median_filter(data: &[f64]) -> Vec<i64> {
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
}