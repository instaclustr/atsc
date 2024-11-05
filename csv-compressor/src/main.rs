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

use crate::metric::Metric;
use brro_compressor::compressor::Compressor;
use brro_compressor::data::CompressedStream;
use brro_compressor::optimizer::OptimizerPlan;
use brro_compressor::utils::readers::bro_reader::read_file;
use clap::{arg, Parser};
use lib_vsri::vsri::Vsri;
use log::debug;
use std::fs;
use std::path::{Path, PathBuf};
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

/// process_csv opens and parses the content of file at path
pub fn process_csv(path: &Path) -> Metric {
    let samples = csv::read_samples_from_csv_file(path).expect("failed to read samples from file");
    Metric::from_samples(&samples).expect("failed to create metric from samples")
}

fn process_args(args: Args) {
    let output_base = args
        .output
        .clone()
        .unwrap_or_else(|| args.input.clone())
        .clone();

    // uncompressing input
    if args.uncompress {
        debug!("Starting uncompressing of {:?}", &args.input);
        if let Some(data) = read_file(&args.input).expect("failed to read bro file") {
            // decomressing data and creating wavbrro from it
            let decompressed_data = decompress_data(&data);
            let mut wbro = WavBrro::new();
            for data in decompressed_data.iter() {
                wbro.add_sample(*data);
            }

            // // reading existing index
            let mut vsri_file_path = args.input.clone();
            vsri_file_path.set_extension("vsri");
            debug!("Reading vsri at {:?}", &output_base);
            let index = Vsri::load(vsri_file_path.to_str().unwrap()).expect("failed to read vsri");

            let metric = Metric::new(wbro, index);

            let mut file_path = output_base.clone();
            file_path.set_extension("wbro");

            debug!("Writing uncompressed wavbrro to disk, path: {file_path:?}");
            WavBrro::to_file_with_data(&file_path, &decompressed_data);

            let samples = metric.get_samples();

            // creating csv output file
            let mut csv_file_path = file_path.clone();
            csv_file_path.set_extension("csv");
            debug!("Writing samples into csv file");
            csv::write_samples_to_csv_file(&csv_file_path, &samples)
                .expect("failed to write samples to file")
        }
    } else {
        debug!("Starting processing of {:?}", args.input);
        let metric = process_csv(&args.input);

        if args.output_wavbrro {
            let mut wavbro_file_path = output_base.clone();
            wavbro_file_path.set_extension("wavbro");
            metric.flush_wavbrro(&wavbro_file_path);
        }

        if args.output_vsri {
            let mut vsri_file_path = output_base.clone();
            vsri_file_path.set_extension("vsri");
            metric
                .flush_indexes(&vsri_file_path)
                .expect("failed to flush vsri to the file");
        }

        // compressing input if no_compression is not set
        if !args.no_compression {
            debug!("Starting compressing");
            let data = metric.wbro.get_samples();
            let compressed = compress_data(&data, &args);

            let mut file_path = output_base.clone();
            file_path.set_extension("bro");

            fs::write(&file_path, compressed).expect("failed to write compressed data");
        }
    }
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    let metadata = fs::metadata(&args.input);
    match metadata {
        Ok(metadata) => {
            if !metadata.is_file() {
                panic!("Input is not a file")
            }

            debug!("Starting processing args {:?}", &args);
            process_args(args);
        }
        Err(err) => panic!(
            "Failed to retrieve metadata of {:?}, err: {err}",
            &args.input
        ),
    }
}
