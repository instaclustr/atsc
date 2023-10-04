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