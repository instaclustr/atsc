use std::error::Error;
use std::fs;
use brro_compressor::compressor::Compressor;
use brro_compressor::data::CompressedStream;
use brro_compressor::optimizer;
use brro_compressor::types::metric_tag::MetricTag;
use brro_compressor::utils::readers::{bro_reader, wav_reader};
use brro_compressor::utils::writers::{bro_writer, wav_writer};
use clap::{arg, command, Parser};
use log::{debug, error};
use std::path::{Path, PathBuf};

/// Processes the given input based on the provided arguments.
fn process_args(arguments: &Args) -> Result<(), Box<dyn Error>> {
    let metadata = fs::metadata(&arguments.input)?;

    // If the input path points to a single file
    if metadata.is_file() {
        debug!("Target is a file");
        process_single_file(arguments)?;
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
/// Processes all files in a given directory.
fn process_directory(arguments: &Args) -> Result<(), Box<dyn Error>> {
    if arguments.uncompress {
        let file_name = arguments.input.file_name().ok_or("Failed to retrieve file name.")?;
        let new_name = format!("{}-decompressed", file_name.to_string_lossy());
        let base_dir = arguments.input.with_file_name(new_name);

        wav_writer::initialize_directory(&base_dir)?;
        //read
        let files = bro_reader::dir_reader(&arguments.input)?;

        for (index, data) in files.contents.iter().enumerate() {
            let arr: &[u8] = data;
            //decompress
            let decompressed_data = decompress_data(arr);
            //write
            let new_path = base_dir.join(&files.names[index]);
            let path_str = new_path.to_str().ok_or("Invalid Unicode in file path")?;
            wav_writer::write_optimal_wav(path_str, decompressed_data, 1);
        }
        Ok(())
    } else {
        let file_name = arguments.input.file_name().ok_or("Failed to retrieve file name.")?;
        let new_name = format!("{}-compressed", file_name.to_string_lossy());
        let base_dir = arguments.input.with_file_name(new_name);

        bro_writer::initialize_directory(&base_dir)?;
        //read
        let files = wav_reader::dir_reader(&arguments.input)?;

        for (index, data) in files.contents.iter().enumerate() {
            let (vec_data, tag) = data;
            //compress
            let compressed_data = compress_data(vec_data, tag, arguments);
            // write
            write_compressed_to_path(&base_dir, &compressed_data, &files.names[index])?;
        }
        Ok(())
    }
}

/// Processes a single file.
fn process_single_file(arguments: &Args) -> Result<(), Box<dyn Error>> {
    debug!("Processing single file...");
    if arguments.uncompress {
        //read
        let vec = bro_reader::read_file(&arguments.input)?;
        let arr: &[u8] = &vec;
        //decompress
        let decompressed_data = decompress_data(arr);
        //write
        let filename_osstr = arguments.input.file_name().ok_or("Failed to get file name.")?;
        let filename_str = filename_osstr.to_str().ok_or("Failed to convert OS string to string.")?;
        let parent = arguments.input.parent().ok_or("Failed to determine parent directory.")?;
        let new_path = parent.join(filename_str);
        let path_str = new_path.to_str().ok_or("Failed to convert path to string.")?;
        wav_writer::write_optimal_wav(path_str, decompressed_data, 1);
    } else {
        //read
        let (vec, tag) = wav_reader::read_file(&arguments.input)?;
        //compress
        let compressed_data = compress_data(&vec, &tag, arguments);
        //write
        let filename_str = arguments
            .input
            .file_name()
            .and_then(|filename_osstr| filename_osstr.to_str())
            .ok_or("Failed to convert filename to string")?;
        write_compressed_to_path(&arguments.input, &compressed_data, filename_str)?;
    }
    Ok(())
}

/// Compresses the data based on the provided tag and arguments.
fn compress_data(vec: &[f64], tag: &MetricTag, arguments: &Args) -> Vec<u8> {
    debug!("Compressing data!");
    let optimizer_results = optimizer::process_data(vec, tag);
    debug!("# Samples in: {}, # Samples out: {}", vec.len(), optimizer_results.len());
    let mut cs = CompressedStream::new();
    let compressor = match arguments.compressor {
        CompressorType::Noop => Compressor::Noop,
        CompressorType::Constant => Compressor::Constant,
        CompressorType::Fft => Compressor::FFT,
        CompressorType::Polynomial => Compressor::Polynomial,
        CompressorType::TopBottom => Compressor::TopBottom,
        CompressorType::Wavelet => Compressor::Wavelet
    };

    cs.compress_chunk_with(&optimizer_results, compressor);
    cs.to_bytes()
}


/// Compresses the data based on the provided tag and arguments.
fn decompress_data(compressed_data: &[u8]) -> Vec<f64> {
    debug!("decompressing data!");
    let cs = CompressedStream::from_bytes(compressed_data);
    cs.decompress()
}
/// Writes the compressed data to the provided path
fn write_compressed_to_path(input_path: &Path, compressed_bytes: &[u8], original_filename: &str) -> Result<(), Box<dyn Error>> {
    // Use BRO extension
    let compressed_filename = bro_writer::replace_extension(&original_filename.to_string(), "bro");
    let target_directory = if input_path.is_dir() { input_path } else { input_path.parent().unwrap() };
    let output_path = target_directory.join(compressed_filename);

    let mut output_file = bro_writer::create_streaming_writer(&output_path)?;
    bro_writer::write_data_to_stream(&mut output_file, compressed_bytes)?;

    Ok(())
}
#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// input file
    input: PathBuf,

    /// Select a compressor, default is Noop
    #[arg(long, value_enum, default_value = "noop")]
    compressor: CompressorType,

     /// Sets the maximum allowed error for the compressed data, must be between 0.01 and 1. Default is 0.05 (5%).
     #[arg(long, action)]
     error: f32,

    /// Uncompresses the input file/directory
    #[arg(short, action)]
    uncompress: bool,

    /// Verbose output
    #[arg(long, action)]
    verbose: bool,
}

#[derive(clap::ValueEnum, Default, Clone, Debug)]
enum CompressorType {
    #[default]
    Noop,
    Fft,
    Wavelet,
    Constant,
    Polynomial,
    TopBottom,
}

fn main() {
    env_logger::init();
    let arguments = Args::parse();
    debug!("{:?}", arguments);

    if let Err(e) = process_args(&arguments) {
        error!("{}", e);
    }
}