use std::path::Path;
use clap::{Parser, command, arg};
use log::debug;
use brro_compressor::compressor::Compressor;
use brro_compressor::optimizer;
use brro_compressor::utils::reader;
use brro_compressor::utils::writer;
use brro_compressor::data::CompressedStream;
use brro_compressor::types::metric_tag::MetricTag;

/// Processes the given input based on the provided arguments.
/// If `arguments.directory` is true, it processes all files in the directory.
/// Otherwise, it processes the individual file.
fn process_args(input_path: &str, arguments: &Args) {
    let path = Path::new(input_path);

    // If the input path points to a directory
    if arguments.directory {
        process_directory(path, arguments);
    }
    // If the input path points to a single file
    else {
        process_single_file(path, arguments);
    }
}

/// Processes all files in a given directory.
fn process_directory(path: &Path, arguments: &Args) {
    let new_name = format!("{}-compressed", path.file_name().unwrap().to_string_lossy());
    let base_dir = path.with_file_name(new_name);

    writer::initialize_directory(&base_dir).expect("Failed to initialize directory");
    let files = reader::stream_reader(path).expect("Failed to read files from directory");

    for (index, data) in files.contents.iter().enumerate() {
        let (vec_data, tag) = data;
        let compressed_data = compress_data(vec_data, tag, arguments);

        let file_name = writer::replace_extension(&files.names[index], "bin");
        let new_path = base_dir.join(&file_name);
        write_compressed_data_to_path(&compressed_data, &new_path);
    }
}

/// Processes a single file.
fn process_single_file(path: &Path, arguments: &Args) {
    if let Some((vec, tag)) = reader::read_file(path).expect("Failed to read file") {
        let compressed_data = compress_data(&vec, &tag, arguments);

        if let Some(filename_osstr) = path.file_name() {
            if let Some(filename_str) = filename_osstr.to_str() {
                let new_filename_string = writer::replace_extension(&filename_str.to_string(),"bin");
                let new_path = path.parent().unwrap().join(new_filename_string);
                write_compressed_data_to_path(&compressed_data, &new_path);
            }
        }
    }
}

/// Compresses the data based on the provided tag and arguments.
fn compress_data(vec: &Vec<f64>, tag: &MetricTag, arguments: &Args) -> Vec<u8> {
    let optimizer_results = optimizer::process_data(vec, tag);
    let optimizer_results_f: Vec<f64> = optimizer_results.iter().map(|&x| x as f64).collect();

    let cs = CompressedStream::new();
    if arguments.constant {
        compress_file(Compressor::Constant, cs, optimizer_results_f)
    } else {
        compress_file(Compressor::Noop, cs, optimizer_results_f)
    }
}
fn compress_file(compressor: Compressor, mut stream: CompressedStream, wav_content: Vec<f64>) -> Vec<u8> {
    return match compressor {
        Compressor::Constant =>{
            stream.compress_chunk_with(&wav_content, Compressor::Constant);
            stream.to_bytes()
        }
        _ =>{
            stream.compress_chunk_with(&wav_content, Compressor::Noop);
            stream.to_bytes()
        }
    }
}
/// Writes the compressed data to the specified path.
fn write_compressed_data_to_path(compressed: &Vec<u8>, path: &Path) {
    let mut file = writer::create_streaming_writer(path).expect("Failed to create a streaming writer");
    writer::write_data_to_stream(&mut file, compressed).expect("Failed to write compressed data");
}


#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// input file
    input: String,

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
    process_args(&arguments.input, &arguments);
}