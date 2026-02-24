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
    Idw,
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
            strict,
        } => cmd_compress(&input, output.as_ref(), codec, error, strict),
        Command::Decompress { input, output } => cmd_decompress(&input, output.as_ref()),
        Command::Inspect { input } => cmd_inspect(&input),
        Command::Bench {
            input,
            error,
            strict,
        } => cmd_bench(&input, error, strict),
    }
}

fn cmd_compress(
    input: &PathBuf,
    output: Option<&PathBuf>,
    codec: CodecChoice,
    error_percent: f64,
    strict: bool,
) -> Result<(), atsc::Error> {
    let data = read_f64_file(input)?;
    let cfg = atsc::codec::CompressConfig {
        max_error: Some(error_percent / 100.0),
        max_iterations: 22,
        reject_nan_inf: strict,
    };

    let bytes = match codec {
        CodecChoice::Auto => atsc::compress(&data, &cfg)?,
        CodecChoice::Noop => encode_forced(&data, &cfg, &atsc::codec::noop::NoopCodec)?,
        CodecChoice::Const => encode_forced(&data, &cfg, &atsc::codec::constant::ConstantCodec)?,
        CodecChoice::Fft => encode_forced(&data, &cfg, &atsc::codec::fft::FftF32Codec)?,
        CodecChoice::Poly => encode_forced(&data, &cfg, &atsc::codec::polynomial::PolynomialCodec)?,
        CodecChoice::Idw => encode_forced(&data, &cfg, &atsc::codec::polynomial::IdwCodec)?,
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
        let frame = codec.compress(&filtered[range], cfg)?;
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

fn cmd_bench(input: &PathBuf, error_percent: f64, strict: bool) -> Result<(), atsc::Error> {
    let data = read_f64_file(input)?;
    let cfg = atsc::codec::CompressConfig {
        max_error: Some(error_percent / 100.0),
        max_iterations: 22,
        reject_nan_inf: strict,
    };

    let codecs: [(&str, &dyn atsc::codec::Codec); 5] = [
        ("noop", &atsc::codec::noop::NoopCodec),
        ("const", &atsc::codec::constant::ConstantCodec),
        ("fft", &atsc::codec::fft::FftF32Codec),
        ("poly", &atsc::codec::polynomial::PolynomialCodec),
        ("idw", &atsc::codec::polynomial::IdwCodec),
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

fn read_f64_file(path: &PathBuf) -> Result<Vec<f64>, atsc::Error> {
    let bytes = std::fs::read(path)?;
    if bytes.len() % 8 != 0 {
        return Err(atsc::Error::UnexpectedEof {
            offset: bytes.len(),
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
