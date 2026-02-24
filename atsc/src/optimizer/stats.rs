//! Shared statistics computed over input data.

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
    #[must_use]
    pub fn new(data: &[f64]) -> Option<Self> {
        let (first, rest) = data.split_first()?;
        let mut min = *first;
        let mut max = *first;
        for v in rest {
            if *v < min {
                min = *v;
            }
            if *v > max {
                max = *v;
            }
        }
        Some(Self { min, max })
    }
}
