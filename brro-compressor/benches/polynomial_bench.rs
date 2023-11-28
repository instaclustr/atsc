use criterion::{criterion_group, criterion_main, Criterion};
use brro_compressor::compressor::polynomial::{polynomial, polynomial_allowed_error, to_data, PolynomialType};

// Define example sample data
const SAMPLE_DATA: &[f64] = &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

// Define the benchmark functions

fn polynomial_compress_speed_benchmark(c: &mut Criterion) {
    c.bench_function("polynomial_compress_speed", |b| {
        b.iter(|| {
            let _compressed_data = polynomial(SAMPLE_DATA, PolynomialType::Polynomial);
        });
    });
}

fn polynomial_compress_bounded_speed_benchmark(c: &mut Criterion) {
    c.bench_function("polynomial_compress_bounded_speed", |b| {
        b.iter(|| {
            let _compressed_data = polynomial_allowed_error(SAMPLE_DATA, 0.5, PolynomialType::Polynomial);
        });
    });
}

fn decompress_and_to_data_speed_benchmark(c: &mut Criterion) {
    // Compress the data once to use in decompression benchmark
    let compressed_data = polynomial(SAMPLE_DATA, PolynomialType::Polynomial);

    c.bench_function("decompress_and_to_data_speed", |b| {
        b.iter(|| {
            let _result = to_data(SAMPLE_DATA.len(), &compressed_data);
        });
    });
}

fn polynomial_compress_idw_speed_benchmark(c: &mut Criterion) {
    c.bench_function("polynomial_compress_idw_speed", |b| {
        b.iter(|| {
            let _compressed_data = polynomial(SAMPLE_DATA, PolynomialType::Idw);
        });
    });
}

fn polynomial_compress_bounded_idw_speed_benchmark(c: &mut Criterion) {
    c.bench_function("polynomial_compress_bounded_idw_speed", |b| {
        b.iter(|| {
            let _compressed_data = polynomial_allowed_error(SAMPLE_DATA, 0.5, PolynomialType::Idw);
        });
    });
}

fn decompress_and_to_data_idw_speed_benchmark(c: &mut Criterion) {
    // Compress the data once to use in decompression benchmark
    let compressed_data = polynomial(SAMPLE_DATA, PolynomialType::Idw);

    c.bench_function("decompress_and_to_data_idw_speed", |b| {
        b.iter(|| {
            let _result = to_data(SAMPLE_DATA.len(), &compressed_data);
        });
    });
}

fn polynomial_compress_memory_benchmark(c: &mut Criterion) {
    c.bench_function("polynomial_compress_memory", |b| {
        b.iter(|| {
            let _compressed_data = polynomial(SAMPLE_DATA, PolynomialType::Polynomial);
        });
    });
}

fn polynomial_compress_bounded_memory_benchmark(c: &mut Criterion) {
    c.bench_function("polynomial_compress_bounded_memory", |b| {
        b.iter(|| {
            let _compressed_data = polynomial_allowed_error(SAMPLE_DATA, 0.5, PolynomialType::Polynomial);
        });
    });
}

fn decompress_and_to_data_memory_benchmark(c: &mut Criterion) {
    // Compress the data once to use in decompression benchmark
    let compressed_data = polynomial(SAMPLE_DATA, PolynomialType::Polynomial);

    c.bench_function("decompress_and_to_data_memory", |b| {
        b.iter(|| {
            let _result = to_data(SAMPLE_DATA.len(), &compressed_data);
        });
    });
}

fn polynomial_compress_idw_memory_benchmark(c: &mut Criterion) {
    c.bench_function("polynomial_compress_idw_memory", |b| {
        b.iter(|| {
            let _compressed_data = polynomial(SAMPLE_DATA, PolynomialType::Idw);
        });
    });
}

fn polynomial_compress_bounded_idw_memory_benchmark(c: &mut Criterion) {
    c.bench_function("polynomial_compress_bounded_idw_memory", |b| {
        b.iter(|| {
            let _compressed_data = polynomial_allowed_error(SAMPLE_DATA, 0.5, PolynomialType::Idw);
        });
    });
}

fn decompress_and_to_data_idw_memory_benchmark(c: &mut Criterion) {
    // Compress the data once to use in decompression benchmark
    let compressed_data = polynomial(SAMPLE_DATA, PolynomialType::Idw);

    c.bench_function("decompress_and_to_data_idw_memory", |b| {
        b.iter(|| {
            let _result = to_data(SAMPLE_DATA.len(), &compressed_data);
        });
    });
}

fn idw_compress_speed_benchmark(c: &mut Criterion) {
    c.bench_function("idw_compress_speed", |b| {
        b.iter(|| {
            let _compressed_data = polynomial(SAMPLE_DATA, PolynomialType::Idw);
        });
    });
}

fn idw_compress_bounded_speed_benchmark(c: &mut Criterion) {
    c.bench_function("idw_compress_bounded_speed", |b| {
        b.iter(|| {
            let _compressed_data = polynomial_allowed_error(SAMPLE_DATA, 0.5, PolynomialType::Idw);
        });
    });
}

fn idw_compress_memory_benchmark(c: &mut Criterion) {
    c.bench_function("idw_compress_memory", |b| {
        b.iter(|| {
            let _compressed_data = polynomial(SAMPLE_DATA, PolynomialType::Idw);
        });
    });
}

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
