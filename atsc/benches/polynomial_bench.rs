use atsc::codec::polynomial::{IdwCodec, PolynomialCodec};
use atsc::codec::{Codec, CompressConfig};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn make_signal(n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| {
            let x = i as f64;
            (x * 0.005).sin() + 0.3 * (x * 0.013).cos()
        })
        .collect()
}

fn bench_poly_compress(c: &mut Criterion) {
    let mut group = c.benchmark_group("poly_compress");
    for &n in &[256usize, 1024, 4096, 65_536] {
        let data = make_signal(n);
        let cfg = CompressConfig {
            max_error: None,
            ..Default::default()
        };
        group.bench_with_input(BenchmarkId::from_parameter(n), &data, |b, d| {
            b.iter(|| {
                let _ = PolynomialCodec.compress(d, &cfg).unwrap();
            })
        });
    }
    group.finish();
}

fn bench_poly_decompress(c: &mut Criterion) {
    let mut group = c.benchmark_group("poly_decompress");
    for &n in &[256usize, 1024, 4096, 65_536] {
        let data = make_signal(n);
        let cfg = CompressConfig {
            max_error: None,
            ..Default::default()
        };
        let frame = PolynomialCodec.compress(&data, &cfg).unwrap();
        group.bench_with_input(BenchmarkId::from_parameter(n), &frame, |b, f| {
            b.iter(|| {
                let _ = PolynomialCodec
                    .decompress(&f.payload, f.sample_count)
                    .unwrap();
            })
        });
    }
    group.finish();
}

fn bench_idw_compress(c: &mut Criterion) {
    let mut group = c.benchmark_group("idw_compress");
    for &n in &[256usize, 1024, 4096, 65_536] {
        let data = make_signal(n);
        let cfg = CompressConfig {
            max_error: None,
            ..Default::default()
        };
        group.bench_with_input(BenchmarkId::from_parameter(n), &data, |b, d| {
            b.iter(|| {
                let _ = IdwCodec.compress(d, &cfg).unwrap();
            })
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_poly_compress,
    bench_poly_decompress,
    bench_idw_compress
);
criterion_main!(benches);
