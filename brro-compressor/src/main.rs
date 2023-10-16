/// The above code is a Rust program that compresses data based on user input and settings.
///
/// Arguments:
///
/// * `arguments`: The `arguments` parameter is a struct that contains the parsed command-line arguments. It has the following fields:
/// * `compressor_settings`: The `compressor_settings` parameter is an optional `CompressorSettings` struct that contains fields for compressor settings. It is used to customize the behavior of the compressor based on user input. If the `compressor_settings` parameter is `Some`, it means that the user has provided specific compressor settings
use brro_compressor::compressor::Compressor;
use brro_compressor::data::CompressedStream;
use brro_compressor::optimizer;
use brro_compressor::types::metric_tag::MetricTag;
use brro_compressor::utils::reader;
use brro_compressor::utils::writer;
use clap::{arg, command, Parser};
use log::debug;
use std::path::Path;
use std::path::PathBuf;

/// Define a data structure for compressor settings
struct CompressorSettings {
    // Define fields for compressor settings here
}

/// Processes the given input based on the provided arguments.
/// If `arguments.directory` is true, it processes all files in the directory.
/// Otherwise, it processes the individual file.
fn process_args(arguments: &Args, compressor_settings: Option<CompressorSettings>) {
    // If the input path points to a directory
    if arguments.directory {
        process_directory(arguments, compressor_settings);
    }
    // If the input path points to a single file
    else {
        process_single_file(arguments, compressor_settings);
    }
}

/// Processes all files in a given directory.
fn process_directory(arguments: &Args, compressor_settings: Option<CompressorSettings>) {
    let new_name = format!(
        "{}-compressed",
        arguments.input.file_name().unwrap().to_string_lossy()
    );
    let base_dir = arguments.input.with_file_name(new_name);

    writer::initialize_directory(&base_dir).expect("Failed to initialize directory");
    let files = reader::stream_reader(&arguments.input).expect("Failed to read files from directory");

    for (index, data) in files.contents.iter().enumerate() {
        let (vec_data, tag) = data;
        let compressed_data = compress_data(vec_data, tag, arguments, &compressor_settings);

        let file_name = writer::replace_extension(&files.names[index], "bin");
        let new_path = base_dir.join(&file_name);
        write_compressed_data_to_path(&compressed_data, &new_path);
    }
}

/// Processes a single file.
fn process_single_file(arguments: &Args, compressor_settings: Option<CompressorSettings>) {
    let (vec, tag) = reader::read_file(&arguments.input).expect("Failed to read file");
    let compressed_data = compress_data(&vec, &tag, arguments, &compressor_settings);

    if let Some(filename_osstr) = arguments.input.file_name() {
        if let Some(filename_str) = filename_osstr.to_str() {
            let new_filename_string = writer::replace_extension(&filename_str.to_string(), "bin");
            let new_path = arguments.input.parent().unwrap().join(new_filename_string);
            write_compressed_data_to_path(&compressed_data, &new_path);
        }
    }
}

/// Compresses the data based on the provided tag and arguments.
fn compress_data(vec: &Vec<f64>, tag: &MetricTag, arguments: &Args, compressor_settings: &Option<CompressorSettings>) -> Vec<u8> {
    let optimizer_results = optimizer::process_data(vec, tag);
    let optimizer_results_f: Vec<f64> = optimizer_results.iter().map(|&x| x as f64).collect();

    let mut cs = CompressedStream::new();
    if arguments.constant {
        cs.compress_chunk_with(&optimizer_results_f, Compressor::Constant);
        cs.to_bytes()
    } else {
        // Check if compressor settings were provided and use them
        if let Some(_settings) = compressor_settings {
            // Use the provided settings
            // You may need to replace `Compressor::Noop` with an actual compressor based on the settings
            cs.compress_chunk_with(&optimizer_results_f, Compressor::Noop);
        } else {
            // Use default settings or handle this case as needed
            cs.compress_chunk_with(&optimizer_results_f, Compressor::Noop);
        }
        cs.to_bytes()
    }
}


/// Writes the compressed data to the specified path.
fn write_compressed_data_to_path(compressed: &[u8], path: &Path) {
    let mut file = writer::create_streaming_writer(path).expect("Failed to create a streaming writer");
    writer::write_data_to_stream(&mut file, compressed).expect("Failed to write compressed data");
}

#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// input file
    input: PathBuf,

    #[arg(short, action)]
    directory: bool,

    /// Forces Noop compressor
    #[arg(long, action)]
    noop: bool,

    /// Forces Constant compressor
    #[arg(long, action)]
    constant: bool,
}

fn main() {
    env_logger::init();
    let arguments = Args::parse();
    debug!("{:?}", arguments);

    // Create an optional CompressorSettings based on user input
    let compressor_settings = if arguments.noop {
        None // Use default settings or handle this case as needed
    } else {
        Some(CompressorSettings {
            // Define settings based on user input here
        })
    };

    process_args(&arguments, compressor_settings);
}