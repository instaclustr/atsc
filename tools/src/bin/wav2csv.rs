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

use clap::{command, Parser};
use csv::Writer;
use log::debug;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

// --- Legacy stuff to read brro "wav" files ---
fn is_wav_file(file_path: &Path) -> bool {
    // Open the file for reading and read the first 12 bytes (header) of the file
    let mut file = File::open(file_path).expect("Can't open file!");
    let mut header = [0u8; 12];
    file.read_exact(&mut header).expect("File too small!");

    // Check if the file starts with "RIFF" and ends with "WAVE" in the header
    header.starts_with(b"RIFF") && &header[8..12] == b"WAVE"
}

fn read_metrics_from_wav(filename: &str) -> Vec<f64> {
    let r_reader = hound::WavReader::open(filename);
    let mut reader = match r_reader {
        Ok(reader) => reader,
        Err(_err) => {
            return Vec::new();
        }
    };
    let num_channels = reader.spec().channels as usize;

    let mut raw_data: Vec<f64> = Vec::new();
    let mut u64_holder: [u16; 4] = [0, 0, 0, 0];

    // Iterate over the samples and channels and push each sample to the vector
    let mut current_channel: usize = 0;
    for sample in reader.samples::<i16>() {
        u64_holder[current_channel] = sample.unwrap() as u16;
        current_channel += 1;
        if current_channel == num_channels {
            raw_data.push(join_u16_into_f64(u64_holder));
            current_channel = 0;
        }
    }
    raw_data
}

fn join_u16_into_f64(bits: [u16; 4]) -> f64 {
    let u64_bits = (bits[0] as u64)
        | ((bits[1] as u64) << 16)
        | ((bits[2] as u64) << 32)
        | ((bits[3] as u64) << 48);

    let out = f64::from_bits(u64_bits);
    if out.is_infinite() || out.is_nan() {
        debug!("Found NaN/Infinite!");
    }
    out
}
// --- Legacy ends (I need to stop lying to myself...) ---

#[derive(Parser, Default, Debug)]
#[command(author, version, about="WAV to CSV converter", long_about = None)]
struct Args {
    /// input file
    input: PathBuf,
}

fn main() {
    env_logger::init();
    let arguments = Args::parse();
    debug!("{:?}", arguments);
    let filename = arguments.input.to_str().unwrap();
    assert!(is_wav_file(&arguments.input));
    let wav_data = read_metrics_from_wav(filename);
    // Hoping for the best
    let file = File::create(format!("{}csv", filename.strip_suffix("wav").unwrap())).unwrap();
    let mut wtr = Writer::from_writer(file);
    // Write the header
    wtr.write_record(&["timestamp", "value"]).unwrap();
    // Clean NaN
    let mut ts: i64 = 1730419200; // Nov 1st 2024
    for datapoint in wav_data.iter().filter(|&x| !x.is_nan()) {
        wtr.write_record([format!("{}", ts), format!("{}", datapoint)])
            .unwrap();
        // One sample per 20sec
        ts += 20;
    }
    // Write the file
    wtr.flush().unwrap();
}
