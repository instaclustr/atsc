//! Shared statistics computed over input data.

use crate::error::{Error, Result};

/// Basic statistics used by codecs and the optimizer.
#[derive(Clone, Copy, Debug)]
pub struct DataStats {
    /// Minimum value.
    pub min: f64,
    /// Maximum value.
    pub max: f64,
}

impl DataStats {
    /// Compute statistics over a non-empty slice.
    ///
    /// # Errors
    /// - Returns `Err(Error::EmptyData)` if `data` is empty.
    /// - Returns `Err(Error::InvalidInput)` if any value is NaN/Inf.
    #[must_use = "handle the Result to observe invalid input"]
    pub fn new(data: &[f64]) -> Result<Self> {
        let (first, rest) = data.split_first().ok_or(Error::EmptyData)?;
        if !first.is_finite() {
            return Err(Error::InvalidInput);
        }
        let mut min = *first;
        let mut max = *first;
        for v in rest {
            if !v.is_finite() {
                return Err(Error::InvalidInput);
            }
            if *v < min {
                min = *v;
            }
            if *v > max {
                max = *v;
            }
        }
        Ok(Self { min, max })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_nan() {
        let data = [1.0, f64::NAN];
        assert!(matches!(DataStats::new(&data), Err(Error::InvalidInput)));
    }

    #[test]
    fn rejects_inf() {
        let data = [1.0, f64::INFINITY];
        assert!(matches!(DataStats::new(&data), Err(Error::InvalidInput)));
    }
}
