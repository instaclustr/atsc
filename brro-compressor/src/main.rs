use std::error::Error;
use std::fs;
use brro_compressor::compressor::Compressor;
use brro_compressor::data::CompressedStream;
use brro_compressor::optimizer;
use brro_compressor::types::metric_tag::MetricTag;
use brro_compressor::utils::writer;
use clap::{arg, command, Parser};
use log::{debug, error};
use std::path::Path;
use std::path::PathBuf;
use brro_compressor::utils::reader::{StreamReader};

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
fn process_directory(arguments: &Args) -> Result<(), std::io::Error> {
    // TODO: Uncompresses directories
    let new_name = format!(
        "{}-compressed",
        arguments.input.file_name().unwrap().to_string_lossy()
    );
    let base_dir = arguments.input.with_file_name(new_name);

    writer::initialize_directory(&base_dir)?;

    let reader = StreamReader::from_directory(arguments.input.clone());

    let mut vec = reader.contents_f;
    let mut tag = reader.tags;
    let names = reader.names;

    for name in names.iter().rev() {
        let vec_data = vec.pop().unwrap();
        let tag_data = tag.pop().unwrap();

        let compressed_data = compress_data(&vec_data, &tag_data, arguments);
        // BRO extension
        let file_name = writer::replace_extension(name, "bro");
        let new_path = base_dir.join(&file_name);
        write_compressed_data_to_path(&compressed_data, &new_path)?;
    }
    Ok(())
}

/// Processes a single file.
fn process_single_file(arguments: &Args) -> Result<(), std::io::Error>  {
    debug!("Processing single file...");
    if arguments.uncompress {

        let mut reader = StreamReader::from_file(arguments.input.clone());

        let vec = reader.contents_u.remove(0);
        let slice_data: &[u8] = &vec;
        let data = decompress_data(slice_data);
        writer::write_optimal_wav(&arguments.input.to_string_lossy(), data, 1);
    } else {
        let mut reader = StreamReader::from_file(arguments.input.clone());

        let vec = reader.contents_f.remove(0);
        let tag = reader.tags.remove(0);

        let compressed_data = compress_data(&vec, &tag, arguments);

        if let Some(filename_osstr) = arguments.input.file_name() {
            if let Some(filename_str) = filename_osstr.to_str() {
                // BRO extension
                let new_filename_string =
                    writer::replace_extension(&filename_str.to_string(), "bro");
                let new_path = arguments.input.parent().unwrap().join(new_filename_string);
                write_compressed_data_to_path(&compressed_data, &new_path)?;
            }
        }
    }
    Ok(())
}

/// Compresses the data based on the provided tag and arguments.
fn compress_data(vec: &Vec<f64>, tag: &MetricTag, arguments: &Args) -> Vec<u8> {
    debug!("Compressing data!");
    let optimizer_results = optimizer::process_data(vec, tag);
    let _optimizer_results_f: Vec<f64> = optimizer_results.iter().map(|&x| x as f64).collect();

    let mut cs = CompressedStream::new();
    let compressor = match arguments.compressor {
        CompressorType::Noop => Compressor::Noop,
        CompressorType::Constant => Compressor::Constant,
        CompressorType::Fft => Compressor::FFT,
        CompressorType::Polynomial => Compressor::Polynomial,
        CompressorType::TopBottom => Compressor::TopBottom,
        CompressorType::Wavelet => Compressor::Wavelet
    };

    cs.compress_chunk_with(vec, compressor);
    cs.to_bytes()
}

/// Compresses the data based on the provided tag and arguments.
fn decompress_data(compressed_data: &[u8]) -> Vec<f64> {
    debug!("decompressing data!");
    let cs = CompressedStream::from_bytes(compressed_data);
    cs.decompress()
}

/// Writes the compressed data to the specified path.
fn write_compressed_data_to_path(compressed: &[u8], path: &Path) -> Result<(), std::io::Error>{
    let mut file =
        writer::create_streaming_writer(path)?;
    writer::write_data_to_stream(&mut file, compressed)?;
    Ok(())
}

#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// input file
    input: PathBuf,

    #[arg(long, value_enum, default_value = "noop")]
    compressor: CompressorType,

    /// Uncompresses the input file/directory
    #[arg(short, action)]
    uncompress: bool,


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
