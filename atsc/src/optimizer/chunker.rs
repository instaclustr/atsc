//! Chunk boundary computation.

use std::ops::Range;

/// A compression plan over a full input slice.
#[derive(Clone, Debug)]
pub struct Plan {
    /// Chunk boundaries as ranges into the input slice.
    pub chunks: Vec<Range<usize>>,
}

impl Plan {
    /// Create a chunk plan for a given input length.
    #[must_use]
    pub fn new(data_len: usize) -> Self {
        let _ = data_len;
        // Implemented in Phase 3.2 / 3.3 of PLAN.md.
        Self { chunks: Vec::new() }
    }
}
