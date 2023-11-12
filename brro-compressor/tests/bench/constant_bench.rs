use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use brro_compressor::compressor::constant::{constant, constant_to_data, Constant};
use rand::Rng;

// Function to generate random data of a given size
fn generate_random_data(size: usize) -> Vec<f64> {
    let mut rng = rand::thread_rng();
    (0..size)
        .map(|_| rng.gen_range(0.0..1.0))
        .collect()
}

// Benchmark for constant compression
fn constant_compression_benchmark(c: &mut Criterion) {
    let data_sizes = [100, 500, 1000, 5000, 10000];

    let mut group = c.benchmark_group("Constant Compression");
    group.throughput(Throughput::Elements(1));

    for size in data_sizes.iter() {
        // Generate random data
        let data = generate_random_data(*size);
        // Optimize the data for compression
        let input = Constant::optimize(&data);

        group.bench_function(format!("Compress (size={})", size), |b| {
            b.iter(|| {
                // Create a new Constant compressor and compress the data
                let mut c = Constant::new(data.len());
                c.compress(&input);
            });
        });
    }

    group.finish();
}

// Benchmark for constant decompression
fn constant_decompression_benchmark(c: &mut Criterion) {
    let data_sizes = [100, 500, 1000, 5000, 10000];

    let mut group = c.benchmark_group("Constant Decompression");
    group.throughput(Throughput::Elements(1));

    for size in data_sizes.iter() {
        // Generate random data
        let data = generate_random_data(*size);
        // Compress the data
        let compressed_data = constant(&data);

        group.bench_function(format!("Decompress (size={})", size), |b| {
            b.iter(|| {
                // Decompress the data
                let _c = constant_to_data(data.len(), &compressed_data);
            });
        });
    }

    group.finish();
}

// Benchmark for constant optimization
fn constant_optimization_benchmark(c: &mut Criterion) {
    let data_sizes = [100, 500, 1000, 5000, 10000]; // Adjust data sizes as needed

    let mut group = c.benchmark_group("Constant Optimization");
    group.throughput(Throughput::Elements(1)); // Adjust throughput unit

    for size in data_sizes.iter() {
        // Generate random data
        let data = generate_random_data(*size);

        group.bench_function(format!("Optimize (size={})", size), |b| {
            b.iter(|| {
                // Optimize the data for compression
                let _optimized_data = Constant::optimize(&data);
            });
        });
    }

    group.finish();
}

// Benchmark for comparing compression and decompression
fn constant_compression_vs_decompression_benchmark(c: &mut Criterion) {
    let data_sizes = [1000, 5000, 10000];

    let mut group = c.benchmark_group("Compression vs. Decompression");
    group.throughput(Throughput::Elements(1));

    for size in data_sizes.iter() {
        // Generate random data
        let data = generate_random_data(*size);
        // Optimize the data for compression
        let input = Constant::optimize(&data);
        // Compress the data
        let compressed_data = constant(&data);

        group.bench_function(format!("Compress (size={})", size), |b| {
            b.iter(|| {
                // Create a new Constant compressor and compress the data
                let mut c = Constant::new(data.len());
                c.compress(&input);
            });
        });

        group.bench_function(format!("Decompress (size={})", size), |b| {
            b.iter(|| {
                // Decompress the data
                let _c = constant_to_data(data.len(), &compressed_data);
            });
        });
    }

    group.finish();
}

// Benchmark for compression ratio
fn constant_compression_ratio_benchmark(c: &mut Criterion) {
    let data_sizes = [100, 500, 1000, 5000, 10000];

    let mut group = c.benchmark_group("Constant Compression Ratio");

    for size in data_sizes.iter() {
        // Generate random data
        let data = generate_random_data(*size);
        // Optimize the data for compression
        let input = Constant::optimize(&data);

        group.bench_function(format!("Compression Ratio (size={})", size), |b| {
            b.iter(|| {
                // Create a new Constant compressor and compress the data
                let mut c = Constant::new(data.len());
                c.compress(&input);
                let compressed_data = c.to_bytes();
                // Calculate compression ratio
                let _compression_ratio = input.len() as f64 / compressed_data.len() as f64;
            });
        });
    }

    group.finish();
}

// Benchmark for comparing compression and data size
fn constant_compression_vs_data_size_benchmark(c: &mut Criterion) {
    let data_sizes = [100, 500, 1000, 5000, 10000]; // Adjust data sizes as needed

    let mut group = c.benchmark_group("Compression vs. Data Size");
    group.throughput(Throughput::Elements(1));

    for size in data_sizes.iter() {
        // Generate random data
        let data = generate_random_data(*size);
        // Optimize the data for compression
        let input = Constant::optimize(&data);

        group.bench_function(format!("Compress (size={})", size), |b| {
            b.iter(|| {
                // Create a new Constant compressor and compress the data
                let mut c = Constant::new(data.len());
                c.compress(&input);
            });
        });
    }

    group.finish();
}

// Benchmark for comparing decompression and data size
fn constant_decompression_vs_data_size_benchmark(c: &mut Criterion) {
    let data_sizes = [100, 500, 1000, 5000, 10000]; // Adjust data sizes as needed

    let mut group = c.benchmark_group("Decompression vs. Data Size");
    group.throughput(Throughput::Elements(1));

    for size in data_sizes.iter() {
        // Generate random data
        let data = generate_random_data(*size);
        // Compress the data
        let compressed_data = constant(&data);

        group.bench_function(format!("Decompress (size={})", size), |b| {
            b.iter(|| {
                // Decompress the data
                let _c = constant_to_data(data.len(), &compressed_data);
            });
        });
    }

    group.finish();
}

// Benchmark for memory usage during compression and decompression
fn constant_memory_usage_benchmark(c: &mut Criterion) {
    let data_sizes = [1000, 5000, 10000]; // Adjust data sizes as needed

    let mut group = c.benchmark_group("Memory Usage");
    group.throughput(Throughput::Elements(1));

    for size in data_sizes.iter() {
        // Generate random data
        let data = generate_random_data(*size);
        // Optimize the data for compression
        let input = Constant::optimize(&data);

        group.bench_function(format!("Compression Memory (size={})", size), |b| {
            b.iter(|| {
                // Create a new Constant compressor and compress the data
                let mut c = Constant::new(data.len());
                c.compress(&input);
                // Measure memory usage here using an appropriate crate
            });
        });

        // Compress the data
        let compressed_data = constant(&data);
        group.bench_function(format!("Decompression Memory (size={})", size), |b| {
            b.iter(|| {
                // Decompress the data
                let _c = constant_to_data(data.len(), &compressed_data);
                // Measure memory usage here using an appropriate crate
            });
        });
    }

    group.finish();
}

// Benchmark for comparing compression with different numbers of residuals
fn constant_compression_vs_residuals_benchmark(c: &mut Criterion) {
    let data_sizes = [1000, 5000, 10000];
    let residuals_counts = [10, 50, 100];

    let mut group = c.benchmark_group("Compression vs. Residuals");
    group.throughput(Throughput::Elements(1));

    for size in data_sizes.iter() {
        for residuals_count in residuals_counts.iter() {
            // Generate random data with additional residuals
            let mut data = generate_random_data(*size);
            data.resize(*size + *residuals_count, 0.0);
            // Optimize the data for compression
            let input = Constant::optimize(&data);

            group.bench_function(format!("Compress (size={}, residuals={})", size, residuals_count), |b| {
                b.iter(|| {
                    // Create a new Constant compressor and compress the data
                    let mut c = Constant::new(data.len());
                    c.compress(&input);
                });
            });
        }
    }

    group.finish();
}

// Define benchmark groups
criterion_group!(
    constant_benches,
    constant_compression_benchmark,
    constant_decompression_benchmark,
    constant_optimization_benchmark,
    constant_compression_vs_decompression_benchmark,
    constant_compression_ratio_benchmark,
    constant_compression_vs_data_size_benchmark,
    constant_decompression_vs_data_size_benchmark,
    constant_memory_usage_benchmark,
    constant_compression_vs_residuals_benchmark,
);

// Run the benchmarks
criterion_main!(constant_benches);
