//! Performance telemetry for codec selection.
//!
//! This module is compiled only when the `perf-telemetry` feature is enabled.

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

/// Coarse-grained telemetry for one codec attempt during chunk selection.
#[derive(Clone, Debug)]
pub struct CodecAttemptTelemetry {
    /// Codec id attempted.
    pub codec_id: u8,
    /// Human-readable codec name.
    pub codec_name: &'static str,
    /// Attempt position within the chunk (0-based).
    pub attempt_order: u16,
    /// Elapsed wall time for this attempt in nanoseconds.
    pub elapsed_ns: u64,
    /// Iteration count used by the codec (0 when unavailable).
    pub iteration_count: u32,
    /// Whether the attempt met the requested error bound.
    pub bound_met: bool,
    /// Error reported by the codec (`best` on bound miss).
    pub measured_error: f64,
    /// Payload size in bytes (`0` on failure).
    pub payload_bytes: u32,
}

/// Coarse-grained telemetry for one chunk selection.
#[derive(Clone, Debug)]
pub struct ChunkTelemetry {
    /// Chunk index in the run.
    pub chunk_index: u32,
    /// Number of input samples in this chunk.
    pub sample_count: u32,
    /// Attempt order and outcomes for each codec.
    pub attempts: Vec<CodecAttemptTelemetry>,
    /// Selected codec id.
    pub selected_codec_id: u8,
    /// Human-readable reason for choosing the selected codec.
    pub selection_reason: &'static str,
    /// Fallback classification: `none`, `lossy_best_effort`, or `noop_safety`.
    pub fallback_reason: &'static str,
    /// Whether fallback/retry path was used.
    pub retried: bool,
}

/// Summary values aggregated across the full compression run.
#[derive(Clone, Debug, Default)]
pub struct RunSummary {
    /// Number of chunks processed.
    pub chunk_count: u32,
    /// Mean number of attempts per chunk.
    pub attempts_per_chunk_avg: f64,
    /// P95 number of attempts per chunk.
    pub attempts_per_chunk_p95: u32,
    /// Number of retry events.
    pub retries_count: u32,
    /// Per-codec cumulative elapsed nanoseconds and bound miss count.
    pub per_codec: Vec<PerCodecSummary>,
    /// Bound miss histogram buckets.
    pub bound_miss_histogram: Vec<BoundMissBucket>,
    /// Number of chunks where Noop was selected.
    pub noop_selected_chunks: u32,
    /// Fraction of chunks where Noop was selected.
    pub noop_selected_rate: f64,
    /// Number of chunks where full-budget lossy met the bound.
    pub lossy_full_budget_bound_met_chunks: u32,
}

/// Aggregated stats for one codec over the run.
#[derive(Clone, Debug)]
pub struct PerCodecSummary {
    /// Codec id.
    pub codec_id: u8,
    /// Codec name.
    pub codec_name: &'static str,
    /// Cumulative attempt time in nanoseconds.
    pub elapsed_ns: u64,
    /// Percentage of total codec time share.
    pub time_share_pct: f64,
    /// Number of attempts.
    pub attempts: u32,
    /// Number of error-bound misses.
    pub bound_misses: u32,
}

/// Histogram bucket for error-bound misses.
#[derive(Clone, Debug)]
pub struct BoundMissBucket {
    /// Human-readable bucket label.
    pub bucket: &'static str,
    /// Number of misses in this bucket.
    pub count: u32,
}

/// In-memory collector used during one compression run.
#[derive(Default)]
pub struct RunTelemetryCollector {
    chunks: Vec<ChunkTelemetry>,
}

impl RunTelemetryCollector {
    /// Create an empty run collector.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append one chunk telemetry record.
    pub fn push_chunk(&mut self, chunk: ChunkTelemetry) {
        self.chunks.push(chunk);
    }

    /// Convert accumulated chunk telemetry into a run summary.
    #[must_use]
    pub fn summarize(&self) -> RunSummary {
        let chunk_count = self.chunks.len() as u32;
        let mut attempts_per_chunk = Vec::with_capacity(self.chunks.len());
        let mut retries_count = 0u32;
        let mut total_elapsed_ns = 0u64;
        let mut per_codec: BTreeMap<u8, (&'static str, u64, u32, u32)> = BTreeMap::new();
        let mut miss_buckets: BTreeMap<&'static str, u32> = BTreeMap::new();
        let mut noop_selected_chunks = 0u32;
        let mut lossy_full_budget_bound_met_chunks = 0u32;

        for chunk in &self.chunks {
            attempts_per_chunk.push(chunk.attempts.len() as u32);
            if chunk.selected_codec_id == 0 {
                noop_selected_chunks = noop_selected_chunks.saturating_add(1);
            }
            if chunk.selection_reason == "full_budget_lossy_bound_met" {
                lossy_full_budget_bound_met_chunks =
                    lossy_full_budget_bound_met_chunks.saturating_add(1);
            }
            if chunk.retried {
                retries_count = retries_count.saturating_add(1);
            }
            for attempt in &chunk.attempts {
                total_elapsed_ns = total_elapsed_ns.saturating_add(attempt.elapsed_ns);
                let entry = per_codec.entry(attempt.codec_id).or_insert((
                    attempt.codec_name,
                    0u64,
                    0u32,
                    0u32,
                ));
                entry.1 = entry.1.saturating_add(attempt.elapsed_ns);
                entry.2 = entry.2.saturating_add(1);
                if !attempt.bound_met {
                    entry.3 = entry.3.saturating_add(1);
                    let label = miss_bucket(attempt.measured_error);
                    *miss_buckets.entry(label).or_insert(0) += 1;
                }
            }
        }

        attempts_per_chunk.sort_unstable();
        let attempts_per_chunk_avg = if attempts_per_chunk.is_empty() {
            0.0
        } else {
            let total: u64 = attempts_per_chunk.iter().map(|&v| v as u64).sum();
            (total as f64) / (attempts_per_chunk.len() as f64)
        };
        let attempts_per_chunk_p95 = percentile_95(&attempts_per_chunk);

        let mut per_codec_vec = Vec::with_capacity(per_codec.len());
        for (codec_id, (codec_name, elapsed_ns, attempts, bound_misses)) in per_codec {
            let time_share_pct = if total_elapsed_ns == 0 {
                0.0
            } else {
                (elapsed_ns as f64) * 100.0 / (total_elapsed_ns as f64)
            };
            per_codec_vec.push(PerCodecSummary {
                codec_id,
                codec_name,
                elapsed_ns,
                time_share_pct,
                attempts,
                bound_misses,
            });
        }

        let histogram_order = [
            "finite <= 0.01",
            "finite <= 0.05",
            "finite <= 0.10",
            "finite > 0.10",
            "non-finite",
        ];
        let mut bound_miss_histogram = Vec::with_capacity(histogram_order.len());
        for bucket in histogram_order {
            bound_miss_histogram.push(BoundMissBucket {
                bucket,
                count: *miss_buckets.get(bucket).unwrap_or(&0),
            });
        }

        RunSummary {
            chunk_count,
            attempts_per_chunk_avg,
            attempts_per_chunk_p95,
            retries_count,
            per_codec: per_codec_vec,
            bound_miss_histogram,
            noop_selected_chunks,
            noop_selected_rate: if chunk_count == 0 {
                0.0
            } else {
                (noop_selected_chunks as f64) / (chunk_count as f64)
            },
            lossy_full_budget_bound_met_chunks,
        }
    }
}

/// Store the last run summary for inspection in tests and tools.
pub fn store_last_run_summary(summary: RunSummary) {
    let store = LAST_SUMMARY.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = store.lock() {
        *guard = Some(summary);
    }
}

/// Return a clone of the most recently stored run summary.
#[must_use]
pub fn last_run_summary() -> Option<RunSummary> {
    let store = LAST_SUMMARY.get_or_init(|| Mutex::new(None));
    if let Ok(guard) = store.lock() {
        return guard.clone();
    }
    None
}

fn miss_bucket(error: f64) -> &'static str {
    if !error.is_finite() {
        return "non-finite";
    }
    if error <= 0.01 {
        return "finite <= 0.01";
    }
    if error <= 0.05 {
        return "finite <= 0.05";
    }
    if error <= 0.10 {
        return "finite <= 0.10";
    }
    "finite > 0.10"
}

fn percentile_95(sorted: &[u32]) -> u32 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = ((sorted.len() as f64) * 0.95).ceil() as usize;
    let at = idx.saturating_sub(1).min(sorted.len().saturating_sub(1));
    sorted[at]
}

static LAST_SUMMARY: OnceLock<Mutex<Option<RunSummary>>> = OnceLock::new();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarize_computes_attempt_stats() {
        let mut collector = RunTelemetryCollector::new();
        collector.push_chunk(ChunkTelemetry {
            chunk_index: 0,
            sample_count: 512,
            attempts: vec![
                CodecAttemptTelemetry {
                    codec_id: 2,
                    codec_name: "fft-f32",
                    attempt_order: 0,
                    elapsed_ns: 100,
                    iteration_count: 0,
                    bound_met: false,
                    measured_error: 0.03,
                    payload_bytes: 0,
                },
                CodecAttemptTelemetry {
                    codec_id: 0,
                    codec_name: "noop",
                    attempt_order: 1,
                    elapsed_ns: 50,
                    iteration_count: 0,
                    bound_met: true,
                    measured_error: 0.0,
                    payload_bytes: 16,
                },
            ],
            selected_codec_id: 0,
            selection_reason: "smallest_payload_within_bound",
            fallback_reason: "none",
            retried: false,
        });
        let summary = collector.summarize();
        assert_eq!(summary.chunk_count, 1);
        assert_eq!(summary.attempts_per_chunk_p95, 2);
        assert_eq!(summary.retries_count, 0);
        assert_eq!(summary.per_codec.len(), 2);
    }
}
