use std::error::Error;
use std::fs;
use brro_compressor::compressor::Compressor;
use brro_compressor::data::CompressedStream;
use brro_compressor::optimizer;
use brro_compressor::types::metric_tag::MetricTag;
use brro_compressor::utils::reader;
use brro_compressor::utils::writer;
use clap::{arg, command, Parser};
use log::{debug, error};
use std::path::Path;
use std::path::PathBuf;

/// Processes the given input based on the provided arguments.
fn process_args(arguments: &Args) -> Result<(), Box<dyn Error>> {

    // Create a vector of boolean values from the argument flags.
    // Then, count how many of these flags are set to true.
    let count = vec![arguments.noop, arguments.constant, arguments.fft]
        .iter()
        .filter(|&&x| x)
        .count();

    if count > 1 {
        return Err("Multiple compressors are set to true. Please specify only one.".into());
    }

    let metadata = fs::metadata(&arguments.input)?;

    // If the input path points to a single file
    if metadata.is_file() {
        debug!("Target is a file");
        process_single_file(arguments);
    }
    // If the input path points to a directory
    else if metadata.is_dir() {
        debug!("Target is a directory");
        process_directory(arguments);
    }
    // If the input path is neither a file nor a directory
    else {
        return Err("The provided path is neither a file nor a directory.".into());
    }

    Ok(())
}

/// Processes all files in a given directory.
fn process_directory(arguments: &Args) {
    let new_name = format!(
        "{}-compressed",
        arguments.input.file_name().unwrap().to_string_lossy()
    );
    let base_dir = arguments.input.with_file_name(new_name);

    writer::initialize_directory(&base_dir).expect("Failed to initialize directory");
    let files =
        reader::stream_reader(&arguments.input).expect("Failed to read files from directory");

    for (index, data) in files.contents.iter().enumerate() {
        let (vec_data, tag) = data;
        let compressed_data = compress_data(vec_data, tag, arguments);

        let file_name = writer::replace_extension(&files.names[index], "bin");
        let new_path = base_dir.join(&file_name);
        write_compressed_data_to_path(&compressed_data, &new_path);
    }
}

/// Processes a single file.
fn process_single_file(arguments: &Args) {
    let (vec, tag) = reader::read_file(&arguments.input).expect("Failed to read file");
    let compressed_data = compress_data(&vec, &tag, arguments);

    if let Some(filename_osstr) = arguments.input.file_name() {
        if let Some(filename_str) = filename_osstr.to_str() {
            let new_filename_string =
                writer::replace_extension(&filename_str.to_string(), "bin");
            let new_path = arguments.input.parent().unwrap().join(new_filename_string);
            write_compressed_data_to_path(&compressed_data, &new_path);
        }
    }
}

/// Compresses the data based on the provided tag and arguments.
fn compress_data(vec: &Vec<f64>, tag: &MetricTag, arguments: &Args) -> Vec<u8> {
    let optimizer_results = optimizer::process_data(vec, tag);
    let optimizer_results_f: Vec<f64> = optimizer_results.iter().map(|&x| x as f64).collect();

    let mut cs = CompressedStream::new();
    let compressor = match arguments {
        _ if arguments.constant => Compressor::Constant,
        _ if arguments.fft => Compressor::FFT,
        _ => Compressor::Noop,
    };

    cs.compress_chunk_with(&optimizer_results_f, compressor);
    cs.to_bytes()
}

/// Writes the compressed data to the specified path.
fn write_compressed_data_to_path(compressed: &[u8], path: &Path) {
    let mut file =
        writer::create_streaming_writer(path).expect("Failed to create a streaming writer");
    writer::write_data_to_stream(&mut file, compressed).expect("Failed to write compressed data");
}

#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// input file
    input: PathBuf,

    /// Forces Noop compressor
    #[arg(long, action)]
    noop: bool,

    /// Forces Constant compressor
    #[arg(long, action)]
    constant: bool,

    /// Forces Constant compressor
    #[arg(long, action)]
    fft: bool,

}

fn main() {
    env_logger::init();
    let arguments = Args::parse();
    debug!("{:?}", arguments);

    if let Err(e) = process_args(&arguments) {
        error!("Error processing arguments: {}", e);
    }
}
