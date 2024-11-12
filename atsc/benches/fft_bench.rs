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

use atsc::compressor::fft::{fft, fft_allowed_error, fft_set, fft_to_data, FFT};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::PathBuf;
use wavbrro::wavbrro::WavBrro;

const TEST_WBRO_PATH: &str = "tests/wbros/go_gc_heap_goal_bytes.wbro";

/// Loads the file, and returns the Vec<f64> result
fn load_data_from_wbro_file() -> Vec<f64> {
    let test_file_path = PathBuf::from(TEST_WBRO_PATH);
    WavBrro::from_file(&test_file_path).unwrap()
}

fn pad_to_size(mut data: Vec<f64>, desired_size: usize) -> Vec<f64> {
    // Check if the data length is smaller than the desired size
    if data.len() < desired_size {
        // Add zeroes to the data until it reaches the desired size
        data.resize(desired_size, 1.0);
    } else {
        // Truncate the data to the exact size if it's larger
        data.truncate(desired_size);
    }
    data
}

/// Basic FFT compression benchmark
fn fft_basic_benchmark(c: &mut Criterion) {
    let data = load_data_from_wbro_file();

    c.bench_function("FFT Compression (Basic)", |b| {
        b.iter(|| {
            let compressed_data = fft(black_box(&data));
            black_box(compressed_data);
        });
    });
}

// Advanced FFT compression benchmark with custom frequency set
fn fft_advanced_benchmark(c: &mut Criterion) {
    let data = load_data_from_wbro_file();

    c.bench_function("FFT Compression (Advanced)", |b| {
        b.iter(|| {
            let compressed_data = fft_set(black_box(&data), 11);
            black_box(compressed_data);
        });
    });
}

// Error-constrained FFT compression benchmark
fn fft_error_benchmark(c: &mut Criterion) {
    let data = load_data_from_wbro_file();
    let max_error = 0.01;

    c.bench_function("FFT Compression (Error Constrained)", |b| {
        b.iter(|| {
            let compressed_data = fft_allowed_error(black_box(&data), max_error);
            black_box(compressed_data);
        });
    });
}

// FFT decompression benchmark
fn fft_decompression_benchmark(c: &mut Criterion) {
    let data = load_data_from_wbro_file();
    let compressed_data = fft(&data);

    c.bench_function("FFT Decompression", |b| {
        b.iter(|| {
            let decompressed_data = fft_to_data(data.len(), black_box(&compressed_data));
            black_box(decompressed_data);
        });
    });
}

// FFT initialization benchmark
fn fft_initialization_benchmark(c: &mut Criterion) {
    c.bench_function("FFT Initialization", |b| {
        b.iter(|| {
            let _planner = FFT::new(1024, 0.0, 1.0);
        });
    });
}

// Decompression without FFT benchmark
fn decompression_without_fft_benchmark(c: &mut Criterion) {
    let data = load_data_from_wbro_file();
    let compressed_data = fft(&data);

    c.bench_function("Decompression without FFT", |b| {
        b.iter(|| {
            let decompressed_data = FFT::decompress(&compressed_data).to_data(1024);
            black_box(decompressed_data);
        });
    });
}

// FFT compression benchmark with varying data sizes
fn fft_varying_data_size_benchmark(c: &mut Criterion) {
    let data = load_data_from_wbro_file();

    let data_small = pad_to_size(data.clone(), 256);
    let data_medium = pad_to_size(data.clone(), 1024);
    let data_large = pad_to_size(data, 4096);

    c.bench_function("FFT Compression (Small Data)", |b| {
        b.iter(|| {
            let _compressed_data = fft(black_box(&data_small));
        });
    });

    c.bench_function("FFT Compression (Medium Data)", |b| {
        b.iter(|| {
            let _compressed_data = fft(black_box(&data_medium));
        });
    });

    c.bench_function("FFT Compression (Large Data)", |b| {
        b.iter(|| {
            let _compressed_data = fft(black_box(&data_large));
        });
    });
}

// Compression ratio vs. time benchmark
fn compression_ratio_vs_time_benchmark(c: &mut Criterion) {
    let data = load_data_from_wbro_file();

    c.bench_function("Compression Ratio vs. Compression Time", |b| {
        b.iter(|| {
            let start = std::time::Instant::now();
            let compressed_data = fft(black_box(&data));
            let elapsed = start.elapsed();
            let compression_ratio = compressed_data.len() as f64 / data.len() as f64;
            black_box((elapsed, compression_ratio));
        });
    });
}

// Multiple compression rounds benchmark
fn multiple_compression_rounds_benchmark(c: &mut Criterion) {
    let data = load_data_from_wbro_file();
    let rounds = 10;

    c.bench_function("Multiple Compression Rounds", |b| {
        b.iter(|| {
            let mut compressed_data = fft(black_box(&data));
            for _ in 1..rounds {
                compressed_data = fft(&FFT::decompress(&compressed_data).to_data(data.len()));
            }
            black_box(compressed_data);
        });
    });
}

//Criterion benchmark group
criterion_group!(
    benches,
    fft_basic_benchmark,
    fft_advanced_benchmark,
    fft_error_benchmark,
    fft_decompression_benchmark,
    fft_initialization_benchmark,
    decompression_without_fft_benchmark,
    fft_varying_data_size_benchmark,
    compression_ratio_vs_time_benchmark,
    multiple_compression_rounds_benchmark,
);
criterion_main!(benches);
