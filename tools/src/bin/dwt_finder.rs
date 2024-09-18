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

use clap::Parser;
use dtw_rs::{Algorithm, DynamicTimeWarping, ParameterizedAlgorithm};

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

    f64::from_bits(u64_bits)
}

#[derive(Parser, Default, Debug)]
struct Arguments {
    /// First wav file
    file_one: String,
    /// Second wav file
    file_two: String,
    /// Distance
    distance: usize,
    /// Block size
    block: usize,
}

fn main() {
    let args = Arguments::parse();
    println!("{:?}", args);

    let binding_a: Vec<f64> = read_metrics_from_wav(&args.file_one);
    let binding_b: Vec<f64> = read_metrics_from_wav(&args.file_two);
    let vec_slices_a: Vec<&[f64]> = binding_a.chunks(args.block).collect();
    let vec_slices_b: Vec<&[f64]> = binding_b.chunks(args.block).collect();
    let data_a = vec_slices_a[0];
    let data_b = vec_slices_b[0];

    let param = dtw_rs::Restriction::Band(args.distance);
    let dtw = DynamicTimeWarping::with_param(data_a, data_b, param);

    println!("Path: {:?}, Distance: {}", dtw.path(), dtw.distance());
}
