//! Chunking and codec selection.

pub mod chunker;
pub mod stats;

use crate::codec::constant::ConstantCodec;
use crate::codec::noop::NoopCodec;
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
    let stats = DataStats::new(data)?;

    if stats.min == stats.max {
        return ConstantCodec.compress(data, config);
    }

    let codecs = ensure_noop(codecs);
    let results = try_all_codecs(data, &codecs, config);

    let mut best_ok: Option<CompressedFrame> = None;
    let mut best_err_bounds: Vec<(usize, f64)> = Vec::new(); // (idx, best_err)
    let mut first_hard_err: Option<(usize, Error)> = None;

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
            Err(e) => {
                if first_hard_err.is_none() {
                    first_hard_err = Some((idx, e));
                }
            }
        }
    }

    if let Some(frame) = best_ok {
        return Ok(frame);
    }

    // Fallback: choose lowest best error (deterministic tie-break by idx).
    if let Some((best_idx, best)) = best_err_bounds.into_iter().min_by(|a, b| {
        let ea = a.1;
        let eb = b.1;
        match (ea.is_finite(), eb.is_finite()) {
            (true, true) => ea
                .partial_cmp(&eb)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0)),
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            (false, false) => a.0.cmp(&b.0),
        }
    }) {
        let mut relaxed = config.clone();
        relaxed.max_error = Some(best);
        if let Some(codec) = codecs.get(best_idx) {
            match codec.compress(data, &relaxed) {
                Ok(frame) => return Ok(frame),
                Err(e) => {
                    if first_hard_err.is_none() {
                        first_hard_err = Some((best_idx, e));
                    }
                }
            }
        }
    }

    Err(first_hard_err
        .map(|(_, e)| e)
        .unwrap_or_else(|| Error::ResourceLimitExceeded("no codecs available".into())))
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

fn ensure_noop<'a>(codecs: &'a [&'a dyn Codec]) -> Vec<&'a dyn Codec> {
    if codecs.is_empty() {
        return vec![&NoopCodec];
    }
    if codecs.iter().any(|c| c.id() == NoopCodec::ID) {
        return codecs.to_vec();
    }
    let mut out = Vec::with_capacity(codecs.len() + 1);
    out.extend_from_slice(codecs);
    out.push(&NoopCodec);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::noop::NoopCodec;
    use crate::codec::CompressConfig;

    struct AlwaysBoundFailCodec {
        id: u8,
    }

    impl Codec for AlwaysBoundFailCodec {
        fn id(&self) -> u8 {
            self.id
        }

        fn name(&self) -> &'static str {
            "always-bound-fail"
        }

        fn compress(&self, data: &[f64], config: &CompressConfig) -> Result<CompressedFrame> {
            let target = config.max_error.unwrap_or(0.0);
            let _ = data;
            Err(Error::ErrorBoundNotMet {
                best: 0.123,
                target,
            })
        }

        fn decompress(&self, _payload: &[u8], _sample_count: u32) -> Result<Vec<f64>> {
            Err(Error::UnknownCodec(self.id))
        }
    }

    #[test]
    fn selects_constant_for_constant_data() {
        let data = vec![1.0; 1024];
        let cfg = CompressConfig::default();
        let codecs: [&dyn Codec; 1] = [&NoopCodec];
        let frame = select_codec(&data, &codecs, &cfg).unwrap();
        assert_eq!(frame.codec_id, ConstantCodec::ID);
        assert_eq!(frame.sample_count, 1024);
    }

    #[test]
    fn guarantees_noop_fallback_for_finite_data() {
        let data: Vec<f64> = (0..2048).map(|i| (i as f64).sin()).collect();
        let cfg = CompressConfig {
            max_error: Some(0.0),
            ..Default::default()
        };
        let bad = AlwaysBoundFailCodec { id: 250 };
        let codecs: [&dyn Codec; 1] = [&bad];
        let frame = select_codec(&data, &codecs, &cfg).unwrap();
        assert_eq!(frame.codec_id, NoopCodec::ID);
    }
}
