use std::{fs::File, path::Path};
use std::io::Write;
use std::fs;
use std::path::PathBuf;
use clap::{Parser, command, arg};
use log::{debug, error, info};
use brro_compressor::compressor;

use brro_compressor::optimizer::optimizer;

fn process_args(input_path: &str, arguments: &Args) {
    if arguments.directory {
        handle_directory(input_path, arguments);
    } else {
        process_file(input_path.into(), arguments, None);
    }
}

fn handle_directory(input_path: &str, arguments: &Args) {
    let new_directory = format!("new_{}", input_path);

    if fs::create_dir_all(&new_directory).is_err() {
        error!("Unable to create directory: {}", new_directory);
        return;
    }

    if let Ok(entries) = fs::read_dir(input_path) {
        for entry_result in entries {
            match entry_result {
                Ok(entry) if entry.path().is_file() => {
                    process_file(entry.path(), arguments, Some(&new_directory));
                }
                Err(e) => error!("Error reading directory entry: {}", e),
                _ => {}
            }
        }
    } else {
        error!("Error reading directory: {}", input_path);
    }
}

fn process_file(full_path: PathBuf, arguments: &Args, new_directory: Option<&str>) {
    if let Some(filename) = full_path.file_name().and_then(|s| s.to_str()) {
        let iwav_data = optimizer::process_data_and_write_output(&full_path);

        // print!("{:?}", iwav_data);

        let iwav_data_f: Vec<f64> = iwav_data.iter().map(|&x| x as f64).collect();

        let mut result: Vec<u8> = Vec::new();
        if arguments.noop {
             result = compressor::noop::noop(&iwav_data_f);
        } else if arguments.constant {
             result = compressor::constant::constant(&iwav_data_f);
        }


        let output_path = construct_output_path(filename, new_directory);
        let mut file = match File::create(&output_path) {
            Ok(file) => file,
            Err(_) => {
                error!("Unable to create file: {}", output_path);
                return;
            }
        };
        writeln!(file, "{:?}", result).expect("Unable to write to file");
    }
}

fn construct_output_path(filename: &str, new_directory: Option<&str>) -> String {
    match new_directory {
        Some(dir) => format!("{}/new_{}.txt", dir, filename),
        None => format!("new_{}.txt", filename),
    }
}

#[derive(Parser, Default, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// input file
    input: String,

    /// Write a new file with optimized settings, named filename_OPT.wav
    #[arg(short)]
    write: bool,

    #[arg(short, action)]
    directory: bool,

    /// Samplerate to generate the optimized file
    #[arg(short, long)]
    samplerate: Option<u32>,

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