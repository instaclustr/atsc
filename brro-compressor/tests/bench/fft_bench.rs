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

// FFT processing benchmark
fn fft_processing_benchmark(c: &mut Criterion) {
    let data = vec![1.0; 1024];

    c.bench_function("FFT Processing", |b| {
        b.iter(|| {
            let mut planner = FFT::new(1024, 0.0, 1.0);
            let mut buffer = FFT::optimize(&data);
            let fft = planner.get_planned_fft_forward(1024);
            fft.process(&mut buffer);
            black_box(buffer);
        });
    });
}

// Frequency point creation benchmark
fn frequency_point_creation_benchmark(c: &mut Criterion) {
    c.bench_function("Frequency Point Creation", |b| {
        b.iter(|| {
            let point = FFT::new(0.5, 0.5);
            black_box(point);
        });
    });
}

// Frequencies trimming benchmark
fn frequencies_trimming_benchmark(c: &mut Criterion) {
    let mut planner = FFT::new(1024, 0.0, 1.0);
    let mut buffer = FFT::optimize(&vec![1.0; 1024]);
    let fft = planner.get_planned_fft_forward(1024);
    fft.process(&mut buffer);
    let mut frequencies = FFT::fft_trim(&mut buffer, 10);

    c.bench_function("Frequencies Trimming", |b| {
        b.iter(|| {
            frequencies = FFT::fft_trim(&mut buffer, 10);
            black_box(&frequencies);
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

// Parallel FFT processing benchmark
fn fft_parallel_processing_benchmark(c: &mut Criterion) {
    let data = vec![1.0; 4096];

    c.bench_function("Parallel FFT Compression", |b| {
        b.iter(|| {
            let compressed_data = data
                .par_iter()
                .map(|chunk| fft(chunk))
                .collect::<Vec<Vec<u8>>>();
            black_box(compressed_data);
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

// Frequency point comparison benchmark
fn frequency_point_comparison_benchmark(c: &mut Criterion) {
    let point1 = FFT::new(0.5, 0.5);
    let point2 = FFT::new(0.6, 0.6);

    c.bench_function("Frequency Point Comparison", |b| {
        b.iter(|| {
            let _result = black_box(point1.partial_cmp(&point2));
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
    fft_processing_benchmark,
    frequency_point_creation_benchmark,
    frequencies_trimming_benchmark,
    decompression_without_fft_benchmark,
    fft_varying_data_size_benchmark,
    fft_parallel_processing_benchmark,
    compression_ratio_vs_time_benchmark,
    multiple_compression_rounds_benchmark,
    frequency_point_comparison_benchmark
);
criterion_main!(benches);
