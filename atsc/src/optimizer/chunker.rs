//! Chunk boundary computation.

use std::ops::Range;

const MIN_CHUNK: usize = 512;
const MAX_CHUNK: usize = 131_072;

/// A compression plan over a full input slice.
#[derive(Clone, Debug)]
pub struct Plan {
    /// Chunk boundaries as ranges into the input slice.
    pub chunks: Vec<Range<usize>>,
}

impl Plan {
    /// Create a chunk plan for a given input length.
    #[must_use = "Plan::new computes chunk boundaries"]
    pub fn new(data_len: usize) -> Self {
        if data_len == 0 {
            return Self { chunks: Vec::new() };
        }

        let mut chunks = Vec::new();
        let mut start = 0usize;
        let mut remaining = data_len;

        while remaining > 0 {
            let size = if remaining >= MAX_CHUNK {
                MAX_CHUNK
            } else if remaining <= MIN_CHUNK {
                remaining
            } else {
                prev_power_of_two(remaining)
            };

            chunks.push(start..start + size);
            start += size;
            remaining -= size;
        }

        Self { chunks }
    }
}

fn prev_power_of_two(n: usize) -> usize {
    debug_assert!(n > 0);
    if n.is_power_of_two() {
        return n;
    }
    let shift = usize::BITS - 1 - n.leading_zeros();
    1usize << shift
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunker_matches_v1_behavior_examples() {
        let plan = Plan::new(131_072 * 3 + 1765);
        let sizes: Vec<usize> = plan.chunks.iter().map(|r| r.end - r.start).collect();
        assert_eq!(sizes, [131_072, 131_072, 131_072, 1024, 512, 229]);

        let plan = Plan::new(31);
        let sizes: Vec<usize> = plan.chunks.iter().map(|r| r.end - r.start).collect();
        assert_eq!(sizes, [31]);

        let plan = Plan::new(2048);
        let sizes: Vec<usize> = plan.chunks.iter().map(|r| r.end - r.start).collect();
        assert_eq!(sizes, [2048]);

        let plan = Plan::new(12032);
        let sizes: Vec<usize> = plan.chunks.iter().map(|r| r.end - r.start).collect();
        assert_eq!(sizes, [8192, 2048, 1024, 512, 256]);
    }
}
