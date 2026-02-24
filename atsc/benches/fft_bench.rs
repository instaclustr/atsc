use atsc::codec::fft::FftF32Codec;
use atsc::codec::{Codec, CompressConfig};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn make_signal(n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| {
            let x = i as f64;
            (x * 0.01).sin() + 0.1 * (x * 0.03).cos()
        })
        .collect()
}

fn bench_fft_compress(c: &mut Criterion) {
    let mut group = c.benchmark_group("fft_f32_compress");
    for &n in &[256usize, 1024, 4096, 65_536] {
        let data = make_signal(n);
        let cfg = CompressConfig {
            max_error: None,
            ..Default::default()
        };
        group.bench_with_input(BenchmarkId::from_parameter(n), &data, |b, d| {
            b.iter(|| {
                let _ = FftF32Codec.compress(d, &cfg).unwrap();
            })
        });
    }
    group.finish();
}

fn bench_fft_decompress(c: &mut Criterion) {
    let mut group = c.benchmark_group("fft_f32_decompress");
    for &n in &[256usize, 1024, 4096, 65_536] {
        let data = make_signal(n);
        let cfg = CompressConfig {
            max_error: None,
            ..Default::default()
        };
        let frame = FftF32Codec.compress(&data, &cfg).unwrap();
        group.bench_with_input(BenchmarkId::from_parameter(n), &frame, |b, f| {
            b.iter(|| {
                let _ = FftF32Codec.decompress(&f.payload, f.sample_count).unwrap();
            })
        });
    }
    group.finish();
}

criterion_group!(benches, bench_fft_compress, bench_fft_decompress);
criterion_main!(benches);
