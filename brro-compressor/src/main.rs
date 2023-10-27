use brro_compressor::compressor::Compressor;
use brro_compressor::data::CompressedStream;
use brro_compressor::optimizer::OptimizerPlan;
use brro_compressor::types::metric_tag::MetricTag;
use brro_compressor::utils::readers::{bro_reader, wav_reader};
use brro_compressor::utils::writers::wav_writer;
use clap::{arg, command, Parser};
use log::{debug, error};
use std::error::Error;
use std::path::{Path, PathBuf};

/// Processes the given input based on the provided arguments.
fn process_args(arguments: &Args) -> Result<(), Box<dyn Error>> {
    let metadata = std::fs::metadata(&arguments.input)?;

    // If the input path points to a single file
    if metadata.is_file() {
        debug!("Target is a file");
        process_single_file(&arguments.input, arguments)?;
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
        let file_name = arguments
            .input
            .file_name()
            .ok_or("Failed to retrieve file name.")?;
        let new_name = format!("{}-decompressed", file_name.to_string_lossy());
        let base_dir = arguments.input.with_file_name(new_name);

        std::fs::create_dir_all(&base_dir)?;
        //read

        // TODO: This should be calling `process_single_file` and avoid code duplication
        for file in bro_reader::dir_reader(&arguments.input)? {
            //decompress
            let decompressed_data = decompress_data(&file.contents);
            //write
            let path = base_dir.join(
                file.original_path
                    .file_name()
                    .ok_or("path has no file name")?,
            );
            // TODO: Decompression shouldn't optimize the WAV
            wav_writer::write_optimal_wav(path, decompressed_data, 1);
        }
        Ok(())
    } else {
        let file_name = arguments
            .input
            .file_name()
            .ok_or("Failed to retrieve file name.")?;
        let new_name = format!("{}-compressed", file_name.to_string_lossy());
        let base_dir = arguments.input.with_file_name(new_name);

        std::fs::create_dir_all(&base_dir)?;

        //read
        for file in wav_reader::dir_reader(&arguments.input)? {
            let compressed_data = compress_data(&file.contents, &file.tag, arguments);
            let mut path = base_dir.join(
                file.original_path
                    .file_name()
                    .ok_or("path has no file name")?,
            );
            path.set_extension("bro");
            std::fs::write(path, compressed_data)?;
        }
        Ok(())
    }
}

/// Processes a single file.
fn process_single_file(file_path: &Path, arguments: &Args) -> Result<(), Box<dyn Error>> {
    debug!("Processing single file...");
    if arguments.uncompress {
        //read
        let vec = bro_reader::read_file(&arguments.input)?;
        let arr: &[u8] = &vec;
        //decompress
        let decompressed_data = decompress_data(arr);
        if arguments.verbose {
            println!("Output={:?}", decompressed_data);
        }
        wav_writer::write_optimal_wav(arguments.input.clone(), decompressed_data, 1);
    } else {
        //read
        let (vec, tag) = wav_reader::read_file(file_path)?;
        if arguments.verbose {
            println!("Input={:?}", vec);
        }
        //compress
        let compressed_data = compress_data(&vec, &tag, arguments);

        //write
        let mut path = arguments.input.clone();
        path.set_extension("bro");
        std::fs::write(path, compressed_data)?;
    }
    Ok(())
}

/// Compresses the data based on the provided tag and arguments.
fn compress_data(vec: &[f64], _tag: &MetricTag, arguments: &Args) -> Vec<u8> {
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
        CompressorType::TopBottom => op.set_compressor(Compressor::TopBottom),
        CompressorType::Wavelet => op.set_compressor(Compressor::Wavelet),
        _ => todo!("Auto selection of compressor not yet implemented!"),
    }
    for (cpr, data) in op.get_execution().into_iter() {
        debug!("Chunk size: {}", data.len());
        // If compressor is a losseless one, compress with the error defined, or default
        match arguments.compressor {
            CompressorType::Fft => {
                cs.compress_chunk_bounded_with(data, cpr.to_owned(), arguments.error as f32 / 100.0)
            }
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
    /// input file
    input: PathBuf,

    /// Select a compressor, default is auto
    #[arg(long, value_enum, default_value = "auto")]
    compressor: CompressorType,

    /// Sets the maximum allowed error for the compressed data, must be between 0 and 50. Default is 5 (5%).
    /// 0 is lossless compression
    /// 50 will do a median filter on the data.
    /// In between will pick optimize for the error
    #[arg(short, long, default_value_t = 5, value_parser = clap::value_parser!(u8).range(0..51))]
    error: u8,

    /// Uncompresses the input file/directory
    #[arg(short, action)]
    uncompress: bool,

    /// Verbose output, dumps everysample in the input file (for compression) and in the ouput file (for decompression)
    #[arg(long, action)]
    verbose: bool,
}

#[derive(clap::ValueEnum, Default, Clone, Debug)]
enum CompressorType {
    #[default]
    Auto,
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
