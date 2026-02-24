//! VSRI (timestamp) compression for ATSC v2 streams.

pub mod segment;

use crate::bytes::{take_i64_le, take_u32_le};
use crate::error::{Error, Result};
use crate::format::limits::MAX_VSRI_SEGMENTS;
use segment::Segment;

/// VSRI index describing timestamp segments for a values stream.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Vsri {
    /// Minimum timestamp (inclusive).
    pub min_ts: i64,
    /// Maximum timestamp (inclusive).
    pub max_ts: i64,
    /// Constant-rate segments.
    pub segments: Vec<Segment>,
}

impl Vsri {
    /// Build a VSRI structure from strictly increasing timestamps.
    ///
    /// # Errors
    /// - Returns `Err(Error::EmptyData)` when `timestamps` is empty.
    /// - Returns `Err(Error::VsriOutOfOrder)` when timestamps are not strictly increasing.
    #[must_use = "handle the Result to observe VSRI errors"]
    pub fn from_timestamps(timestamps: &[i64]) -> Result<Self> {
        let (first, rest) = timestamps.split_first().ok_or(Error::EmptyData)?;
        let min_ts = *first;
        let mut max_ts = *first;
        let mut prev = *first;
        for &ts in rest {
            if ts <= prev {
                return Err(Error::VsriOutOfOrder { ts, max: prev });
            }
            prev = ts;
            max_ts = ts;
        }

        let segments = build_segments(timestamps)?;
        Ok(Self {
            min_ts,
            max_ts,
            segments,
        })
    }

    /// Expand segments back into a timestamp vector.
    ///
    /// # Errors
    /// Returns `Err` on invariant violations or arithmetic overflow.
    #[must_use = "handle the Result to observe VSRI errors"]
    pub fn timestamps(&self) -> Result<Vec<i64>> {
        validate_segments(&self.segments)?;
        let total = self.total_samples()?;
        let total_usize: usize = total.try_into().map_err(|_| {
            Error::ResourceLimitExceeded("timestamp count does not fit usize".into())
        })?;
        let mut out = Vec::with_capacity(total_usize);

        for seg in &self.segments {
            for i in 0..seg.count {
                let x = seg.x0 + i as i64;
                out.push(seg.time_at(x)?);
            }
        }

        // Monotonic check (strict).
        let mut prev = out[0];
        for &ts in &out[1..] {
            if ts <= prev {
                return Err(Error::VsriInvariantViolation(
                    "expanded timestamps are not strictly increasing".into(),
                ));
            }
            prev = ts;
        }

        if out.first().copied() != Some(self.min_ts) || out.last().copied() != Some(self.max_ts) {
            return Err(Error::VsriInvariantViolation(
                "min_ts/max_ts do not match expanded timestamps".into(),
            ));
        }

        Ok(out)
    }

    /// Total number of samples covered by all segments.
    ///
    /// # Errors
    /// Returns `Err` if the sum overflows.
    #[must_use = "handle the Result to observe VSRI errors"]
    pub fn total_samples(&self) -> Result<u64> {
        self.segments.iter().try_fold(0u64, |acc, s| {
            acc.checked_add(s.count as u64)
                .ok_or_else(|| Error::ResourceLimitExceeded("total VSRI samples overflow".into()))
        })
    }

    /// Encode VSRI payload bytes (codec id 128).
    #[must_use = "use the returned bytes as a VSRI codec payload"]
    pub fn encode(&self) -> Result<Vec<u8>> {
        validate_segments(&self.segments)?;
        let seg_count: u32 =
            self.segments.len().try_into().map_err(|_| {
                Error::ResourceLimitExceeded("segment_count exceeds u32::MAX".into())
            })?;
        if seg_count > MAX_VSRI_SEGMENTS {
            return Err(Error::ResourceLimitExceeded(
                "segment_count exceeds MAX_VSRI_SEGMENTS".into(),
            ));
        }

        let mut out = Vec::with_capacity(8 + 8 + 4 + (self.segments.len() * 28));
        out.extend_from_slice(&self.min_ts.to_le_bytes());
        out.extend_from_slice(&self.max_ts.to_le_bytes());
        out.extend_from_slice(&seg_count.to_le_bytes());
        for s in &self.segments {
            out.extend_from_slice(&s.rate.to_le_bytes());
            out.extend_from_slice(&s.x0.to_le_bytes());
            out.extend_from_slice(&s.y0.to_le_bytes());
            out.extend_from_slice(&s.count.to_le_bytes());
        }
        Ok(out)
    }

    /// Decode a VSRI payload.
    ///
    /// # Errors
    /// Returns `Err` on truncation or invariant violations.
    #[must_use = "handle the Result to observe VSRI decode errors"]
    pub fn decode(payload: &[u8]) -> Result<Self> {
        let mut off = 0usize;
        let min_ts = take_i64_le(payload, &mut off)?;
        let max_ts = take_i64_le(payload, &mut off)?;
        let seg_count = take_u32_le(payload, &mut off)?;
        if seg_count > MAX_VSRI_SEGMENTS {
            return Err(Error::ResourceLimitExceeded(
                "segment_count exceeds MAX_VSRI_SEGMENTS".into(),
            ));
        }
        let seg_count_usize: usize = seg_count
            .try_into()
            .map_err(|_| Error::ResourceLimitExceeded("segment_count does not fit usize".into()))?;
        let mut segments = Vec::with_capacity(seg_count_usize);
        for _ in 0..seg_count_usize {
            let rate = take_i64_le(payload, &mut off)?;
            let x0 = take_i64_le(payload, &mut off)?;
            let y0 = take_i64_le(payload, &mut off)?;
            let count = take_u32_le(payload, &mut off)?;
            segments.push(Segment {
                rate,
                x0,
                y0,
                count,
            });
        }
        if off != payload.len() {
            return Err(Error::ResourceLimitExceeded(
                "vsri payload has trailing bytes".into(),
            ));
        }

        let vsri = Self {
            min_ts,
            max_ts,
            segments,
        };
        // Validate by expanding; ensures strict monotonic and min/max consistency.
        let _ = vsri.timestamps()?;
        Ok(vsri)
    }
}

fn build_segments(timestamps: &[i64]) -> Result<Vec<Segment>> {
    let len = timestamps.len();
    if len == 0 {
        return Err(Error::EmptyData);
    }
    if len == 1 {
        return Ok(vec![Segment {
            rate: 0,
            x0: 0,
            y0: timestamps[0],
            count: 1,
        }]);
    }

    let mut segments = Vec::new();
    let mut start: usize = 0;
    while start < len {
        if start == len - 1 {
            segments.push(Segment {
                rate: 0,
                x0: start as i64,
                y0: timestamps[start],
                count: 1,
            });
            break;
        }

        let rate = timestamps[start + 1]
            .checked_sub(timestamps[start])
            .ok_or_else(|| Error::ResourceLimitExceeded("timestamp delta overflow".into()))?;
        let mut count: usize = 2;
        while start + count < len {
            let prev = timestamps[start + count - 1];
            let cur = timestamps[start + count];
            let delta = cur
                .checked_sub(prev)
                .ok_or_else(|| Error::ResourceLimitExceeded("timestamp delta overflow".into()))?;
            if delta != rate {
                break;
            }
            count += 1;
        }

        let count_u32: u32 = count
            .try_into()
            .map_err(|_| Error::ResourceLimitExceeded("segment count exceeds u32::MAX".into()))?;
        segments.push(Segment {
            rate,
            x0: start as i64,
            y0: timestamps[start],
            count: count_u32,
        });

        start += count;
    }

    validate_segments(&segments)?;
    Ok(segments)
}

fn validate_segments(segments: &[Segment]) -> Result<()> {
    if segments.is_empty() {
        return Err(Error::VsriInvariantViolation(
            "vsri must contain at least one segment".into(),
        ));
    }
    let mut expected_x0 = 0i64;
    for (idx, s) in segments.iter().enumerate() {
        if s.count == 0 {
            return Err(Error::VsriInvariantViolation(
                "segment count must be >= 1".into(),
            ));
        }
        if s.x0 != expected_x0 {
            return Err(Error::VsriInvariantViolation(format!(
                "segment {idx} has x0={}, expected {}",
                s.x0, expected_x0
            )));
        }
        expected_x0 = expected_x0
            .checked_add(s.count as i64)
            .ok_or_else(|| Error::ResourceLimitExceeded("sample index overflow".into()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_time_at_uses_x0_offset() {
        let s = Segment {
            rate: 10,
            x0: 100,
            y0: 1_000,
            count: 10,
        };
        assert_eq!(s.time_at(100).unwrap(), 1_000);
        assert_eq!(s.time_at(101).unwrap(), 1_010);
        assert_eq!(s.time_at(109).unwrap(), 1_090);
    }

    #[test]
    fn vsri_encode_decode_roundtrip() {
        let timestamps = [100i64, 110, 120, 130, 200, 205, 210];
        let v = Vsri::from_timestamps(&timestamps).unwrap();
        let bytes = v.encode().unwrap();
        let decoded = Vsri::decode(&bytes).unwrap();
        assert_eq!(decoded, v);
        assert_eq!(decoded.timestamps().unwrap(), timestamps);
    }

    #[test]
    fn from_timestamps_rejects_non_monotonic() {
        let timestamps = [10i64, 10i64];
        let err = Vsri::from_timestamps(&timestamps).unwrap_err();
        assert!(matches!(err, Error::VsriOutOfOrder { .. }));
    }

    #[test]
    fn decode_rejects_non_contiguous_x0() {
        let mut payload = Vec::new();
        payload.extend_from_slice(&0i64.to_le_bytes()); // min_ts
        payload.extend_from_slice(&10i64.to_le_bytes()); // max_ts
        payload.extend_from_slice(&1u32.to_le_bytes()); // segment_count
        payload.extend_from_slice(&1i64.to_le_bytes()); // rate
        payload.extend_from_slice(&5i64.to_le_bytes()); // x0 (invalid; must be 0)
        payload.extend_from_slice(&0i64.to_le_bytes()); // y0
        payload.extend_from_slice(&2u32.to_le_bytes()); // count

        let err = Vsri::decode(&payload).unwrap_err();
        assert!(matches!(err, Error::VsriInvariantViolation(_)));
    }
}
