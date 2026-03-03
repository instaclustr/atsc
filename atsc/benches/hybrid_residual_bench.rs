use atsc::codec::hybrid_residual::HybridResidualCodec;
use atsc::codec::{Codec, CompressConfig};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn make_signal(n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| {
            let x = i as f64;
            (x * 0.01).sin() + 0.25 * (x * 0.021).cos()
        })
        .collect()
}

fn bench_hybrid_compress(c: &mut Criterion) {
    let mut group = c.benchmark_group("hybrid_residual_compress");
    for &n in &[256usize, 1024, 4096, 65_536] {
        let data = make_signal(n);
        let cfg = CompressConfig {
            max_error: Some(0.01),
            ..Default::default()
        };
        group.bench_with_input(BenchmarkId::from_parameter(n), &data, |b, d| {
            b.iter(|| {
                let _ = HybridResidualCodec.compress(d, &cfg).unwrap();
            })
        });
    }
    group.finish();
}

fn bench_hybrid_decompress(c: &mut Criterion) {
    let mut group = c.benchmark_group("hybrid_residual_decompress");
    for &n in &[256usize, 1024, 4096, 65_536] {
        let data = make_signal(n);
        let cfg = CompressConfig {
            max_error: Some(0.01),
            ..Default::default()
        };
        let frame = HybridResidualCodec.compress(&data, &cfg).unwrap();
        group.bench_with_input(BenchmarkId::from_parameter(n), &frame, |b, f| {
            b.iter(|| {
                let _ = HybridResidualCodec
                    .decompress(&f.payload, f.sample_count)
                    .unwrap();
            })
        });
    }
    group.finish();
}

criterion_group!(benches, bench_hybrid_compress, bench_hybrid_decompress);
criterion_main!(benches);
