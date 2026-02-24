//! VSRI segment type.

/// One constant-rate timestamp segment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Segment {
    /// Timestamp rate.
    pub rate: i64,
    /// Starting x value (sample index).
    pub x0: i64,
    /// Starting y value (timestamp).
    pub y0: i64,
    /// Number of samples covered by this segment.
    pub count: u32,
}
