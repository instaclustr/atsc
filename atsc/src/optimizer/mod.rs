//! Chunking and codec selection.

pub mod chunker;
pub mod stats;
#[cfg(feature = "perf-telemetry")]
pub mod telemetry;

use crate::codec::constant::ConstantCodec;
use crate::codec::noop::NoopCodec;
use crate::codec::{Codec, CompressConfig, CompressedFrame};
use crate::error::{Error, Result};
use stats::DataStats;
#[cfg(feature = "perf-telemetry")]
use std::time::Instant;
#[cfg(feature = "perf-telemetry")]
use telemetry::{ChunkTelemetry, CodecAttemptTelemetry, RunTelemetryCollector};

const PROBE_ITER_BUDGET: u32 = 3;

/// Select the best codec for a chunk.
///
/// Strategy (PLAN.md Phase 3.1):
/// - If the data is constant (`min == max`), use the Constant codec immediately.
/// - Probe all codecs with a small iteration budget and pick the smallest payload among
///   bound-meeting probe results.
/// - If no probe meets the bound, run full budget only on the best probe candidate by error.
/// - If full budget still misses, return the best-effort payload from the selected codec.
///
/// # Errors
/// Returns `Err` if compression fails for all codecs.
#[cfg(not(feature = "perf-telemetry"))]
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

    let sample_count: u32 = data
        .len()
        .try_into()
        .map_err(|_| Error::SampleCountOverflow {
            count: data.len() as u64,
        })?;
    let codecs = ensure_noop(codecs);

    if config.max_error.is_none() {
        let results = try_all_codecs(data, &codecs, config);
        let mut best_ok: Option<CompressedFrame> = None;
        let mut first_hard_err: Option<Error> = None;
        for res in results {
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
                Err(e) => {
                    if first_hard_err.is_none() {
                        first_hard_err = Some(e);
                    }
                }
            }
        }
        if let Some(frame) = best_ok {
            return Ok(frame);
        }
        return Err(first_hard_err
            .unwrap_or_else(|| Error::ResourceLimitExceeded("no codecs available".into())));
    }

    let mut probe_cfg = config.clone();
    probe_cfg.max_iterations = probe_cfg.max_iterations.min(PROBE_ITER_BUDGET);

    let mut best_probe_met: Option<(usize, CompressedFrame)> = None;
    let mut best_probe_candidate: Option<(usize, CompressedFrame)> = None;
    let mut first_hard_err: Option<Error> = None;

    for (idx, codec) in codecs.iter().enumerate() {
        match codec.compress(data, &probe_cfg) {
            Ok(frame) => {
                match &best_probe_met {
                    None => best_probe_met = Some((idx, frame.clone())),
                    Some((_, current)) => {
                        if frame.payload.len() < current.payload.len()
                            || (frame.payload.len() == current.payload.len()
                                && frame.measured_error < current.measured_error)
                        {
                            best_probe_met = Some((idx, frame.clone()));
                        }
                    }
                }
                match &best_probe_candidate {
                    None => best_probe_candidate = Some((idx, frame)),
                    Some((_, current)) => {
                        if frame.measured_error < current.measured_error
                            || (frame.measured_error == current.measured_error
                                && frame.payload.len() < current.payload.len())
                        {
                            best_probe_candidate = Some((idx, frame));
                        }
                    }
                }
            }
            Err(Error::ErrorBoundNotMet {
                best, best_payload, ..
            }) => {
                if let Some(payload) = best_payload {
                    let frame = CompressedFrame {
                        codec_id: codec.id(),
                        sample_count,
                        payload,
                        measured_error: best,
                    };
                    match &best_probe_candidate {
                        None => best_probe_candidate = Some((idx, frame)),
                        Some((best_idx, current)) => {
                            let ord = compare_best_error(best, current.measured_error)
                                .then_with(|| idx.cmp(best_idx));
                            if ord == std::cmp::Ordering::Less {
                                best_probe_candidate = Some((idx, frame));
                            }
                        }
                    }
                }
            }
            Err(e) => {
                if first_hard_err.is_none() {
                    first_hard_err = Some(e);
                }
            }
        }
    }

    if let Some((_, frame)) = best_probe_met {
        return Ok(frame);
    }

    let Some((selected_idx, selected_probe_frame)) = best_probe_candidate else {
        return Err(first_hard_err
            .unwrap_or_else(|| Error::ResourceLimitExceeded("no codecs available".into())));
    };

    if let Some(codec) = codecs.get(selected_idx) {
        match codec.compress(data, config) {
            Ok(frame) => return Ok(frame),
            Err(Error::ErrorBoundNotMet {
                best, best_payload, ..
            }) => {
                if let Some(payload) = best_payload {
                    return Ok(CompressedFrame {
                        codec_id: codec.id(),
                        sample_count,
                        payload,
                        measured_error: best,
                    });
                }
                return Ok(selected_probe_frame);
            }
            Err(_) => return Ok(selected_probe_frame),
        }
    }

    Ok(selected_probe_frame)
}

/// Select the best codec for a chunk.
///
/// # Errors
/// Returns `Err` if compression fails for all codecs.
#[cfg(feature = "perf-telemetry")]
#[must_use = "handle the Result to observe selection errors"]
pub fn select_codec(
    data: &[f64],
    codecs: &[&dyn Codec],
    config: &CompressConfig,
) -> Result<CompressedFrame> {
    select_codec_impl(data, codecs, config, None, 0)
}

/// Select the best codec and emit per-chunk telemetry.
///
/// This API is only available when the `perf-telemetry` feature is enabled.
#[cfg(feature = "perf-telemetry")]
#[must_use = "handle the Result to observe selection errors"]
pub fn select_codec_with_telemetry(
    data: &[f64],
    codecs: &[&dyn Codec],
    config: &CompressConfig,
    chunk_index: u32,
    collector: &mut RunTelemetryCollector,
) -> Result<CompressedFrame> {
    select_codec_impl(data, codecs, config, Some(collector), chunk_index)
}

#[cfg(feature = "perf-telemetry")]
fn select_codec_impl(
    data: &[f64],
    codecs: &[&dyn Codec],
    config: &CompressConfig,
    mut collector: Option<&mut RunTelemetryCollector>,
    chunk_index: u32,
) -> Result<CompressedFrame> {
    let stats = DataStats::new(data)?;

    if stats.min == stats.max {
        let start = Instant::now();
        let frame = ConstantCodec.compress(data, config)?;
        if let Some(c) = collector.as_mut() {
            let sample_count: u32 = data.len().try_into().unwrap_or(u32::MAX);
            c.push_chunk(ChunkTelemetry {
                chunk_index,
                sample_count,
                attempts: vec![CodecAttemptTelemetry {
                    codec_id: ConstantCodec::ID,
                    codec_name: "constant",
                    attempt_order: 0,
                    elapsed_ns: start.elapsed().as_nanos().try_into().unwrap_or(u64::MAX),
                    iteration_count: 0,
                    bound_met: true,
                    measured_error: frame.measured_error,
                    payload_bytes: frame.payload.len().try_into().unwrap_or(u32::MAX),
                }],
                selected_codec_id: frame.codec_id,
                selection_reason: "constant_data_fast_path",
                retried: false,
            });
        }
        return Ok(frame);
    }

    let codecs = ensure_noop(codecs);
    if config.max_error.is_none() {
        let mut attempts = Vec::with_capacity(codecs.len());
        let mut best_ok: Option<CompressedFrame> = None;
        let mut first_hard_err: Option<Error> = None;
        for (idx, codec) in codecs.iter().enumerate() {
            let start = Instant::now();
            match codec.compress(data, config) {
                Ok(frame) => {
                    attempts.push(CodecAttemptTelemetry {
                        codec_id: codec.id(),
                        codec_name: codec.name(),
                        attempt_order: idx as u16,
                        elapsed_ns: start.elapsed().as_nanos().try_into().unwrap_or(u64::MAX),
                        iteration_count: 0,
                        bound_met: true,
                        measured_error: frame.measured_error,
                        payload_bytes: frame.payload.len().try_into().unwrap_or(u32::MAX),
                    });
                    match &best_ok {
                        None => best_ok = Some(frame),
                        Some(current) => {
                            if frame.payload.len() < current.payload.len()
                                || (frame.payload.len() == current.payload.len()
                                    && frame.measured_error < current.measured_error)
                            {
                                best_ok = Some(frame);
                            }
                        }
                    }
                }
                Err(e) => {
                    attempts.push(CodecAttemptTelemetry {
                        codec_id: codec.id(),
                        codec_name: codec.name(),
                        attempt_order: idx as u16,
                        elapsed_ns: start.elapsed().as_nanos().try_into().unwrap_or(u64::MAX),
                        iteration_count: 0,
                        bound_met: false,
                        measured_error: f64::INFINITY,
                        payload_bytes: 0,
                    });
                    if first_hard_err.is_none() {
                        first_hard_err = Some(e);
                    }
                }
            }
        }
        let sample_count: u32 = data.len().try_into().unwrap_or(u32::MAX);
        if let Some(frame) = best_ok {
            if let Some(c) = collector.as_mut() {
                c.push_chunk(ChunkTelemetry {
                    chunk_index,
                    sample_count,
                    attempts,
                    selected_codec_id: frame.codec_id,
                    selection_reason: "smallest_payload_no_bound",
                    retried: false,
                });
            }
            return Ok(frame);
        }
        if let Some(c) = collector.as_mut() {
            c.push_chunk(ChunkTelemetry {
                chunk_index,
                sample_count,
                attempts,
                selected_codec_id: 255,
                selection_reason: "selection_failed",
                retried: false,
            });
        }
        return Err(first_hard_err
            .unwrap_or_else(|| Error::ResourceLimitExceeded("no codecs available".into())));
    }

    let mut probe_cfg = config.clone();
    probe_cfg.max_iterations = probe_cfg.max_iterations.min(PROBE_ITER_BUDGET);
    let sample_count: u32 = data.len().try_into().unwrap_or(u32::MAX);
    let mut attempts = Vec::with_capacity(codecs.len());
    let mut best_probe_met: Option<(usize, CompressedFrame)> = None;
    let mut best_probe_candidate: Option<(usize, CompressedFrame)> = None;
    let mut first_hard_err: Option<Error> = None;

    for (idx, codec) in codecs.iter().enumerate() {
        let start = Instant::now();
        match codec.compress(data, &probe_cfg) {
            Ok(frame) => {
                attempts.push(CodecAttemptTelemetry {
                    codec_id: codec.id(),
                    codec_name: codec.name(),
                    attempt_order: idx as u16,
                    elapsed_ns: start.elapsed().as_nanos().try_into().unwrap_or(u64::MAX),
                    iteration_count: 0,
                    bound_met: true,
                    measured_error: frame.measured_error,
                    payload_bytes: frame.payload.len().try_into().unwrap_or(u32::MAX),
                });
                match &best_probe_met {
                    None => best_probe_met = Some((idx, frame.clone())),
                    Some(current) => {
                        if frame.payload.len() < current.1.payload.len()
                            || (frame.payload.len() == current.1.payload.len()
                                && frame.measured_error < current.1.measured_error)
                        {
                            best_probe_met = Some((idx, frame.clone()));
                        }
                    }
                }
                match &best_probe_candidate {
                    None => best_probe_candidate = Some((idx, frame)),
                    Some(current) => {
                        if frame.measured_error < current.1.measured_error
                            || (frame.measured_error == current.1.measured_error
                                && frame.payload.len() < current.1.payload.len())
                        {
                            best_probe_candidate = Some((idx, frame));
                        }
                    }
                }
            }
            Err(Error::ErrorBoundNotMet {
                best, best_payload, ..
            }) => {
                let payload_len = best_payload
                    .as_ref()
                    .and_then(|p| u32::try_from(p.len()).ok())
                    .unwrap_or(0);
                attempts.push(CodecAttemptTelemetry {
                    codec_id: codec.id(),
                    codec_name: codec.name(),
                    attempt_order: idx as u16,
                    elapsed_ns: start.elapsed().as_nanos().try_into().unwrap_or(u64::MAX),
                    iteration_count: 0,
                    bound_met: false,
                    measured_error: best,
                    payload_bytes: payload_len,
                });
                if let Some(payload) = best_payload {
                    let frame = CompressedFrame {
                        codec_id: codec.id(),
                        sample_count,
                        payload,
                        measured_error: best,
                    };
                    match &best_probe_candidate {
                        None => best_probe_candidate = Some((idx, frame)),
                        Some((best_idx, current)) => {
                            let ord = compare_best_error(best, current.measured_error)
                                .then_with(|| idx.cmp(best_idx));
                            if ord == std::cmp::Ordering::Less {
                                best_probe_candidate = Some((idx, frame));
                            }
                        }
                    }
                }
            }
            Err(e) => {
                attempts.push(CodecAttemptTelemetry {
                    codec_id: codec.id(),
                    codec_name: codec.name(),
                    attempt_order: idx as u16,
                    elapsed_ns: start.elapsed().as_nanos().try_into().unwrap_or(u64::MAX),
                    iteration_count: 0,
                    bound_met: false,
                    measured_error: f64::INFINITY,
                    payload_bytes: 0,
                });
                if first_hard_err.is_none() {
                    first_hard_err = Some(e);
                }
            }
        }
    }

    if let Some((_, frame)) = best_probe_met {
        if let Some(c) = collector.as_mut() {
            c.push_chunk(ChunkTelemetry {
                chunk_index,
                sample_count,
                attempts,
                selected_codec_id: frame.codec_id,
                selection_reason: "smallest_payload_probe_bound_met",
                retried: false,
            });
        }
        return Ok(frame);
    }

    let Some((selected_idx, selected_probe_frame)) = best_probe_candidate else {
        if let Some(c) = collector.as_mut() {
            c.push_chunk(ChunkTelemetry {
                chunk_index,
                sample_count,
                attempts,
                selected_codec_id: 255,
                selection_reason: "selection_failed",
                retried: false,
            });
        }
        return Err(first_hard_err
            .unwrap_or_else(|| Error::ResourceLimitExceeded("no codecs available".into())));
    };

    if let Some(codec) = codecs.get(selected_idx) {
        let start = Instant::now();
        match codec.compress(data, config) {
            Ok(frame) => {
                attempts.push(CodecAttemptTelemetry {
                    codec_id: codec.id(),
                    codec_name: codec.name(),
                    attempt_order: attempts.len().try_into().unwrap_or(u16::MAX),
                    elapsed_ns: start.elapsed().as_nanos().try_into().unwrap_or(u64::MAX),
                    iteration_count: 0,
                    bound_met: true,
                    measured_error: frame.measured_error,
                    payload_bytes: frame.payload.len().try_into().unwrap_or(u32::MAX),
                });
                if let Some(c) = collector.as_mut() {
                    c.push_chunk(ChunkTelemetry {
                        chunk_index,
                        sample_count,
                        attempts,
                        selected_codec_id: frame.codec_id,
                        selection_reason: "full_budget_probe_winner",
                        retried: true,
                    });
                }
                return Ok(frame);
            }
            Err(Error::ErrorBoundNotMet {
                best, best_payload, ..
            }) => {
                let payload_len = best_payload
                    .as_ref()
                    .and_then(|p| u32::try_from(p.len()).ok())
                    .unwrap_or(0);
                attempts.push(CodecAttemptTelemetry {
                    codec_id: codec.id(),
                    codec_name: codec.name(),
                    attempt_order: attempts.len().try_into().unwrap_or(u16::MAX),
                    elapsed_ns: start.elapsed().as_nanos().try_into().unwrap_or(u64::MAX),
                    iteration_count: 0,
                    bound_met: false,
                    measured_error: best,
                    payload_bytes: payload_len,
                });
                let frame = if let Some(payload) = best_payload {
                    CompressedFrame {
                        codec_id: codec.id(),
                        sample_count,
                        payload,
                        measured_error: best,
                    }
                } else {
                    selected_probe_frame
                };
                if let Some(c) = collector.as_mut() {
                    c.push_chunk(ChunkTelemetry {
                        chunk_index,
                        sample_count,
                        attempts,
                        selected_codec_id: frame.codec_id,
                        selection_reason: "best_effort_probe_winner",
                        retried: true,
                    });
                }
                return Ok(frame);
            }
            Err(_) => {
                if let Some(c) = collector.as_mut() {
                    c.push_chunk(ChunkTelemetry {
                        chunk_index,
                        sample_count,
                        attempts,
                        selected_codec_id: selected_probe_frame.codec_id,
                        selection_reason: "best_effort_probe_winner",
                        retried: true,
                    });
                }
                return Ok(selected_probe_frame);
            }
        }
    }

    if let Some(c) = collector.as_mut() {
        c.push_chunk(ChunkTelemetry {
            chunk_index,
            sample_count,
            attempts,
            selected_codec_id: 255,
            selection_reason: "selection_failed",
            retried: false,
        });
    }
    Ok(selected_probe_frame)
}

#[cfg(not(feature = "perf-telemetry"))]
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

fn compare_best_error(a: f64, b: f64) -> std::cmp::Ordering {
    match (a.is_finite(), b.is_finite()) {
        (true, true) => a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal),
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        (false, false) => std::cmp::Ordering::Equal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::noop::NoopCodec;
    use crate::codec::CompressConfig;
    use std::sync::atomic::{AtomicUsize, Ordering};

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
                best_payload: None,
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

    struct ProbeAwareCodec<'a> {
        id: u8,
        probe_best: f64,
        full_error: f64,
        probe_calls: &'a AtomicUsize,
        full_calls: &'a AtomicUsize,
    }

    impl Codec for ProbeAwareCodec<'_> {
        fn id(&self) -> u8 {
            self.id
        }

        fn name(&self) -> &'static str {
            "probe-aware"
        }

        fn compress(&self, data: &[f64], config: &CompressConfig) -> Result<CompressedFrame> {
            let sample_count: u32 =
                data.len()
                    .try_into()
                    .map_err(|_| Error::SampleCountOverflow {
                        count: data.len() as u64,
                    })?;
            if config.max_iterations <= PROBE_ITER_BUDGET {
                self.probe_calls.fetch_add(1, Ordering::Relaxed);
                return Err(Error::ErrorBoundNotMet {
                    best: self.probe_best,
                    target: config.max_error.unwrap_or(0.0),
                    best_payload: Some(vec![self.id; 8]),
                });
            }
            self.full_calls.fetch_add(1, Ordering::Relaxed);
            Ok(CompressedFrame {
                codec_id: self.id,
                sample_count,
                payload: vec![self.id; 16],
                measured_error: self.full_error,
            })
        }

        fn decompress(&self, _payload: &[u8], _sample_count: u32) -> Result<Vec<f64>> {
            Err(Error::UnknownCodec(self.id))
        }
    }

    #[test]
    fn reduced_probe_runs_full_budget_only_on_selected_codec() {
        let a_probe_calls = AtomicUsize::new(0);
        let a_full_calls = AtomicUsize::new(0);
        let b_probe_calls = AtomicUsize::new(0);
        let b_full_calls = AtomicUsize::new(0);
        let codec_a = ProbeAwareCodec {
            id: 0,
            probe_best: 0.2,
            full_error: 0.05,
            probe_calls: &a_probe_calls,
            full_calls: &a_full_calls,
        };
        let codec_b = ProbeAwareCodec {
            id: 241,
            probe_best: 0.4,
            full_error: 0.02,
            probe_calls: &b_probe_calls,
            full_calls: &b_full_calls,
        };
        let codecs: [&dyn Codec; 2] = [&codec_a, &codec_b];
        let cfg = CompressConfig {
            max_error: Some(-1.0),
            max_iterations: 20,
            ..Default::default()
        };
        let data: Vec<f64> = (0..512).map(|i| (i as f64).sin()).collect();

        let frame = select_codec(&data, &codecs, &cfg).unwrap();
        assert_eq!(frame.codec_id, 0);
        assert_eq!(a_probe_calls.load(Ordering::Relaxed), 1);
        assert_eq!(b_probe_calls.load(Ordering::Relaxed), 1);
        assert_eq!(a_full_calls.load(Ordering::Relaxed), 1);
        assert_eq!(b_full_calls.load(Ordering::Relaxed), 0);
    }
}
