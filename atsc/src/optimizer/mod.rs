//! Chunking and codec selection.

pub mod chunker;
pub mod stats;

use crate::codec::constant::ConstantCodec;
use crate::codec::{Codec, CompressConfig, CompressedFrame};
use crate::error::{Error, Result};
use stats::DataStats;

/// Select the best codec for a chunk.
///
/// Strategy (PLAN.md Phase 3.1):
/// - If the data is constant (`min == max`), use the Constant codec immediately.
/// - Otherwise, try all codecs and select the smallest payload that meets the error bound.
/// - If no codec meets the bound, select the codec with the lowest reported `best` error and
///   retry using `max_error = best`.
///
/// # Errors
/// Returns `Err` if compression fails for all codecs.
#[must_use = "handle the Result to observe selection errors"]
pub fn select_codec(
    data: &[f64],
    codecs: &[&dyn Codec],
    config: &CompressConfig,
) -> Result<CompressedFrame> {
    if data.is_empty() {
        return Err(Error::EmptyData);
    }
    if data.iter().any(|v| !v.is_finite()) {
        return Err(Error::InvalidInput);
    }
    let stats = DataStats::new(data).ok_or(Error::EmptyData)?;

    if stats.min == stats.max {
        return ConstantCodec.compress(data, config);
    }

    let results = try_all_codecs(data, codecs, config);

    let mut best_ok: Option<CompressedFrame> = None;
    let mut best_err_bounds: Vec<(usize, f64)> = Vec::new();
    let mut last_err: Option<Error> = None;

    for (idx, res) in results.into_iter().enumerate() {
        match res {
            Ok(frame) => match &best_ok {
                None => best_ok = Some(frame),
                Some(current) => {
                    if frame.payload.len() < current.payload.len()
                        || (frame.payload.len() == current.payload.len()
                            && frame.measured_error < current.measured_error)
                    {
                        best_ok = Some(frame);
                    }
                }
            },
            Err(Error::ErrorBoundNotMet { best, .. }) => {
                best_err_bounds.push((idx, best));
            }
            Err(e) => last_err = Some(e),
        }
    }

    if let Some(frame) = best_ok {
        return Ok(frame);
    }

    // Fallback: choose lowest best error and re-run with relaxed bound.
    if let Some((best_idx, best)) = best_err_bounds
        .into_iter()
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    {
        let mut relaxed = config.clone();
        relaxed.max_error = Some(best);
        if let Some(codec) = codecs.get(best_idx) {
            if let Ok(frame) = codec.compress(data, &relaxed) {
                return Ok(frame);
            }
        }
    }

    Err(last_err.unwrap_or_else(|| Error::ResourceLimitExceeded("no codecs available".into())))
}

fn try_all_codecs(
    data: &[f64],
    codecs: &[&dyn Codec],
    config: &CompressConfig,
) -> Vec<Result<CompressedFrame>> {
    #[cfg(feature = "rayon")]
    {
        use rayon::prelude::*;
        codecs
            .par_iter()
            .map(|c| c.compress(data, config))
            .collect()
    }

    #[cfg(not(feature = "rayon"))]
    {
        codecs.iter().map(|c| c.compress(data, config)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::noop::NoopCodec;

    #[test]
    fn selects_constant_for_constant_data() {
        let data = vec![1.0; 1024];
        let cfg = CompressConfig::default();
        let codecs: [&dyn Codec; 1] = [&NoopCodec];
        let frame = select_codec(&data, &codecs, &cfg).unwrap();
        assert_eq!(frame.codec_id, ConstantCodec::ID);
        assert_eq!(frame.sample_count, 1024);
    }
}
