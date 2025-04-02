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

use atsc::compressor::Compressor;
use atsc::csv::{read_samples, read_samples_with_headers};
use atsc::data::CompressedStream;
use atsc::optimizer::OptimizerPlan;
use atsc::utils::readers::bro_reader;
use clap::{arg, command, Parser};
use log::{debug, error};
use std::error::Error;
use std::path::PathBuf;
use wavbrro::wavbrro::WavBrro;

/// Processes the given input based on the provided arguments.
fn process_args(arguments: &Args) -> Result<(), Box<dyn Error>> {
    let metadata = std::fs::metadata(&arguments.input)?;

    // If the input path points to a single file
    if metadata.is_file() {
        debug!("Target is a file");
        process_single_file(arguments.input.clone(), arguments)?;
    }
    // If the input path points to a directory
    else if metadata.is_dir() {
        debug!("Target is a directory");
        process_directory(arguments)?;
    }
    // If the input path is neither a file nor a directory
    else {
        return Err("The provided path is neither a file nor a directory.".into());
    }

    Ok(())
}

fn process_directory(arguments: &Args) -> Result<(), Box<dyn Error>> {
    // Assuming you want to process each file inside this directory
    for entry in std::fs::read_dir(arguments.input.clone())? {
        let path = entry?.path();
        if path.is_file() {
            match process_single_file(path.clone(), arguments) {
                Ok(_) => continue,
                //TODO: Files are created while this walks the dir, gives a funny output
                //NOTE: Due to the way read_dir works, it seems we can't do much about this except collecting
                //      before and then iterating. But that might lead to a MASSIVE array. So it keeps a `funny` output
                //      output for the time beeing.
                Err(err) => error!("{} File: {}", err, path.display()),
            }
            // We need to make sure we skip anything but BRO and WBRO, this can be done on single file processors
            process_single_file(path, arguments)?;
        }
    }
    Ok(())
}

fn process_single_file(mut file_path: PathBuf, arguments: &Args) -> Result<(), Box<dyn Error>> {
    debug!("Processing single file...");
    if arguments.uncompress {
        //read
        if let Some(vec) = bro_reader::read_file(&file_path)? {
            let arr: &[u8] = &vec;
            //decompress
            let decompressed_data = decompress_data(arr);
            if arguments.verbose {
                println!("Output={:?}", decompressed_data);
            }
            file_path.set_extension("wbro");
            WavBrro::to_file_with_data(&file_path, &decompressed_data)
        }
    } else if arguments.csv {
        // Read samples from csv and compress it
        let samples = if arguments.no_header {
            debug!("Reading samples from csv with no header");
            read_samples(&file_path)?
        } else {
            debug!("Reading samples from csv with headers");
            let headers_option = match arguments.fields.clone() {
                Some(fields) => fields,
                None => "time, value".to_string(),
            };
            let headers: Vec<_> = headers_option.split(",").collect();
            // Assuming that header[0] is a time field and header[1] is value field
            read_samples_with_headers(&file_path, headers[0], headers[1])?
        };

        let data: Vec<f64> = samples.into_iter().map(|sample| sample.value).collect();

        if arguments.verbose {
            println!("Input={:?}", data);
        }

        // Compress
        let compressed_data = compress_data(&data, arguments);

        // Write
        file_path.set_extension("bro");
        std::fs::write(file_path, compressed_data)?;
    } else {
        // Read an WavBRRO file and compress it
        let data = WavBrro::from_file(&file_path)?;
        if arguments.verbose {
            println!("Input={:?}", data);
        }

        //compress
        let compressed_data = compress_data(&data, arguments);

        //write
        file_path.set_extension("bro");
        std::fs::write(file_path, compressed_data)?;
    }
    Ok(())
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
        CompressorType::Rle => op.set_compressor(Compressor::RLE),
        CompressorType::Fft => op.set_compressor(Compressor::FFT),
        CompressorType::Polynomial => op.set_compressor(Compressor::Polynomial),
        CompressorType::Idw => op.set_compressor(Compressor::Idw),
        CompressorType::Auto => op.set_compressor(Compressor::Auto),
    }
    for (cpr, data) in op.get_execution().into_iter() {
        debug!("Chunk size: {}", data.len());
        // If compressor is a lossy one, compress with the error defined, or default
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
            // If compressor is a lossless one, just compress
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

#[derive(Parser, Default, Debug)]
#[command(author, version, about="A Time-Series compressor", long_about = None)]
struct Args {
    input: PathBuf,

    /// Select a compressor, default is auto
    #[arg(long, value_enum, default_value = "auto")]
    compressor: CompressorType,

    /// Sets the maximum allowed error for the compressed data, must be between 0 and 50. Default is 3 (3%).
    /// 0 is lossless compression
    /// 50 will do a median filter on the data.
    /// In between will pick optimize for the error
    #[arg(short, long, default_value_t = 3, value_parser = clap::value_parser!(u8).range(0..51), verbatim_doc_comment )]
    error: u8,

    /// Uncompresses the input file/directory
    #[arg(short, action)]
    uncompress: bool,

    /// Samples the input data instead of using all the data for selecting the optimal compressor.
    /// Only impacts speed, might or not increased compression ratio. For best results use 0 (default).
    /// Only works when compression = Auto.
    /// 0 will use all the data (slowest)
    /// 6 will sample 128 data points (fastest)
    #[arg(short, long, default_value_t = 0, value_parser = clap::value_parser!(u8).range(0..7), verbatim_doc_comment )]
    compression_selection_sample_level: u8,

    /// Verbose output, dumps everysample in the input file (for compression) and in the ouput file (for decompression)
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
    /// It assumes that the one before comma is a name of time field and the one
    /// after comma is value field.
    #[arg(long, default_value = "time,value", verbatim_doc_comment)]
    fields: Option<String>,
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
    Rle,
}

fn main() {
    env_logger::init();
    let arguments = Args::parse();
    debug!("{:?}", arguments);

    if let Err(e) = process_args(&arguments) {
        error!("{}", e);
        std::process::exit(1);
    }
}
