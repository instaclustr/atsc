use criterion::{criterion_group, criterion_main, Criterion};
use brro_compressor::compressor::noop::{Noop, noop};

/// Benchmark for the compression operation using the Noop algorithm.
fn noop_compression_benchmark(c: &mut Criterion) {
    // Generate sample data
    let data = vec![1.0; 1000];
    let mut group = c.benchmark_group("Noop Compression");

    group.bench_function("Compress", |b| {
        b.iter(|| {
            let _compressed_data = noop(&data);
        });
    });

    group.finish();
}

/// Benchmark for the decompression operation using the Noop algorithm.
fn noop_decompression_benchmark(c: &mut Criterion) {
    // Generate sample data
    let data = vec![1.0; 1000];
    // Compress the data using Noop
    let compressed_data = noop(&data);

    c.bench_function("Decompress", |b| {
        b.iter(|| {
            let _decompressed_data = Noop::decompress(&compressed_data);
        });
    });
}

/// Benchmark for the optimization operation using the Noop algorithm.
fn noop_optimize_benchmark(c: &mut Criterion) {
    // Generate sample data
    let data = vec![1.0; 1000];
    let mut group = c.benchmark_group("Noop Optimize");

    group.bench_function("Optimize", |b| {
        b.iter(|| {
            let _optimized_data = Noop::optimize(&data);
        });
    });

    group.finish();
}

/// Benchmark for the serialization operation using the Noop algorithm.
fn noop_serialization_benchmark(c: &mut Criterion) {
    // Generate sample data
    let data = vec![1.0; 1000];
    // Initialize Noop with the data length
    let nooper = Noop::new(data.len());

    c.bench_function("Serialization", |b| {
        b.iter(|| {
            let _serialized_data = nooper.to_bytes();
        });
    });
}

/// Benchmark for the data extraction operation using the Noop algorithm.
fn noop_data_extraction_benchmark(c: &mut Criterion) {
    // Generate sample data
    let data = vec![1.0; 1000];
    // Initialize Noop with the data length and compress the data
    let nooper = Noop::new(data.len());
    nooper.compress(&data);
    let compressed_data = nooper.to_bytes();

    c.bench_function("Data Extraction", |b| {
        b.iter(|| {
            let _extracted_data = Noop::to_data(&nooper, data.len());
        });
    });
}

// Group all Noop benchmarks
criterion_group!(
    noop_benches,
    noop_compression_benchmark,
    noop_decompression_benchmark,
    noop_optimize_benchmark,
    noop_serialization_benchmark,
    noop_data_extraction_benchmark
);

// Run all benchmarks
criterion_main!(noop_benches);
