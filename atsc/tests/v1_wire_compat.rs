use atsc::{compressor::Compressor, data::CompressedStream};

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
    let samples = wave();
    assert_lossy_fixture(
        "fft",
        Compressor::FFT,
        &samples,
        include_bytes!("fixtures/v1/fft.bro"),
    );
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
