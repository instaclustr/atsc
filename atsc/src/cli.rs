/*
Copyright 2024 NetApp, Inc.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use std::{
    ffi::OsStr,
    fmt, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use atsc::{
    compressor::Compressor,
    csv::{read_samples, read_samples_with_headers},
    data::CompressedStream,
    decoder::Decoder,
    error::{DecodeError, EncodeError},
    optimizer::OptimizerPlan,
};
use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;
use wavbrro::wavbrro::{Error as WavBrroError, WavBrro};

/// Stable process exit codes for failures handled by the CLI boundary.
pub const EXIT_IO: i32 = 1;
pub const EXIT_USAGE: i32 = 2;
pub const EXIT_DECODE: i32 = 3;
pub const EXIT_ENCODE: i32 = 4;
pub const EXIT_INPUT_FORMAT: i32 = 5;

#[derive(Parser, Debug)]
#[command(author, version, about = "A Time-Series compressor", long_about = None)]
pub struct Args {
    /// Input file or directory (legacy compatibility mode)
    #[arg(value_name = "INPUT")]
    input: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Command>,

    /// Uncompresses the input file/directory (legacy compatibility mode)
    #[arg(short = 'u', action)]
    uncompress: bool,

    #[command(flatten)]
    compression: CompressionOptions,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Compress a WBRO or CSV file, or an eligible directory
    Compress(CompressArgs),
    /// Decompress a BRO file, or all BRO files in a directory
    Decompress(DecompressArgs),
    /// Inspect BRO metadata without decoding frame payloads
    Inspect(InspectArgs),
    /// Parse and fully decode a BRO file
    Verify(VerifyArgs),
}

#[derive(clap::Args, Debug)]
struct CompressArgs {
    input: PathBuf,

    /// Output file (single-file inputs only)
    #[arg(short, long)]
    output: Option<PathBuf>,

    #[command(flatten)]
    options: CompressionOptions,
}

#[derive(clap::Args, Debug)]
struct DecompressArgs {
    input: PathBuf,

    /// Output file (single-file inputs only)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Dump every decompressed sample to stdout
    #[arg(long, action)]
    verbose: bool,
}

#[derive(clap::Args, Debug)]
struct InspectArgs {
    input: PathBuf,

    /// Emit one JSON document instead of human-readable text
    #[arg(long, action)]
    json: bool,
}

#[derive(clap::Args, Debug)]
struct VerifyArgs {
    input: PathBuf,
}

#[derive(clap::Args, Debug)]
struct CompressionOptions {
    /// Select a compressor, default is auto
    #[arg(long, value_enum, default_value = "auto")]
    compressor: CompressorType,

    /// Sets the maximum allowed error for the compressed data, must be between 0 and 50. Default is 3 (3%).
    /// 0 is lossless compression
    /// 50 will do a median filter on the data.
    /// In between will optimize for the error
    #[arg(
        short = 'e',
        long,
        default_value_t = 3,
        value_parser = clap::value_parser!(u8).range(0..51),
        verbatim_doc_comment
    )]
    error: u8,

    /// Samples the input data instead of using all the data for selecting the optimal compressor.
    /// Only impacts speed and might or might not increase compression ratio. For best results use 0 (default).
    /// Only works when compression = Auto.
    /// 0 will use all the data (slowest)
    /// 6 will sample 128 data points (fastest)
    #[arg(
        short = 'c',
        long,
        default_value_t = 0,
        value_parser = clap::value_parser!(u8).range(0..7),
        verbatim_doc_comment
    )]
    compression_selection_sample_level: u8,

    /// Verbose output, dumps every sample in the input file
    #[arg(long, action)]
    verbose: bool,

    /// Defines user input as a CSV file
    #[arg(long, action)]
    csv: bool,

    /// Defines if the CSV has no header
    #[arg(long, action)]
    no_header: bool,

    /// Defines names of fields in CSV file. It should follow this format:
    ///   --fields=TIME_FIELD_NAME,VALUE_FIELD_NAME
    /// It assumes that the one before comma is a time field and the one
    /// after comma is a value field.
    #[arg(long, default_value = "time,value", verbatim_doc_comment)]
    fields: String,
}

#[derive(ValueEnum, Default, Clone, Copy, Debug)]
enum CompressorType {
    #[default]
    Auto,
    Noop,
    Fft,
    Constant,
    Polynomial,
    Idw,
    Rle,
}

impl From<CompressorType> for Compressor {
    fn from(value: CompressorType) -> Self {
        match value {
            CompressorType::Auto => Self::Auto,
            CompressorType::Noop => Self::Noop,
            CompressorType::Fft => Self::FFT,
            CompressorType::Constant => Self::Constant,
            CompressorType::Polynomial => Self::Polynomial,
            CompressorType::Idw => Self::Idw,
            CompressorType::Rle => Self::RLE,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Decode(#[from] DecodeError),
    #[error(transparent)]
    Encode(#[from] EncodeError),
    #[error("{0}")]
    Usage(String),
    #[error("{0}")]
    InputFormat(String),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Batch(BatchError),
}

impl CliError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Io(_) | Self::Json(_) => EXIT_IO,
            Self::Usage(_) => EXIT_USAGE,
            Self::Decode(_) => EXIT_DECODE,
            Self::Encode(_) => EXIT_ENCODE,
            Self::InputFormat(_) => EXIT_INPUT_FORMAT,
            Self::Batch(error) => error.exit_code(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct BatchError {
    failures: Vec<(PathBuf, CliError)>,
}

impl BatchError {
    fn exit_code(&self) -> i32 {
        self.failures
            .iter()
            .map(|(_, error)| error.exit_code())
            .max()
            .unwrap_or(EXIT_IO)
    }
}

impl fmt::Display for BatchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (path, error) in &self.failures {
            writeln!(formatter, "{}: {error}", path.display())?;
        }
        let count = self.failures.len();
        write!(
            formatter,
            "{count} file{} failed",
            if count == 1 { "" } else { "s" }
        )
    }
}

pub fn run(args: Args) -> Result<(), CliError> {
    match args.command {
        Some(Command::Compress(command)) => {
            compress_path(&command.input, command.output.as_deref(), &command.options)
        }
        Some(Command::Decompress(command)) => {
            decompress_path(&command.input, command.output.as_deref(), command.verbose)
        }
        Some(Command::Inspect(command)) => inspect(&command.input, command.json),
        Some(Command::Verify(command)) => verify(&command.input),
        None => {
            let input = args.input.ok_or_else(|| {
                CliError::Usage(
                    "an input file or directory is required; see `atsc --help`".to_string(),
                )
            })?;
            if args.uncompress {
                decompress_path(&input, None, args.compression.verbose)
            } else {
                compress_path(&input, None, &args.compression)
            }
        }
    }
}

fn compress_path(
    input: &Path,
    output: Option<&Path>,
    options: &CompressionOptions,
) -> Result<(), CliError> {
    let metadata = fs::metadata(input)?;
    if metadata.is_file() {
        let output = output
            .map(Path::to_path_buf)
            .unwrap_or_else(|| input.with_extension("bro"));
        return compress_file(input, &output, options);
    }
    if metadata.is_dir() {
        reject_directory_output(output)?;
        let extension = if options.csv { "csv" } else { "wbro" };
        return process_directory(input, extension, |path| {
            let output = path.with_extension("bro");
            compress_file(path, &output, options)
        });
    }
    Err(CliError::Usage(format!(
        "{} is neither a file nor a directory",
        input.display()
    )))
}

fn decompress_path(input: &Path, output: Option<&Path>, verbose: bool) -> Result<(), CliError> {
    let metadata = fs::metadata(input)?;
    if metadata.is_file() {
        let output = output
            .map(Path::to_path_buf)
            .unwrap_or_else(|| input.with_extension("wbro"));
        return decompress_file(input, &output, verbose);
    }
    if metadata.is_dir() {
        reject_directory_output(output)?;
        return process_directory(input, "bro", |path| {
            let output = path.with_extension("wbro");
            decompress_file(path, &output, verbose)
        });
    }
    Err(CliError::Usage(format!(
        "{} is neither a file nor a directory",
        input.display()
    )))
}

fn reject_directory_output(output: Option<&Path>) -> Result<(), CliError> {
    if output.is_some() {
        return Err(CliError::Usage(
            "--output can only be used with a single input file".to_string(),
        ));
    }
    Ok(())
}

fn process_directory(
    input: &Path,
    extension: &str,
    mut process: impl FnMut(&Path) -> Result<(), CliError>,
) -> Result<(), CliError> {
    let mut entries = fs::read_dir(input)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    entries.retain(|path| path.is_file() && has_extension(path, extension));
    entries.sort();

    let failures = entries
        .into_iter()
        .filter_map(|path| process(&path).err().map(|error| (path, error)))
        .collect::<Vec<_>>();
    if failures.is_empty() {
        Ok(())
    } else {
        Err(CliError::Batch(BatchError { failures }))
    }
}

fn has_extension(path: &Path, expected: &str) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|extension| extension.eq_ignore_ascii_case(expected))
}

fn compress_file(
    input: &Path,
    output: &Path,
    options: &CompressionOptions,
) -> Result<(), CliError> {
    let data = read_compression_input(input, options)?;
    if options.verbose {
        writeln!(io::stdout().lock(), "Input={data:?}")?;
    }
    let compressed = compress_data(&data, options)?;
    fs::write(output, compressed)?;
    Ok(())
}

fn read_compression_input(
    input: &Path,
    options: &CompressionOptions,
) -> Result<Vec<f64>, CliError> {
    if options.csv {
        let samples = if options.no_header {
            read_samples(input).map_err(|error| CliError::InputFormat(error.to_string()))?
        } else {
            let (timestamp_field, value_field) = parse_fields(&options.fields)?;
            read_samples_with_headers(input, timestamp_field, value_field)
                .map_err(|error| CliError::InputFormat(error.to_string()))?
        };
        return Ok(samples.into_iter().map(|sample| sample.value).collect());
    }

    WavBrro::from_file(input).map_err(map_wavbrro_error)
}

fn parse_fields(fields: &str) -> Result<(&str, &str), CliError> {
    let mut fields = fields.split(',').map(str::trim);
    let timestamp = fields.next().unwrap_or_default();
    let value = fields.next().unwrap_or_default();
    if timestamp.is_empty() || value.is_empty() || fields.next().is_some() {
        return Err(CliError::Usage(
            "--fields must be TIME_FIELD_NAME,VALUE_FIELD_NAME".to_string(),
        ));
    }
    Ok((timestamp, value))
}

fn map_wavbrro_error(error: WavBrroError) -> CliError {
    match error {
        WavBrroError::IoError(error) => CliError::Io(error),
        error => CliError::InputFormat(error.to_string()),
    }
}

fn compress_data(data: &[f64], options: &CompressionOptions) -> Result<Vec<u8>, EncodeError> {
    let mut plan = OptimizerPlan::plan(data);
    plan.set_compressor(options.compressor.into());
    let mut stream = CompressedStream::new();
    let max_error = f32::from(options.error) / 100.0;

    for (compressor, chunk) in plan.get_execution() {
        stream.try_compress_chunk_bounded_with(
            chunk,
            *compressor,
            max_error,
            usize::from(options.compression_selection_sample_level),
        )?;
    }
    Ok(stream.to_bytes())
}

fn decompress_file(input: &Path, output: &Path, verbose: bool) -> Result<(), CliError> {
    let stream = read_stream(input)?;
    let data = Decoder::new().decode(&stream)?;
    if verbose {
        writeln!(io::stdout().lock(), "Output={data:?}")?;
    }
    write_wbro(output, &data)
}

fn write_wbro(output: &Path, data: &[f64]) -> Result<(), CliError> {
    let mut wbro = WavBrro::new();
    for sample in data {
        wbro.add_sample(*sample);
    }
    let payload = wbro.to_bytes();
    let mut bytes = Vec::with_capacity(12 + payload.len());
    bytes.extend_from_slice(b"WBRO0000WBRO");
    bytes.extend_from_slice(&payload);
    fs::write(output, bytes)?;
    Ok(())
}

fn read_stream(input: &Path) -> Result<CompressedStream, CliError> {
    let bytes = fs::read(input)?;
    Ok(CompressedStream::try_from_bytes(&bytes)?)
}

#[derive(Serialize)]
struct Inspection {
    version: u32,
    frames: usize,
    samples: usize,
    frame_details: Vec<FrameDetail>,
}

#[derive(Serialize)]
struct FrameDetail {
    index: usize,
    sample_offset: usize,
    samples: usize,
    codec: &'static str,
    payload_bytes: usize,
}

fn inspection(stream: &CompressedStream) -> Inspection {
    let frame_details = stream
        .frame_info()
        .map(|frame| FrameDetail {
            index: frame.index,
            sample_offset: frame.sample_offset,
            samples: frame.sample_count,
            codec: compressor_name(frame.compressor),
            payload_bytes: frame.payload_bytes,
        })
        .collect();
    Inspection {
        version: stream.header.version,
        frames: stream.frame_count(),
        samples: stream.sample_count(),
        frame_details,
    }
}

fn inspect(input: &Path, json: bool) -> Result<(), CliError> {
    if !fs::metadata(input)?.is_file() {
        return Err(CliError::Usage(
            "inspect requires a single input file".to_string(),
        ));
    }
    let stream = read_stream(input)?;
    let inspection = inspection(&stream);
    let mut stdout = io::stdout().lock();
    if json {
        serde_json::to_writer(&mut stdout, &inspection)?;
        writeln!(stdout)?;
        return Ok(());
    }

    writeln!(stdout, "version: {}", inspection.version)?;
    writeln!(stdout, "frames: {}", inspection.frames)?;
    writeln!(stdout, "samples: {}", inspection.samples)?;
    for frame in inspection.frame_details {
        writeln!(
            stdout,
            "frame {}: offset={} samples={} codec={} payload bytes={}",
            frame.index, frame.sample_offset, frame.samples, frame.codec, frame.payload_bytes
        )?;
    }
    Ok(())
}

fn verify(input: &Path) -> Result<(), CliError> {
    if !fs::metadata(input)?.is_file() {
        return Err(CliError::Usage(
            "verify requires a single input file".to_string(),
        ));
    }
    let stream = read_stream(input)?;
    Decoder::new().decode(&stream)?;
    writeln!(io::stdout().lock(), "valid")?;
    Ok(())
}

fn compressor_name(compressor: Compressor) -> &'static str {
    match compressor {
        Compressor::Noop => "noop",
        Compressor::FFT => "fft",
        Compressor::Idw => "idw",
        Compressor::Constant => "constant",
        Compressor::Polynomial => "polynomial",
        Compressor::Auto => "auto",
        Compressor::RLE => "rle",
    }
}
