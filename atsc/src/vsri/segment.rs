//! VSRI segment type.

use crate::error::{Error, Result};

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

impl Segment {
    /// Last sample index covered by this segment (inclusive).
    #[must_use = "compute the segment end index"]
    pub fn x_end(&self) -> i64 {
        self.x0 + (self.count as i64).saturating_sub(1)
    }

    /// Timestamp at the given sample index `x`.
    ///
    /// # Errors
    /// Returns `Err` if `x` is outside the segment range or arithmetic overflows.
    #[must_use = "handle the Result to observe VSRI errors"]
    pub fn time_at(&self, x: i64) -> Result<i64> {
        if self.count == 0 {
            return Err(Error::VsriInvariantViolation(
                "segment count must be >= 1".into(),
            ));
        }
        if x < self.x0 || x > self.x_end() {
            return Err(Error::VsriInvariantViolation(
                "x outside segment range".into(),
            ));
        }
        let dx = x - self.x0;
        let dy = self
            .rate
            .checked_mul(dx)
            .ok_or_else(|| Error::ResourceLimitExceeded("timestamp arithmetic overflow".into()))?;
        self.y0
            .checked_add(dy)
            .ok_or_else(|| Error::ResourceLimitExceeded("timestamp arithmetic overflow".into()))
    }
}
