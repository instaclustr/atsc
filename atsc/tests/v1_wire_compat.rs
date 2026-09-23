use atsc::{compressor::Compressor, data::CompressedStream};

/// 9-byte header, then frame vector length, frame_size, varint sample_count
/// (3 bytes), codec, payload length, FFT id, frequency count and position.
const FIRST_FFT_COEFFICIENT_LOW_BYTE: usize = 19;

fn encode(compressor: Compressor, samples: &[f64]) -> Vec<u8> {
    let mut stream = CompressedStream::new();
    stream.compress_chunk_with(samples, compressor);
    stream.to_bytes()
}

fn assert_lossless_fixture(name: &str, compressor: Compressor, samples: &[f64], fixture: &[u8]) {
    assert_eq!(
        encode(compressor, samples),
        fixture,
        "{name} BRO v1 bytes changed"
    );

    let stream = CompressedStream::from_bytes(fixture);
    assert_eq!(
        stream.decompress(),
        samples,
        "{name} BRO v1 fixture no longer decodes losslessly"
    );
}

fn assert_lossy_fixture(name: &str, compressor: Compressor, samples: &[f64], fixture: &[u8]) {
    let generated = encode(compressor, samples);
    assert_eq!(generated.as_slice(), fixture, "{name} BRO v1 bytes changed");

    let fixture_decoded = CompressedStream::from_bytes(fixture).decompress();
    let generated_decoded = CompressedStream::from_bytes(&generated).decompress();
    assert_eq!(
        fixture_decoded, generated_decoded,
        "{name} BRO v1 decoded output changed"
    );
    assert_eq!(
        fixture_decoded.len(),
        samples.len(),
        "{name} BRO v1 fixture decoded to the wrong sample count"
    );
    assert_eq!(
        generated_decoded.len(),
        samples.len(),
        "{name} newly generated BRO v1 bytes decoded to the wrong sample count"
    );
    assert!(
        fixture_decoded
            .iter()
            .chain(&generated_decoded)
            .all(|value| value.is_finite()),
        "{name} BRO v1 decoded output contained a non-finite value"
    );
}

/// rustfft picks architecture-specific SIMD kernels, so FFT coefficients may
/// differ by an f32 ULP between hosts. Decoded samples are rounded to five
/// decimals (quantum 1e-5); 1e-4 absorbs one rounding flip plus f32 IFFT drift.
const FFT_CROSS_ARCH_TOLERANCE: f64 = 1e-4;
/// The unbounded FFT fixture keeps few frequencies, so it decodes `wave()` with
/// a max absolute error of 1.35408 and a mean of 0.48054; the bounds add 1e-3
/// (10x the cross-architecture tolerance) so decoder regressions still fail.
const FFT_FIXTURE_MAX_INPUT_ERROR: f64 = 1.3551;
const FFT_FIXTURE_MEAN_INPUT_ERROR: f64 = 0.4815;

fn decode_fixture(name: &str, bytes: &[u8]) -> Vec<f64> {
    CompressedStream::try_from_bytes(bytes)
        .unwrap_or_else(|error| panic!("{name} BRO v1 bytes no longer parse: {error}"))
        .try_decompress()
        .unwrap_or_else(|error| panic!("{name} BRO v1 bytes no longer decode: {error}"))
}

fn assert_fft_fixture(samples: &[f64], fixture: &[u8]) {
    let fixture_decoded = decode_fixture("fft fixture", fixture);
    assert_eq!(
        fixture_decoded.len(),
        samples.len(),
        "fft BRO v1 fixture decoded to the wrong sample count"
    );
    assert!(
        fixture_decoded.iter().all(|value| value.is_finite()),
        "fft BRO v1 fixture decoded to a non-finite value"
    );
    let max_input_error = fixture_decoded
        .iter()
        .zip(samples)
        .map(|(decoded, sample)| (decoded - sample).abs())
        .fold(0.0, f64::max);
    let mean_input_error = fixture_decoded
        .iter()
        .zip(samples)
        .map(|(decoded, sample)| (decoded - sample).abs())
        .sum::<f64>()
        / samples.len() as f64;
    assert!(
        max_input_error <= FFT_FIXTURE_MAX_INPUT_ERROR
            && mean_input_error <= FFT_FIXTURE_MEAN_INPUT_ERROR,
        "fft BRO v1 fixture drifted from its input: max {max_input_error}, mean {mean_input_error}"
    );

    let generated = encode(Compressor::FFT, samples);
    assert_eq!(
        generated.len(),
        fixture.len(),
        "fft BRO v1 encoded size changed"
    );
    let generated_decoded = decode_fixture("fft generated", &generated);
    assert_eq!(generated_decoded.len(), fixture_decoded.len());
    for (index, (generated, fixture)) in generated_decoded.iter().zip(&fixture_decoded).enumerate()
    {
        assert!(
            (generated - fixture).abs() <= FFT_CROSS_ARCH_TOLERANCE,
            "fft decoded sample {index} drifted: generated {generated}, fixture {fixture}"
        );
    }
}

fn wave() -> Vec<f64> {
    (0..256).map(|i| ((i as f64) / 8.0).sin()).collect()
}

#[test]
fn constant_v1_wire_compatibility() {
    assert_lossless_fixture(
        "constant",
        Compressor::Constant,
        &[7.0; 16],
        include_bytes!("fixtures/v1/constant.bro"),
    );
}

#[test]
fn noop_v1_wire_compatibility() {
    assert_lossless_fixture(
        "noop",
        Compressor::Noop,
        &[1.0, -2.0, 3.0, 4.0, 5.0],
        include_bytes!("fixtures/v1/noop.bro"),
    );
}

#[test]
fn rle_v1_wire_compatibility() {
    assert_lossless_fixture(
        "rle",
        Compressor::RLE,
        &[1.0, 1.0, 2.0, 2.0, 1.0, 1.0],
        include_bytes!("fixtures/v1/rle.bro"),
    );
}

#[test]
fn fft_v1_wire_compatibility() {
    assert_fft_fixture(&wave(), include_bytes!("fixtures/v1/fft.bro"));
}

#[test]
fn fft_v1_check_accepts_one_ulp_coefficient_drift() {
    let mut fixture = include_bytes!("fixtures/v1/fft.bro").to_vec();
    fixture[FIRST_FFT_COEFFICIENT_LOW_BYTE] ^= 1;
    assert_fft_fixture(&wave(), &fixture);
}

#[test]
fn polynomial_v1_wire_compatibility() {
    let samples = wave();
    assert_lossy_fixture(
        "polynomial",
        Compressor::Polynomial,
        &samples,
        include_bytes!("fixtures/v1/polynomial.bro"),
    );
}

#[test]
fn idw_v1_wire_compatibility() {
    let samples = wave();
    assert_lossy_fixture(
        "idw",
        Compressor::Idw,
        &samples,
        include_bytes!("fixtures/v1/idw.bro"),
    );
}
