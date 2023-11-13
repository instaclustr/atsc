use criterion::{black_box, criterion_group, criterion_main, Criterion};
use brro_compressor::compressor::fft::{fft, fft_to_data, fft_allowed_error, fft_set, FFT};

// Basic FFT compression benchmark
fn fft_basic_benchmark(c: &mut Criterion) {
    let data = vec![1.0; 1024];

    c.bench_function("FFT Compression (Basic)", |b| {
        b.iter(|| {
            let compressed_data = fft(black_box(&data));
            black_box(compressed_data);
        });
    });
}

// Advanced FFT compression benchmark with custom frequency set
fn fft_advanced_benchmark(c: &mut Criterion) {
    let data = vec![1.0; 1024];

    c.bench_function("FFT Compression (Advanced)", |b| {
        b.iter(|| {
            let compressed_data = fft_set(black_box(&data), 11);
            black_box(compressed_data);
        });
    });
}

// Error-constrained FFT compression benchmark
fn fft_error_benchmark(c: &mut Criterion) {
    let data = vec![1.0; 1024];
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
    let data = vec![1.0; 1024];
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
    let data = vec![1.0; 1024];
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
    let data_small = vec![1.0; 256];
    let data_medium = vec![1.0; 1024];
    let data_large = vec![1.0; 4096];

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
    let data = vec![1.0; 1024];

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
    let data = vec![1.0; 1024];
    let rounds = 10;

    c.bench_function("Multiple Compression Rounds", |b| {
        b.iter(|| {
            let mut compressed_data = fft(black_box(&data));
            for _ in 1..rounds {
                compressed_data =
                    fft(&FFT::decompress(&compressed_data).to_data(data.len()));
            }
            black_box(compressed_data);
        });
    });
}
//Memmory usage benchmark
fn memory_usage_benchmark(c: &mut Criterion){
    let data = vec![1.0; 1024];

    c.bench_function("FFT Memory Usage", |b| {
        b.iter(|| {
            let _compressed_data = fft(black_box(&data));
        });
    });
}

// Criterion benchmark group
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
    memory_usage_benchmark,
);
criterion_main!(benches);
