// Import necessary libraries
use criterion::{criterion_group, criterion_main, Criterion, black_box};
use brro_compressor::compressor::polynomial::{polynomial, polynomial_allowed_error, to_data, PolynomialType};

// Define example sample data
const SAMPLE_DATA: &[f64] = &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

// Define the benchmark functions

// Benchmark for the speed of polynomial compression
fn polynomial_compress_speed_benchmark(c: &mut Criterion) {
    c.bench_function("polynomial_compress_speed", |b| {
        b.iter(|| {
            let _compressed_data = black_box(polynomial(SAMPLE_DATA, PolynomialType::Polynomial));
        });
    });
}

// Benchmark for the speed of polynomial compression with bounded error
fn polynomial_compress_bounded_speed_benchmark(c: &mut Criterion) {
    c.bench_function("polynomial_compress_bounded_speed", |b| {
        b.iter(|| {
            let _compressed_data = black_box(polynomial_allowed_error(SAMPLE_DATA, 0.5, PolynomialType::Polynomial));
        });
    });
}

// Benchmark for the speed of decompression and conversion to data
fn decompress_and_to_data_speed_benchmark(c: &mut Criterion) {
    // Compress the data once to use in decompression benchmark
    let compressed_data = polynomial(SAMPLE_DATA, PolynomialType::Polynomial);

    c.bench_function("decompress_and_to_data_speed", |b| {
        b.iter(|| {
            let _result = black_box(to_data(SAMPLE_DATA.len(), &compressed_data));
        });
    });
}

// Benchmark for the speed of polynomial compression with IDW type
fn polynomial_compress_idw_speed_benchmark(c: &mut Criterion) {
    c.bench_function("polynomial_compress_idw_speed", |b| {
        b.iter(|| {
            let _compressed_data = black_box(polynomial(SAMPLE_DATA, PolynomialType::Idw));
        });
    });
}

// Benchmark for the speed of polynomial compression with IDW type and bounded error
fn polynomial_compress_bounded_idw_speed_benchmark(c: &mut Criterion) {
    c.bench_function("polynomial_compress_bounded_idw_speed", |b| {
        b.iter(|| {
            let _compressed_data = black_box(polynomial_allowed_error(SAMPLE_DATA, 0.5, PolynomialType::Idw));
        });
    });
}

// Benchmark for the speed of decompression and conversion to data with IDW type
fn decompress_and_to_data_idw_speed_benchmark(c: &mut Criterion) {
    // Compress the data once to use in decompression benchmark
    let compressed_data = polynomial(SAMPLE_DATA, PolynomialType::Idw);

    c.bench_function("decompress_and_to_data_idw_speed", |b| {
        b.iter(|| {
            let _result = black_box(to_data(SAMPLE_DATA.len(), &compressed_data));
        });
    });
}

// Benchmark for the memory usage of polynomial compression
fn polynomial_compress_memory_benchmark(c: &mut Criterion) {
    c.bench_function("polynomial_compress_memory", |b| {
        b.iter(|| {
            let _compressed_data = polynomial(SAMPLE_DATA, PolynomialType::Polynomial);
        });
    });
}

// Benchmark for the memory usage of polynomial compression with bounded error
fn polynomial_compress_bounded_memory_benchmark(c: &mut Criterion) {
    c.bench_function("polynomial_compress_bounded_memory", |b| {
        b.iter(|| {
            let _compressed_data = polynomial_allowed_error(SAMPLE_DATA, 0.5, PolynomialType::Polynomial);
        });
    });
}

// Benchmark for the memory usage of decompression and conversion to data
fn decompress_and_to_data_memory_benchmark(c: &mut Criterion) {
    // Compress the data once to use in decompression benchmark
    let compressed_data = polynomial(SAMPLE_DATA, PolynomialType::Polynomial);

    c.bench_function("decompress_and_to_data_memory", |b| {
        b.iter(|| {
            let _result = to_data(SAMPLE_DATA.len(), &compressed_data);
        });
    });
}

// Benchmark for the memory usage of polynomial compression with IDW type
fn polynomial_compress_idw_memory_benchmark(c: &mut Criterion) {
    c.bench_function("polynomial_compress_idw_memory", |b| {
        b.iter(|| {
            let _compressed_data = polynomial(SAMPLE_DATA, PolynomialType::Idw);
        });
    });
}

// Benchmark for the memory usage of polynomial compression with IDW type and bounded error
fn polynomial_compress_bounded_idw_memory_benchmark(c: &mut Criterion) {
    c.bench_function("polynomial_compress_bounded_idw_memory", |b| {
        b.iter(|| {
            let _compressed_data = polynomial_allowed_error(SAMPLE_DATA, 0.5, PolynomialType::Idw);
        });
    });
}

// Benchmark for the memory usage of decompression and conversion to data with IDW type
fn decompress_and_to_data_idw_memory_benchmark(c: &mut Criterion) {
    // Compress the data once to use in decompression benchmark
    let compressed_data = polynomial(SAMPLE_DATA, PolynomialType::Idw);

    c.bench_function("decompress_and_to_data_idw_memory", |b| {
        b.iter(|| {
            let _result = to_data(SAMPLE_DATA.len(), &compressed_data);
        });
    });
}

// Benchmark for the speed of IDW compression
fn idw_compress_speed_benchmark(c: &mut Criterion) {
    c.bench_function("idw_compress_speed", |b| {
        b.iter(|| {
            let _compressed_data = black_box(polynomial(SAMPLE_DATA, PolynomialType::Idw));
        });
    });
}

// Benchmark for the speed of IDW compression with bounded error
fn idw_compress_bounded_speed_benchmark(c: &mut Criterion) {
    c.bench_function("idw_compress_bounded_speed", |b| {
        b.iter(|| {
            let _compressed_data = black_box(polynomial_allowed_error(SAMPLE_DATA, 0.5, PolynomialType::Idw));
        });
    });
}

// Benchmark for the memory usage of IDW compression
fn idw_compress_memory_benchmark(c: &mut Criterion) {
    c.bench_function("idw_compress_memory", |b| {
        b.iter(|| {
            let _compressed_data = polynomial(SAMPLE_DATA, PolynomialType::Idw);
        });
    });
}

// Benchmark for the memory usage of IDW compression with bounded error
fn idw_compress_bounded_memory_benchmark(c: &mut Criterion) {
    c.bench_function("idw_compress_bounded_memory", |b| {
        b.iter(|| {
            let _compressed_data = polynomial_allowed_error(SAMPLE_DATA, 0.5, PolynomialType::Idw);
        });
    });
}

// Add the benchmark group to criterion
criterion_group!(
    benches,
    polynomial_compress_speed_benchmark,
    polynomial_compress_bounded_speed_benchmark,
    decompress_and_to_data_speed_benchmark,
    polynomial_compress_idw_speed_benchmark,
    polynomial_compress_bounded_idw_speed_benchmark,
    decompress_and_to_data_idw_speed_benchmark,
    polynomial_compress_memory_benchmark,
    polynomial_compress_bounded_memory_benchmark,
    decompress_and_to_data_memory_benchmark,
    polynomial_compress_idw_memory_benchmark,
    polynomial_compress_bounded_idw_memory_benchmark,
    decompress_and_to_data_idw_memory_benchmark,
    idw_compress_speed_benchmark,
    idw_compress_bounded_speed_benchmark,
    idw_compress_memory_benchmark,
    idw_compress_bounded_memory_benchmark
);

// Run all benchmarks in the group
criterion_main!(benches);
