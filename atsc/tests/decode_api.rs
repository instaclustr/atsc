use atsc::{
    compressor::{
        constant::Constant,
        fft::{FrequencyPoint, FFT},
        noop::Noop,
        polynomial::{Polynomial, PolynomialType},
        rle::IndexRLE,
        Compressor,
    },
    data::CompressedStream,
    error::{DecodeError, DecodeLimits},
    optimizer::utils::Bitdepth,
};
use rustfft::num_complex::Complex;

const FIXTURES: [(&str, &[u8]); 6] = [
    ("constant", include_bytes!("fixtures/v1/constant.bro")),
    ("noop", include_bytes!("fixtures/v1/noop.bro")),
    ("rle", include_bytes!("fixtures/v1/rle.bro")),
    ("fft", include_bytes!("fixtures/v1/fft.bro")),
    ("polynomial", include_bytes!("fixtures/v1/polynomial.bro")),
    ("idw", include_bytes!("fixtures/v1/idw.bro")),
];

#[test]
fn empty_input_reports_truncated_header() {
    assert!(matches!(
        CompressedStream::try_from_bytes(&[]),
        Err(DecodeError::TruncatedHeader { actual: 0 })
    ));
}

#[test]
fn invalid_magic_is_rejected() {
    assert!(matches!(
        CompressedStream::try_from_bytes(b"NOT-BRRO!"),
        Err(DecodeError::InvalidMagic { .. })
    ));
}

#[test]
fn future_version_is_rejected() {
    let mut future_version_fixture = include_bytes!("fixtures/v1/constant.bro").to_vec();
    future_version_fixture[4..8].copy_from_slice(&2_u32.to_le_bytes());

    assert!(matches!(
        CompressedStream::try_from_bytes(&future_version_fixture),
        Err(DecodeError::UnsupportedVersion {
            found: 2,
            supported: 1
        })
    ));
}

#[test]
fn input_limit_is_enforced_before_decode() {
    assert!(matches!(
        CompressedStream::try_from_bytes_with_limits(
            include_bytes!("fixtures/v1/constant.bro"),
            DecodeLimits {
                max_input_bytes: 8,
                ..DecodeLimits::default()
            },
        ),
        Err(DecodeError::InputLimitExceeded { .. })
    ));
}

#[test]
fn header_and_body_frame_counts_must_match() {
    let mut mismatched = include_bytes!("fixtures/v1/constant.bro").to_vec();
    mismatched[8] = 2;

    assert!(matches!(
        CompressedStream::try_from_bytes(&mismatched),
        Err(DecodeError::FrameCountMismatch { header: 2, body: 1 })
    ));
}

#[test]
fn truncated_codec_fixtures_return_typed_errors() {
    for (name, fixture) in FIXTURES {
        let truncated = &fixture[..fixture.len() - 1];
        assert!(
            matches!(
                CompressedStream::try_from_bytes(truncated),
                Err(DecodeError::Bincode { .. })
            ),
            "{name} fixture did not return a bincode decode error"
        );
    }
}

#[test]
fn trailing_container_bytes_are_rejected() {
    let mut trailing = include_bytes!("fixtures/v1/constant.bro").to_vec();
    trailing.push(0);

    assert!(matches!(
        CompressedStream::try_from_bytes(&trailing),
        Err(DecodeError::TrailingBytes { remaining: 1 })
    ));
}

#[test]
fn frame_limit_is_enforced() {
    assert!(matches!(
        CompressedStream::try_from_bytes_with_limits(
            include_bytes!("fixtures/v1/constant.bro"),
            DecodeLimits {
                max_frames: 0,
                ..DecodeLimits::default()
            },
        ),
        Err(DecodeError::FrameLimitExceeded {
            actual: 1,
            limit: 0
        })
    ));
}

#[test]
fn sample_limit_is_enforced() {
    assert!(matches!(
        CompressedStream::try_from_bytes_with_limits(
            include_bytes!("fixtures/v1/constant.bro"),
            DecodeLimits {
                max_samples: 15,
                ..DecodeLimits::default()
            },
        ),
        Err(DecodeError::SampleLimitExceeded {
            actual: 16,
            limit: 15
        })
    ));
}

#[test]
fn codec_payloads_reject_wrong_ids() {
    let mut constant = Constant::new(1, 1.0, Bitdepth::U8);
    constant.id = 0;
    assert!(matches!(
        Constant::try_decompress(&constant.to_bytes()),
        Err(DecodeError::InvalidFrame {
            codec: Compressor::Constant,
            ..
        })
    ));

    let mut noop = Noop::new(1);
    noop.compress(&[1.0]);
    noop.id = 0;
    assert!(matches!(
        Noop::try_decompress(&noop.to_bytes()),
        Err(DecodeError::InvalidFrame {
            codec: Compressor::Noop,
            ..
        })
    ));

    let mut rle = IndexRLE::new(&[1.0], Bitdepth::U8);
    rle.id = 0;
    assert!(matches!(
        IndexRLE::try_decompress(&rle.to_bytes()),
        Err(DecodeError::InvalidFrame {
            codec: Compressor::RLE,
            ..
        })
    ));

    let mut fft = FFT::new(1, 0.0, 1.0);
    fft.id = 0;
    assert!(matches!(
        FFT::try_decompress(&fft.to_bytes()),
        Err(DecodeError::InvalidFrame {
            codec: Compressor::FFT,
            ..
        })
    ));
}

#[test]
fn codec_payloads_reject_trailing_bytes() {
    let constant = Constant::new(1, 1.0, Bitdepth::U8);
    let mut bytes = constant.to_bytes();
    bytes.push(0);

    assert!(matches!(
        Constant::try_decompress(&bytes),
        Err(DecodeError::TrailingBytes { remaining: 1 })
    ));
}

#[test]
fn polynomial_payload_invariants_are_validated() {
    let mut polynomial = Polynomial::new(1, 0.0, 1.0, PolynomialType::Polynomial, Bitdepth::U8);
    polynomial.data_points.push(0.0);
    polynomial.point_step = 0;
    assert!(matches!(
        Polynomial::try_decompress(&polynomial.to_bytes()),
        Err(DecodeError::InvalidFrame {
            codec: Compressor::Polynomial,
            ..
        })
    ));

    polynomial.point_step = 1;
    polynomial.data_points.clear();
    assert!(matches!(
        Polynomial::try_decompress(&polynomial.to_bytes()),
        Err(DecodeError::InvalidFrame {
            codec: Compressor::Polynomial,
            ..
        })
    ));
}

#[test]
fn noop_value_count_must_match_frame_samples() {
    let mut noop = Noop::new(1);
    noop.compress(&[1.0]);

    assert!(matches!(
        Compressor::Noop.try_decompress(2, &noop.to_bytes()),
        Err(DecodeError::InvalidFrame {
            codec: Compressor::Noop,
            ..
        })
    ));
}

#[test]
fn rle_run_starts_must_include_zero_and_fit_frame() {
    let mut missing_zero = IndexRLE::new(&[1.0], Bitdepth::U8);
    missing_zero.rle[0].1[0] = 1;
    assert!(matches!(
        Compressor::RLE.try_decompress(2, &missing_zero.to_bytes()),
        Err(DecodeError::InvalidFrame {
            codec: Compressor::RLE,
            ..
        })
    ));

    let mut out_of_bounds = IndexRLE::new(&[1.0, 2.0], Bitdepth::U8);
    let nonzero_start = out_of_bounds
        .rle
        .iter_mut()
        .flat_map(|(_, starts)| starts)
        .find(|start| **start != 0)
        .expect("two runs include a nonzero start");
    *nonzero_start = 2;
    assert!(matches!(
        Compressor::RLE.try_decompress(2, &out_of_bounds.to_bytes()),
        Err(DecodeError::InvalidFrame {
            codec: Compressor::RLE,
            ..
        })
    ));
}

#[test]
fn fft_frequency_positions_must_fit_reconstructed_length() {
    let mut fft = FFT::new(1, 0.0, 1.0);
    fft.frequencies
        .push(FrequencyPoint::from_complex_with_position(
            Complex { re: 1.0, im: 0.0 },
            1,
        ));

    assert!(matches!(
        Compressor::FFT.try_decompress(1, &fft.to_bytes()),
        Err(DecodeError::InvalidFrame {
            codec: Compressor::FFT,
            ..
        })
    ));
}

#[test]
fn polynomial_payload_type_must_match_frame_codec() {
    let mut idw = Polynomial::new(1, 0.0, 1.0, PolynomialType::Idw, Bitdepth::U8);
    idw.data_points.push(0.0);

    assert!(matches!(
        Compressor::Polynomial.try_decompress(1, &idw.to_bytes()),
        Err(DecodeError::InvalidFrame {
            codec: Compressor::Polynomial,
            ..
        })
    ));
}

#[test]
fn auto_is_not_a_valid_stored_frame_codec() {
    assert!(matches!(
        Compressor::Auto.try_decompress(1, &[]),
        Err(DecodeError::InvalidFrame {
            codec: Compressor::Auto,
            ..
        })
    ));
}

#[test]
fn valid_fixtures_use_the_fallible_decode_path() {
    for (name, fixture) in FIXTURES {
        let stream = CompressedStream::try_from_bytes(fixture)
            .unwrap_or_else(|error| panic!("{name} fixture failed to parse: {error}"));
        let samples = stream
            .try_decompress()
            .unwrap_or_else(|error| panic!("{name} fixture failed to decompress: {error}"));
        assert!(!samples.is_empty(), "{name} fixture decoded no samples");
    }
}
