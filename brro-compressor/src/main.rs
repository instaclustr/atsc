use std::{path::Path};
use clap::{Parser, command, arg};
use log::{debug, error, info};
use brro_compressor::compressor;

use brro_compressor::optimizer::optimizer;
use brro_compressor::utils::reader;
use brro_compressor::utils::writer;

fn process_args(input_path: &str, arguments: &Args) {
    let path = Path::new(input_path);

    let base_dir = Path::new("new");

    writer::initialize_directory(&base_dir).expect("Failed to initialize directory");

    //REPLACE
    let mut counter = 0;
    if arguments.directory {
        let reader_results = reader::stream_reader(path).expect("TODO: panic message");
        for data in reader_results {
            let (vec_data, tag) = data;
            let optimizer_results = optimizer::process_data(vec_data, tag);

            let iwav_data_f: Vec<f64> = optimizer_results.iter().map(|&x| x as f64).collect();

            let mut compressed: Vec<u8> = Vec::new();
            if arguments.noop {
                compressed = compressor::noop::noop(&iwav_data_f);
            } else if arguments.constant {
                compressed = compressor::constant::constant(&iwav_data_f);
            }


            let file_name = format!("compressed_{}.txt", counter);
            let new_path = base_dir.join(&file_name);
            let mut file = writer::create_streaming_writer(&new_path).expect("TODO: panic message");
            writer::write_data_to_stream(&mut file, &compressed).expect("Failed to write compressed data");
            counter += 1;
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