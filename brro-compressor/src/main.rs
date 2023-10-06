use std::path::Path;
use clap::{Parser, command, arg};
use log::debug;
use brro_compressor::compressor;

use brro_compressor::optimizer;
use brro_compressor::utils::reader;
use brro_compressor::utils::writer;

fn process_args(input_path: &str, arguments: &Args) {
    let path = Path::new(input_path);

    let new_name = format!("{}-compressed", path.file_name().unwrap().to_string_lossy());
    let base_dir = path.with_file_name(new_name);

    writer::initialize_directory(&base_dir).expect("Failed to initialize directory");

    if arguments.directory {
        let files = reader::stream_reader(path).expect("TODO: panic message");
        for (index, data) in files.contents.iter().enumerate() {
            let (vec_data, tag) = data;
            let optimizer_results = processors::process_data(vec_data, tag);

            let optimizer_results_f: Vec<f64> = optimizer_results.iter().map(|&x| x as f64).collect();

            let mut compressed: Vec<u8> = Vec::new();
            if arguments.noop {
                compressed = compressor::noop::noop(&optimizer_results_f);
            } else if arguments.constant {
                compressed = compressor::constant::constant(&optimizer_results_f);
            }

            let file_name =  writer::replace_extension(&files.names[index], "txt)");
            let new_path = base_dir.join(&file_name);
            let mut file = writer::create_streaming_writer(&new_path).expect("TODO: panic message");
            writer::write_data_to_stream(&mut file, &compressed).expect("Failed to write compressed data");
        }
    } else {
        // process_file(input_path.into());
    }
}

#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// input file
    input: String,

    #[arg(short, action)]
    directory: bool,

    /// Write optimized samples to a file, named as optimized.out
    #[arg(long, action)]
    noop: bool,

    /// Write optimized samples to a file, named as optimized.out
    #[arg(long, action)]
    constant: bool,

}

fn main() {
    // How to break the float part??? --> THERE ARE NO FLOATS!
    // https://access.redhat.com/documentation/en-us/red_hat_enterprise_linux/6/html/deployment_guide/s2-proc-stat
    env_logger::init();
    let arguments = Args::parse();
    debug!("{:?}", arguments);
    process_args(&arguments.input, &arguments);
}