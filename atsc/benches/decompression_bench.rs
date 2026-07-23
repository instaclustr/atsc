use atsc::{compressor::Compressor, data::CompressedStream, optimizer::OptimizerPlan};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

const LARGE_SAMPLES: usize = 131_072;
const LOSSY_SAMPLES: usize = 8_192;
const IDW_SAMPLES: usize = 2_048;

fn constant_samples(size: usize) -> Vec<f64> {
    vec![7.0; size]
}

fn stepped_samples(size: usize) -> Vec<f64> {
    (0..size).map(|i| ((i / 256) % 16) as f64).collect()
}

fn sine_samples(size: usize) -> Vec<f64> {
    (0..size).map(|i| ((i as f64) / 8.0).sin()).collect()
}

fn compress_plan(plan: OptimizerPlan) -> CompressedStream {
    let mut stream = CompressedStream::new();
    for (compressor, samples) in plan.get_execution() {
        stream.compress_chunk_with(samples, *compressor);
    }
    stream
}

fn single_codec_stream(samples: Vec<f64>, compressor: Compressor) -> CompressedStream {
    let mut plan = OptimizerPlan::plan(&samples);
    plan.set_compressor(compressor);
    compress_plan(plan)
}

fn frame_plan(frames: Vec<(Compressor, Vec<f64>)>) -> OptimizerPlan {
    let chunk_sizes = frames.iter().map(|(_, samples)| samples.len()).collect();
    let compressors = frames.iter().map(|(compressor, _)| *compressor).collect();
    let data = frames
        .into_iter()
        .flat_map(|(_, samples)| samples)
        .collect();

    OptimizerPlan {
        data,
        chunk_sizes,
        compressors,
    }
}

fn codec_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("codec");

    let constant = single_codec_stream(constant_samples(LARGE_SAMPLES), Compressor::Constant);
    group.bench_function("constant/131072", |b| {
        b.iter(|| black_box(constant.decompress()))
    });

    let noop = single_codec_stream(stepped_samples(LARGE_SAMPLES), Compressor::Noop);
    group.bench_function("noop/131072", |b| b.iter(|| black_box(noop.decompress())));

    let rle = single_codec_stream(stepped_samples(LARGE_SAMPLES), Compressor::RLE);
    group.bench_function("rle/131072", |b| b.iter(|| black_box(rle.decompress())));

    let fft = single_codec_stream(sine_samples(LOSSY_SAMPLES), Compressor::FFT);
    group.bench_function("fft/8192", |b| b.iter(|| black_box(fft.decompress())));

    let polynomial = single_codec_stream(sine_samples(LOSSY_SAMPLES), Compressor::Polynomial);
    group.bench_function("polynomial/8192", |b| {
        b.iter(|| black_box(polynomial.decompress()))
    });

    let idw = single_codec_stream(sine_samples(IDW_SAMPLES), Compressor::Idw);
    group.bench_function("idw/2048", |b| b.iter(|| black_box(idw.decompress())));

    group.finish();
}

fn stream_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("stream");

    let mixed = compress_plan(frame_plan(vec![
        (Compressor::Constant, constant_samples(LARGE_SAMPLES)),
        (Compressor::Noop, stepped_samples(LARGE_SAMPLES)),
        (Compressor::RLE, stepped_samples(LARGE_SAMPLES)),
        (Compressor::FFT, sine_samples(LOSSY_SAMPLES)),
        (Compressor::Polynomial, sine_samples(LOSSY_SAMPLES)),
        (Compressor::Idw, sine_samples(IDW_SAMPLES)),
    ]));
    group.bench_function("mixed/6_frames", |b| {
        b.iter(|| black_box(mixed.decompress()))
    });

    let fft = compress_plan(frame_plan(vec![
        (Compressor::FFT, sine_samples(65_536)),
        (Compressor::FFT, sine_samples(65_536)),
    ]));
    group.bench_function("fft/2x65536", |b| b.iter(|| black_box(fft.decompress())));

    group.finish();
}

criterion_group!(benches, codec_benchmarks, stream_benchmarks);
criterion_main!(benches);
