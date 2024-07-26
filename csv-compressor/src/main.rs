use crate::csv::{SampleParser, SampleWriter};
use crate::metric::Metric;
use brro_compressor::compressor::Compressor;
use brro_compressor::data::CompressedStream;
use brro_compressor::optimizer::OptimizerPlan;
use brro_compressor::utils::readers::bro_reader::read_file;
use clap::{arg, Parser};
use log::debug;
use std::cell::RefCell;
use std::fs;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::rc::Rc;
use vsri::Vsri;
use wavbrro::wavbrro::WavBrro;

mod csv;
mod metric;

#[derive(Parser, Default, Debug)]
#[command(
    author, version, about = "A Time-Series compressor utilizes Brro Compressor for CSV format", long_about = None
)]
pub struct Args {
    /// Path to input
    input: PathBuf,

    /// Defines where the result will be stored
    #[arg(short, long, action)]
    output: Option<PathBuf>,

    /// Defines if we should uncompress input
    #[arg(short, action)]
    uncompress: bool,

    // Disables compression operation
    #[arg(long, action)]
    no_compression: bool,

    /// Enables output of generated VSRI
    #[arg(long, action)]
    output_vsri: bool,

    /// Enables output of generated WavBrro
    #[arg(long, action)]
    output_wavbrro: bool,

    /// Enable output result of decompression in CSV format
    #[arg(long, action)]
    output_csv: bool,

    /// Select a compressor, default is auto
    #[arg(long, value_enum, default_value = "auto")]
    compressor: CompressorType,

    /// Sets the maximum allowed error for the compressed data, must be between 0 and 50. Default is 5 (5%).
    /// 0 is lossless compression
    /// 50 will do a median filter on the data.
    /// In between will pick optimize for the error
    #[arg(short, long, default_value_t = 5, value_parser = clap::value_parser ! (u8).range(0..51))]
    error: u8,

    /// Samples the input data instead of using all the data for selecting the optimal compressor.
    /// Only impacts speed, might or not increased compression ratio. For best results use 0 (default).
    /// Only works when compression = Auto.
    /// 0 will use all the data (slowest)
    /// 6 will sample 128 data points (fastest)
    #[arg(short, long, default_value_t = 0, value_parser = clap::value_parser ! (u8).range(0..7))]
    compression_selection_sample_level: u8,
}

#[derive(clap::ValueEnum, Default, Clone, Debug)]
enum CompressorType {
    #[default]
    Auto,
    Noop,
    Fft,
    Constant,
    Polynomial,
    Idw,
}

/// Compresses the data based on the provided tag and arguments.
fn compress_data(vec: &[f64], arguments: &Args) -> Vec<u8> {
    debug!("Compressing data!");
    //let optimizer_results = optimizer::process_data(vec, tag);
    // Create Optimization Plan and Stream for the data.
    let mut op = OptimizerPlan::plan(vec);
    let mut cs = CompressedStream::new();
    // Assign the compressor if it was selected
    match arguments.compressor {
        CompressorType::Noop => op.set_compressor(Compressor::Noop),
        CompressorType::Constant => op.set_compressor(Compressor::Constant),
        CompressorType::Fft => op.set_compressor(Compressor::FFT),
        CompressorType::Polynomial => op.set_compressor(Compressor::Polynomial),
        CompressorType::Idw => op.set_compressor(Compressor::Idw),
        CompressorType::Auto => op.set_compressor(Compressor::Auto),
    }
    for (cpr, data) in op.get_execution().into_iter() {
        debug!("Chunk size: {}", data.len());
        // If compressor is a losseless one, compress with the error defined, or default
        match arguments.compressor {
            CompressorType::Fft
            | CompressorType::Polynomial
            | CompressorType::Idw
            | CompressorType::Auto => cs.compress_chunk_bounded_with(
                data,
                cpr.to_owned(),
                arguments.error as f32 / 100.0,
                arguments.compression_selection_sample_level as usize,
            ),
            _ => cs.compress_chunk_with(data, cpr.to_owned()),
        }
    }
    cs.to_bytes()
}

/// Compresses the data based on the provided tag and arguments.
fn decompress_data(compressed_data: &[u8]) -> Vec<f64> {
    debug!("decompressing data!");
    let cs = CompressedStream::from_bytes(compressed_data);
    cs.decompress()
}

pub fn process_csv(args: &Args) -> Metric {
    let res = OpenOptions::new().read(true).write(true).open(&args.input);
    if res.is_err() {
        panic!(
            "[PANIC] Failed to open file {}, err: {}",
            &args.input.to_str().unwrap(),
            res.err().unwrap()
        )
    }

    let file = res.unwrap();
    let mut parser = SampleParser::new(file);
    let samples = parser.parse().expect("failed to parse samples");

    Metric::from_samples(&samples).expect("failed to create metric from samples")
}

fn process_args(args: &Args) {
    let output_base = args
        .output
        .clone()
        .unwrap_or(args.input.clone())
        .to_str()
        .unwrap()
        .to_string();

    // uncompressing input
    if args.uncompress {
        debug!("Starting uncompressing of {}", args.input.to_str().unwrap());
        if let Some(data) = read_file(&args.input).expect("failed to read bro file") {
            // decomressing data and creating wavbrro from it
            let decompressed_data = Rc::new(RefCell::new(decompress_data(&data)));
            let mut wbro = WavBrro::new();
            for data in decompressed_data.borrow().iter() {
                wbro.add_sample(*data);
            }

            // reading existing index
            let mut vsri_file_path = args.input.clone();
            vsri_file_path.set_extension("");
            let index = Vsri::load(vsri_file_path.to_str().unwrap()).expect("failed to read vsri");

            let metric = Metric::new(wbro, index);

            let mut file_path = PathBuf::from(output_base);
            file_path.set_extension("wbro");

            debug!(
                "Writing uncompressed wavbrro to disk, path: {}",
                &file_path.to_str().unwrap()
            );
            WavBrro::to_file_with_data(&file_path, &decompressed_data.borrow());

            let samples = metric.get_samples();

            // creating csv output file
            let mut csv_file_path = file_path.clone();
            csv_file_path.set_extension("csv");
            debug!(
                "Creating file for csv output at {}",
                csv_file_path.to_str().unwrap()
            );
            let csv_file = OpenOptions::new()
                .create(true)
                .write(true)
                .read(true)
                .open(&csv_file_path)
                .expect("failed to create csv output file");

            let csv_file = Rc::new(RefCell::new(csv_file));

            debug!("Writing samples into csv file");
            let writer = SampleWriter::new(Rc::clone(&csv_file));
            writer
                .write_samples(&samples)
                .expect("failed to write samples to csv")
        }
    } else {
        debug!("Starting processing of {}", args.input.to_str().unwrap());
        let metric = process_csv(args);

        if args.output_wavbrro {
            metric.flush_wavbrro(&output_base);
        }

        if args.output_vsri {
            metric.flush_indexes(&output_base);
        }

        // compressing input if no_compression is not set
        if !args.no_compression {
            debug!("Starting compressing");
            let data = metric.wbro.get_samples();
            let compressed = compress_data(&data, args);

            let mut file_path = PathBuf::from(output_base);
            file_path.set_extension("bro");

            fs::write(&file_path, compressed).expect("failed to write compressed data");
        }
    }
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    let res = fs::metadata(&args.input);
    if res.is_err() {
        panic!(
            "Failed to retrieve metadata of {}, err: {}",
            &args.input.to_str().unwrap(),
            res.err().unwrap()
        )
    }

    let metadata = res.unwrap();
    if !metadata.is_file() {
        panic!("Input is not a file")
    }

    debug!("Starting processing args");
    process_args(&args);
}
