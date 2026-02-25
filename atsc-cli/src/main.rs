use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Compress a raw little-endian f64 file into an `.atsc` stream.
    Compress {
        input: PathBuf,
        /// Output path (defaults to `<input>.atsc`).
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Codec selection.
        #[arg(long, value_enum, default_value = "auto")]
        codec: CodecChoice,
        /// Maximum allowed error as a percentage (e.g. 5 = 5% NRMSE).
        #[arg(long, default_value_t = 5.0)]
        error: f64,
        /// Budget for bounded/iterative codecs.
        #[arg(long, default_value_t = 22)]
        iters: u32,
        /// Enable strict NaN/Inf rejection (no filtering).
        #[arg(long)]
        strict: bool,
    },

    /// Decompress an `.atsc` stream into raw little-endian f64 output.
    Decompress {
        input: PathBuf,
        /// Output path (defaults to `<input>.f64`).
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Inspect an ATSC v2 stream.
    Inspect { input: PathBuf },

    /// Run a quick benchmark of codecs over the input file.
    Bench {
        input: PathBuf,
        /// Maximum allowed error as a percentage (e.g. 5 = 5% NRMSE).
        #[arg(long, default_value_t = 5.0)]
        error: f64,
        /// Budget for bounded/iterative codecs.
        #[arg(long, default_value_t = 22)]
        iters: u32,
        /// Enable strict NaN/Inf rejection (no filtering).
        #[arg(long)]
        strict: bool,
    },
    /// Generate a baseline performance report matrix.
    Baseline {
        /// Input real-world `.f64` dataset used in the mixed/real-world profile.
        input: PathBuf,
        /// Output markdown report path.
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Number of runs per matrix cell (median reported).
        #[arg(long, default_value_t = 3)]
        runs: u32,
        /// Enable strict NaN/Inf rejection (no filtering).
        #[arg(long)]
        strict: bool,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum CodecChoice {
    Auto,
    Fft,
    Poly,
    Const,
    Noop,
}

fn main() {
    env_logger::init();
    let args = Args::parse();
    if let Err(e) = run(args) {
        log::error!("{e}");
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<(), atsc::Error> {
    match args.command {
        Command::Compress {
            input,
            output,
            codec,
            error,
            iters,
            strict,
        } => cmd_compress(&input, output.as_ref(), codec, error, iters, strict),
        Command::Decompress { input, output } => cmd_decompress(&input, output.as_ref()),
        Command::Inspect { input } => cmd_inspect(&input),
        Command::Bench {
            input,
            error,
            iters,
            strict,
        } => cmd_bench(&input, error, iters, strict),
        Command::Baseline {
            input,
            output,
            runs,
            strict,
        } => cmd_baseline(&input, output.as_ref(), runs, strict),
    }
}

fn cmd_compress(
    input: &PathBuf,
    output: Option<&PathBuf>,
    codec: CodecChoice,
    error_percent: f64,
    iters: u32,
    strict: bool,
) -> Result<(), atsc::Error> {
    let data = read_f64_file(input)?;
    let cfg = atsc::codec::CompressConfig {
        max_error: Some(error_percent / 100.0),
        max_iterations: iters,
        reject_nan_inf: strict,
    };

    let bytes = match codec {
        CodecChoice::Auto => atsc::compress(&data, &cfg)?,
        CodecChoice::Noop => encode_forced(&data, &cfg, &atsc::codec::noop::NoopCodec)?,
        CodecChoice::Const => encode_forced(&data, &cfg, &atsc::codec::constant::ConstantCodec)?,
        CodecChoice::Fft => encode_forced(&data, &cfg, &atsc::codec::fft::FftF32Codec)?,
        CodecChoice::Poly => encode_forced(&data, &cfg, &atsc::codec::polynomial::PolynomialCodec)?,
    };

    let out_path = output
        .cloned()
        .unwrap_or_else(|| input.with_extension("atsc"));
    std::fs::write(out_path, bytes)?;
    Ok(())
}

fn encode_forced(
    data: &[f64],
    cfg: &atsc::codec::CompressConfig,
    codec: &dyn atsc::codec::Codec,
) -> Result<Vec<u8>, atsc::Error> {
    let filtered = if cfg.reject_nan_inf {
        if data.iter().any(|v| !v.is_finite()) {
            return Err(atsc::Error::InvalidInput);
        }
        data.to_vec()
    } else {
        let mut out = Vec::with_capacity(data.len());
        let mut removed = 0usize;
        for &v in data {
            if v.is_finite() {
                out.push(v);
            } else {
                removed += 1;
            }
        }
        if removed > 0 {
            log::warn!("filtered {removed} NaN/Inf samples from input");
        }
        out
    };

    let plan = atsc::optimizer::chunker::Plan::new(filtered.len());
    let mut frames = Vec::with_capacity(plan.chunks.len());
    for range in plan.chunks {
        let chunk = &filtered[range];
        let frame = match codec.compress(chunk, cfg) {
            Ok(frame) => frame,
            Err(atsc::Error::ErrorBoundNotMet {
                best, best_payload, ..
            }) => {
                let sample_count: u32 =
                    chunk
                        .len()
                        .try_into()
                        .map_err(|_| atsc::Error::SampleCountOverflow {
                            count: chunk.len() as u64,
                        })?;
                let payload = best_payload.ok_or_else(|| {
                    atsc::Error::ResourceLimitExceeded(
                        "codec bound miss did not include best-effort payload".into(),
                    )
                })?;
                atsc::codec::CompressedFrame {
                    codec_id: codec.id(),
                    sample_count,
                    payload,
                    measured_error: best,
                }
            }
            Err(e) => return Err(e),
        };
        frames.push(frame);
    }
    let header = atsc::format::header::Header {
        version: 1,
        flags: 0,
    };
    atsc::format::stream::encode_stream(&header, &frames)
}

fn cmd_decompress(input: &PathBuf, output: Option<&PathBuf>) -> Result<(), atsc::Error> {
    let bytes = std::fs::read(input)?;
    let header = atsc::format::stream::decode_header(&bytes)?;
    let out_base = output
        .cloned()
        .unwrap_or_else(|| input.with_extension("f64"));

    if (header.flags & 1) != 0 {
        let (timestamps, values) = atsc::decompress_with_timestamps(&bytes)?;
        let ts_path = out_base.with_extension("timestamps.i64");
        let values_path = out_base.with_extension("values.f64");
        write_i64_file(&ts_path, &timestamps)?;
        write_f64_file(&values_path, &values)?;
        return Ok(());
    }

    let values = atsc::decompress(&bytes)?;
    write_f64_file(&out_base, &values)?;
    Ok(())
}

fn cmd_inspect(input: &PathBuf) -> Result<(), atsc::Error> {
    let bytes = std::fs::read(input)?;
    let info = atsc::inspect(&bytes)?;
    println!("version: {}", info.version);
    println!("frame_count: {}", info.frame_count);
    println!("total_samples: {}", info.total_samples);
    println!("has_vsri: {}", info.has_vsri);
    println!();
    println!("frames:");
    for (i, f) in info.frames.iter().enumerate() {
        println!(
            "  {:>4}: codec={} ({}), samples={}, payload_bytes={}",
            i, f.codec_name, f.codec_id, f.sample_count, f.payload_bytes
        );
    }
    Ok(())
}

fn cmd_bench(
    input: &PathBuf,
    error_percent: f64,
    iters: u32,
    strict: bool,
) -> Result<(), atsc::Error> {
    let data = read_f64_file(input)?;
    let cfg = atsc::codec::CompressConfig {
        max_error: Some(error_percent / 100.0),
        max_iterations: iters,
        reject_nan_inf: strict,
    };

    let codecs: [(&str, &dyn atsc::codec::Codec); 4] = [
        ("noop", &atsc::codec::noop::NoopCodec),
        ("const", &atsc::codec::constant::ConstantCodec),
        ("fft", &atsc::codec::fft::FftF32Codec),
        ("poly", &atsc::codec::polynomial::PolynomialCodec),
    ];

    println!("name\tbytes\terror\ttime_ms");
    for (name, codec) in codecs {
        let start = std::time::Instant::now();
        let bytes = encode_forced(&data, &cfg, codec)?;
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        let info = atsc::inspect(&bytes)?;
        // For bench output, report total payload bytes and worst-case measured_error is not tracked yet.
        let total_bytes: u64 = info.frames.iter().map(|f| f.payload_bytes as u64).sum();
        println!("{name}\t{total_bytes}\tn/a\t{elapsed:.3}");
    }

    Ok(())
}

#[cfg(feature = "perf-telemetry")]
#[derive(Clone, Copy, Debug)]
enum BaselineMode {
    Auto,
    Fft,
    Poly,
    Noop,
}

#[cfg(feature = "perf-telemetry")]
impl BaselineMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Fft => "forced-fft",
            Self::Poly => "forced-poly",
            Self::Noop => "forced-noop",
        }
    }
}

#[cfg(feature = "perf-telemetry")]
#[derive(Clone, Debug)]
struct BenchCell {
    dataset: &'static str,
    chunk_size: usize,
    error_percent: f64,
    mode: BaselineMode,
    median_time_ms: f64,
    ratio: f64,
    nrmse: f64,
    per_chunk_decision_ms: Option<f64>,
    attempts_avg: Option<f64>,
    attempts_p95: Option<u32>,
    retries_count: Option<u32>,
    bound_miss_rate: Option<f64>,
    codec_time_share: Option<String>,
}

fn cmd_baseline(
    input: &PathBuf,
    output: Option<&PathBuf>,
    runs: u32,
    strict: bool,
) -> Result<(), atsc::Error> {
    if runs == 0 {
        return Err(atsc::Error::ResourceLimitExceeded(
            "runs must be at least 1".into(),
        ));
    }
    #[cfg(not(feature = "perf-telemetry"))]
    {
        let _ = (input, output, strict);
        Err(atsc::Error::ResourceLimitExceeded(
            "baseline requires building atsc-cli with --features perf-telemetry".into(),
        ))
    }
    #[cfg(feature = "perf-telemetry")]
    {
        let real = read_f64_file(input)?;
        if real.is_empty() {
            return Err(atsc::Error::EmptyData);
        }

        let mut rows = Vec::new();
        let chunk_sizes = [512usize, 2048, 8192, 65_536];
        let errors = [0.5_f64, 1.0, 2.0, 5.0];
        let modes = [
            BaselineMode::Auto,
            BaselineMode::Fft,
            BaselineMode::Poly,
            BaselineMode::Noop,
        ];
        let datasets = build_baseline_datasets(&real, 262_144);

        for (dataset_name, values) in &datasets {
            for &chunk_size in &chunk_sizes {
                for &error_percent in &errors {
                    let cfg = atsc::codec::CompressConfig {
                        max_error: Some(error_percent / 100.0),
                        max_iterations: 22,
                        reject_nan_inf: strict,
                    };
                    for &mode in &modes {
                        let mut timings = Vec::with_capacity(runs as usize);
                        let mut last_outcome = None;
                        for _ in 0..runs {
                            let started = std::time::Instant::now();
                            let outcome = run_matrix_case(values, chunk_size, mode, &cfg)?;
                            let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
                            timings.push(elapsed_ms);
                            last_outcome = Some(outcome);
                        }
                        timings
                            .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                        let median_time_ms = timings[timings.len() / 2];
                        if let Some(outcome) = last_outcome {
                            rows.push(BenchCell {
                                dataset: dataset_name,
                                chunk_size,
                                error_percent,
                                mode,
                                median_time_ms,
                                ratio: outcome.ratio,
                                nrmse: outcome.nrmse,
                                per_chunk_decision_ms: outcome.per_chunk_decision_ms,
                                attempts_avg: outcome.attempts_avg,
                                attempts_p95: outcome.attempts_p95,
                                retries_count: outcome.retries_count,
                                bound_miss_rate: outcome.bound_miss_rate,
                                codec_time_share: outcome.codec_time_share,
                            });
                        }
                    }
                }
            }
        }

        let mut report = String::new();
        report.push_str("# ATSC Baseline Matrix Report\n\n");
        report.push_str("Datasets: periodic, smooth-trend, noisy-entropy, mixed-realworld\n");
        report.push_str("Chunk sizes: 512, 2048, 8192, 65536\n");
        report.push_str("Max error (%): 0.5, 1, 2, 5\n");
        report.push_str("Modes: auto, forced-fft, forced-poly, forced-noop\n");
        report.push_str(&format!("Runs per cell: {runs}\n\n"));
        report.push_str("|dataset|chunk|error_%|mode|median_time_ms|ratio|nrmse|decision_ms|attempts_avg|attempts_p95|retries|bound_miss_rate|codec_time_share|\n");
        report.push_str("|---|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---|\n");
        for row in &rows {
            let decision = row
                .per_chunk_decision_ms
                .map(|v| format!("{v:.4}"))
                .unwrap_or_else(|| "n/a".to_string());
            let attempts_avg = row
                .attempts_avg
                .map(|v| format!("{v:.2}"))
                .unwrap_or_else(|| "n/a".to_string());
            let attempts_p95 = row
                .attempts_p95
                .map(|v| v.to_string())
                .unwrap_or_else(|| "n/a".to_string());
            let retries = row
                .retries_count
                .map(|v| v.to_string())
                .unwrap_or_else(|| "n/a".to_string());
            let miss_rate = row
                .bound_miss_rate
                .map(|v| format!("{v:.3}"))
                .unwrap_or_else(|| "n/a".to_string());
            let share = row
                .codec_time_share
                .clone()
                .unwrap_or_else(|| "n/a".to_string());
            report.push_str(&format!(
                "|{}|{}|{:.1}|{}|{:.3}|{:.4}|{:.6}|{}|{}|{}|{}|{}|{}|\n",
                row.dataset,
                row.chunk_size,
                row.error_percent,
                row.mode.as_str(),
                row.median_time_ms,
                row.ratio,
                row.nrmse,
                decision,
                attempts_avg,
                attempts_p95,
                retries,
                miss_rate,
                share
            ));
        }
        report.push_str(
            "\nNotes: `attempts_*`, `retries`, `bound_miss_rate`, and `codec_time_share` come from perf-telemetry in auto mode only.\n",
        );
        report.push_str("Iteration metrics are currently not emitted by codecs, so per-codec avg/p95 iterations are not yet available.\n");

        let out = output
            .cloned()
            .unwrap_or_else(|| PathBuf::from("baseline-matrix.md"));
        std::fs::write(&out, report)?;
        println!("wrote baseline report: {}", out.display());
        Ok(())
    }
}

#[cfg(feature = "perf-telemetry")]
#[derive(Clone, Debug)]
struct CaseOutcome {
    ratio: f64,
    nrmse: f64,
    per_chunk_decision_ms: Option<f64>,
    attempts_avg: Option<f64>,
    attempts_p95: Option<u32>,
    retries_count: Option<u32>,
    bound_miss_rate: Option<f64>,
    codec_time_share: Option<String>,
}

#[cfg(feature = "perf-telemetry")]
fn run_matrix_case(
    data: &[f64],
    chunk_size: usize,
    mode: BaselineMode,
    cfg: &atsc::codec::CompressConfig,
) -> Result<CaseOutcome, atsc::Error> {
    let filtered = filter_values(data, cfg.reject_nan_inf)?;
    if filtered.is_empty() {
        return Err(atsc::Error::EmptyData);
    }
    let mut frames = Vec::new();
    match mode {
        BaselineMode::Auto => {
            let codecs: [&dyn atsc::codec::Codec; 3] = [
                &atsc::codec::noop::NoopCodec,
                &atsc::codec::fft::FftF32Codec,
                &atsc::codec::polynomial::PolynomialCodec,
            ];
            let mut collector = atsc::optimizer::telemetry::RunTelemetryCollector::new();
            for (idx, chunk) in split_chunks(&filtered, chunk_size).into_iter().enumerate() {
                let frame = atsc::optimizer::select_codec_with_telemetry(
                    chunk,
                    &codecs,
                    cfg,
                    idx.try_into().unwrap_or(u32::MAX),
                    &mut collector,
                )?;
                frames.push(frame);
            }
            let encoded = encode_frames(&frames)?;
            let decoded = atsc::decompress(&encoded)?;
            let nrmse = atsc::metrics::nrmse(&filtered, &decoded)?;
            let info = atsc::inspect(&encoded)?;
            let ratio = info.compression_ratio(encoded.len()).unwrap_or(0.0);
            let summary = collector.summarize();
            let total_attempts: u32 = summary.per_codec.iter().map(|v| v.attempts).sum();
            let total_misses: u32 = summary.per_codec.iter().map(|v| v.bound_misses).sum();
            let miss_rate = if total_attempts == 0 {
                0.0
            } else {
                (total_misses as f64) / (total_attempts as f64)
            };
            let total_elapsed_ns: u64 = summary.per_codec.iter().map(|v| v.elapsed_ns).sum();
            let per_chunk_decision_ms = if summary.chunk_count == 0 {
                0.0
            } else {
                (total_elapsed_ns as f64) / 1_000_000.0 / (summary.chunk_count as f64)
            };
            let mut shares = String::new();
            for (i, codec) in summary.per_codec.iter().enumerate() {
                if i > 0 {
                    shares.push_str(", ");
                }
                shares.push_str(&format!(
                    "{}:{:.1}%",
                    codec.codec_name, codec.time_share_pct
                ));
            }
            return Ok(CaseOutcome {
                ratio,
                nrmse,
                per_chunk_decision_ms: Some(per_chunk_decision_ms),
                attempts_avg: Some(summary.attempts_per_chunk_avg),
                attempts_p95: Some(summary.attempts_per_chunk_p95),
                retries_count: Some(summary.retries_count),
                bound_miss_rate: Some(miss_rate),
                codec_time_share: Some(shares),
            });
        }
        BaselineMode::Fft => {
            let codec = atsc::codec::fft::FftF32Codec;
            for chunk in split_chunks(&filtered, chunk_size) {
                frames.push(compress_chunk_best_effort(chunk, cfg, &codec)?);
            }
        }
        BaselineMode::Poly => {
            let codec = atsc::codec::polynomial::PolynomialCodec;
            for chunk in split_chunks(&filtered, chunk_size) {
                frames.push(compress_chunk_best_effort(chunk, cfg, &codec)?);
            }
        }
        BaselineMode::Noop => {
            let codec = atsc::codec::noop::NoopCodec;
            for chunk in split_chunks(&filtered, chunk_size) {
                frames.push(compress_chunk_best_effort(chunk, cfg, &codec)?);
            }
        }
    }
    let encoded = encode_frames(&frames)?;
    let decoded = atsc::decompress(&encoded)?;
    let nrmse = atsc::metrics::nrmse(&filtered, &decoded)?;
    let info = atsc::inspect(&encoded)?;
    let ratio = info.compression_ratio(encoded.len()).unwrap_or(0.0);
    Ok(CaseOutcome {
        ratio,
        nrmse,
        per_chunk_decision_ms: None,
        attempts_avg: None,
        attempts_p95: None,
        retries_count: None,
        bound_miss_rate: None,
        codec_time_share: None,
    })
}

#[cfg(feature = "perf-telemetry")]
fn build_baseline_datasets(real: &[f64], len: usize) -> Vec<(&'static str, Vec<f64>)> {
    let periodic: Vec<f64> = (0..len)
        .map(|i| {
            let x = i as f64;
            (x * 0.01).sin() + 0.3 * (x * 0.07).cos()
        })
        .collect();
    let smooth_trend: Vec<f64> = (0..len)
        .map(|i| {
            let t = i as f64 / len as f64;
            0.2 * (i as f64 * 0.01).sin() + (t * t * 5.0) - 2.5
        })
        .collect();
    let noisy_entropy = make_deterministic_noise(len);
    let mixed_realworld: Vec<f64> = (0..len)
        .map(|i| {
            let real_v = real[i % real.len()];
            match i % 4 {
                0 => periodic[i],
                1 => smooth_trend[i],
                2 => noisy_entropy[i],
                _ => real_v,
            }
        })
        .collect();
    vec![
        ("periodic", periodic),
        ("smooth-trend", smooth_trend),
        ("noisy-entropy", noisy_entropy),
        ("mixed-realworld", mixed_realworld),
    ]
}

#[cfg(feature = "perf-telemetry")]
fn make_deterministic_noise(len: usize) -> Vec<f64> {
    let mut state = 0x9E3779B97F4A7C15_u64;
    let mut out = Vec::with_capacity(len);
    for _ in 0..len {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let upper = (state >> 11) as f64;
        let unit = upper / ((1u64 << 53) as f64);
        out.push((unit * 2.0) - 1.0);
    }
    out
}

#[cfg(feature = "perf-telemetry")]
fn split_chunks(data: &[f64], chunk_size: usize) -> Vec<&[f64]> {
    if chunk_size == 0 {
        return vec![data];
    }
    let mut chunks = Vec::new();
    let mut start = 0usize;
    while start < data.len() {
        let end = (start + chunk_size).min(data.len());
        chunks.push(&data[start..end]);
        start = end;
    }
    chunks
}

#[cfg(feature = "perf-telemetry")]
fn filter_values(data: &[f64], strict: bool) -> Result<Vec<f64>, atsc::Error> {
    let mut out = Vec::with_capacity(data.len());
    for &v in data {
        if v.is_finite() {
            out.push(v);
        } else if strict {
            return Err(atsc::Error::InvalidInput);
        }
    }
    Ok(out)
}

#[cfg(feature = "perf-telemetry")]
fn encode_frames(frames: &[atsc::codec::CompressedFrame]) -> Result<Vec<u8>, atsc::Error> {
    let header = atsc::format::header::Header {
        version: 1,
        flags: 0,
    };
    atsc::format::stream::encode_stream(&header, frames)
}

#[cfg(feature = "perf-telemetry")]
fn compress_chunk_best_effort(
    data: &[f64],
    cfg: &atsc::codec::CompressConfig,
    codec: &dyn atsc::codec::Codec,
) -> Result<atsc::codec::CompressedFrame, atsc::Error> {
    match codec.compress(data, cfg) {
        Ok(frame) => Ok(frame),
        Err(atsc::Error::ErrorBoundNotMet {
            best, best_payload, ..
        }) => {
            let payload = best_payload.ok_or_else(|| {
                atsc::Error::ResourceLimitExceeded(
                    "codec bound miss did not include best-effort payload".into(),
                )
            })?;
            let sample_count: u32 =
                data.len()
                    .try_into()
                    .map_err(|_| atsc::Error::SampleCountOverflow {
                        count: data.len() as u64,
                    })?;
            Ok(atsc::codec::CompressedFrame {
                codec_id: codec.id(),
                sample_count,
                payload,
                measured_error: best,
            })
        }
        Err(e) => Err(e),
    }
}

fn read_f64_file(path: &PathBuf) -> Result<Vec<f64>, atsc::Error> {
    let bytes = std::fs::read(path)?;
    if bytes.len() % 8 != 0 {
        return Err(atsc::Error::UnexpectedEof {
            offset: bytes.len() as u64,
            expected: 8,
        });
    }
    let mut out = Vec::with_capacity(bytes.len() / 8);
    for chunk in bytes.chunks_exact(8) {
        let mut arr = [0u8; 8];
        arr.copy_from_slice(chunk);
        let v = f64::from_le_bytes(arr);
        out.push(v);
    }
    Ok(out)
}

fn write_f64_file(path: &PathBuf, values: &[f64]) -> Result<(), atsc::Error> {
    let mut out = Vec::with_capacity(values.len() * 8);
    for &v in values {
        out.extend_from_slice(&v.to_le_bytes());
    }
    std::fs::write(path, out)?;
    Ok(())
}

fn write_i64_file(path: &PathBuf, values: &[i64]) -> Result<(), atsc::Error> {
    let mut out = Vec::with_capacity(values.len() * 8);
    for &v in values {
        out.extend_from_slice(&v.to_le_bytes());
    }
    std::fs::write(path, out)?;
    Ok(())
}

#[cfg(all(test, feature = "perf-telemetry"))]
mod tests {
    use super::*;

    #[test]
    fn split_chunks_covers_all_samples() {
        let data: Vec<f64> = (0..10).map(|v| v as f64).collect();
        let chunks = split_chunks(&data, 4);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], &[0.0, 1.0, 2.0, 3.0]);
        assert_eq!(chunks[1], &[4.0, 5.0, 6.0, 7.0]);
        assert_eq!(chunks[2], &[8.0, 9.0]);
    }

    #[test]
    fn deterministic_noise_is_stable() {
        let a = make_deterministic_noise(8);
        let b = make_deterministic_noise(8);
        assert_eq!(a, b);
    }
}
