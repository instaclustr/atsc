use atsc::{
    compressor::Compressor, data::CompressedStream, decoder::Decoder, optimizer::OptimizerPlan,
};
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
    let mut constant_decoder = Decoder::new();
    let mut constant_output = Vec::new();
    group.bench_function("decoder/reused/constant/131072", |b| {
        b.iter(|| {
            constant_output.clear();
            let _ = black_box(
                constant_decoder
                    .decode_into(&constant, &mut constant_output)
                    .expect("constant stream must decode"),
            );
            let _ = black_box(constant_output.as_slice());
        })
    });

    let noop = single_codec_stream(stepped_samples(LARGE_SAMPLES), Compressor::Noop);
    group.bench_function("noop/131072", |b| b.iter(|| black_box(noop.decompress())));
    let mut noop_decoder = Decoder::new();
    let mut noop_output = Vec::new();
    group.bench_function("decoder/reused/noop/131072", |b| {
        b.iter(|| {
            noop_output.clear();
            let _ = black_box(
                noop_decoder
                    .decode_into(&noop, &mut noop_output)
                    .expect("noop stream must decode"),
            );
            let _ = black_box(noop_output.as_slice());
        })
    });

    let rle = single_codec_stream(stepped_samples(LARGE_SAMPLES), Compressor::RLE);
    group.bench_function("rle/131072", |b| b.iter(|| black_box(rle.decompress())));
    let mut rle_decoder = Decoder::new();
    let mut rle_output = Vec::new();
    group.bench_function("decoder/reused/rle/131072", |b| {
        b.iter(|| {
            rle_output.clear();
            let _ = black_box(
                rle_decoder
                    .decode_into(&rle, &mut rle_output)
                    .expect("RLE stream must decode"),
            );
            let _ = black_box(rle_output.as_slice());
        })
    });

    let fft = single_codec_stream(sine_samples(LOSSY_SAMPLES), Compressor::FFT);
    group.bench_function("fft/8192", |b| b.iter(|| black_box(fft.decompress())));
    let mut fft_decoder = Decoder::new();
    let mut fft_output = Vec::new();
    group.bench_function("decoder/reused/fft/8192", |b| {
        b.iter(|| {
            fft_output.clear();
            let _ = black_box(
                fft_decoder
                    .decode_into(&fft, &mut fft_output)
                    .expect("FFT stream must decode"),
            );
            let _ = black_box(fft_output.as_slice());
        })
    });

    let polynomial = single_codec_stream(sine_samples(LOSSY_SAMPLES), Compressor::Polynomial);
    group.bench_function("polynomial/8192", |b| {
        b.iter(|| black_box(polynomial.decompress()))
    });
    let mut polynomial_decoder = Decoder::new();
    let mut polynomial_output = Vec::new();
    group.bench_function("decoder/reused/polynomial/8192", |b| {
        b.iter(|| {
            polynomial_output.clear();
            let _ = black_box(
                polynomial_decoder
                    .decode_into(&polynomial, &mut polynomial_output)
                    .expect("polynomial stream must decode"),
            );
            let _ = black_box(polynomial_output.as_slice());
        })
    });

    let idw = single_codec_stream(sine_samples(IDW_SAMPLES), Compressor::Idw);
    group.bench_function("idw/2048", |b| b.iter(|| black_box(idw.decompress())));
    let mut idw_decoder = Decoder::new();
    let mut idw_output = Vec::new();
    group.bench_function("decoder/reused/idw/2048", |b| {
        b.iter(|| {
            idw_output.clear();
            let _ = black_box(
                idw_decoder
                    .decode_into(&idw, &mut idw_output)
                    .expect("IDW stream must decode"),
            );
            let _ = black_box(idw_output.as_slice());
        })
    });

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
    let mut mixed_decoder = Decoder::new();
    let mut mixed_output = Vec::new();
    group.bench_function("decoder/reused/mixed/6_frames", |b| {
        b.iter(|| {
            mixed_output.clear();
            let _ = black_box(
                mixed_decoder
                    .decode_into(&mixed, &mut mixed_output)
                    .expect("mixed stream must decode"),
            );
            let _ = black_box(mixed_output.as_slice());
        })
    });

    let fft = compress_plan(frame_plan(vec![
        (Compressor::FFT, sine_samples(65_536)),
        (Compressor::FFT, sine_samples(65_536)),
    ]));
    group.bench_function("fft/2x65536", |b| b.iter(|| black_box(fft.decompress())));
    let mut fft_decoder = Decoder::new();
    let mut fft_output = Vec::new();
    group.bench_function("decoder/reused/fft/2x65536", |b| {
        b.iter(|| {
            fft_output.clear();
            let _ = black_box(
                fft_decoder
                    .decode_into(&fft, &mut fft_output)
                    .expect("FFT stream must decode"),
            );
            let _ = black_box(fft_output.as_slice());
        })
    });

    group.finish();
}

criterion_group!(benches, codec_benchmarks, stream_benchmarks);
criterion_main!(benches);
