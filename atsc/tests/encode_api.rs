use std::panic::{catch_unwind, AssertUnwindSafe};

use atsc::{
    compressor::Compressor, data::CompressedStream, error::EncodeError,
    utils::error::calculate_error,
};

const FRAME_SIZE: usize = 131_072;
const MAX_ERROR: f32 = 0.03;
const FASTEST_SELECTION: usize = 6;

fn offset_high_frequency_samples() -> Vec<f64> {
    (0..FRAME_SIZE)
        .map(|index| if index % 2 == 0 { 1.0 } else { 3.0 })
        .collect()
}

fn assert_stream_meets_bound(stream: &CompressedStream, samples: &[f64], max_error: f32) {
    let decoded = stream.try_decompress().expect("bounded stream must decode");
    assert_eq!(decoded.len(), samples.len());
    assert!(decoded.iter().all(|value| value.is_finite()));
    let actual = calculate_error(samples, &decoded);
    assert!(
        actual <= f64::from(max_error),
        "decoded error {actual} exceeded requested bound {max_error}"
    );
}

fn assert_stream_is_empty(stream: &CompressedStream) {
    assert_eq!(stream.frame_count(), 0);
    assert_eq!(stream.sample_count(), 0);
    assert_eq!(stream.header.get_frame_count(), 0);
}

#[test]
fn forced_high_frequency_stream_meets_bound_or_returns_typed_error_transactionally() {
    let samples = offset_high_frequency_samples();
    let mut stream = CompressedStream::new();

    match stream.try_compress_chunk_bounded_with(
        &samples,
        Compressor::FFT,
        MAX_ERROR,
        FASTEST_SELECTION,
    ) {
        Ok(()) => {
            assert_eq!(stream.frame_count(), 1);
            assert_eq!(stream.sample_count(), samples.len());
            assert_stream_meets_bound(&stream, &samples, MAX_ERROR);
        }
        Err(EncodeError::ErrorBoundNotMet {
            codec,
            requested,
            actual,
        }) => {
            assert_eq!(codec, Compressor::FFT);
            assert_eq!(requested, f64::from(MAX_ERROR));
            assert!(!actual.is_finite() || actual > requested);
            assert_stream_is_empty(&stream);
        }
        Err(error) => panic!("unexpected bounded FFT error: {error}"),
    }
}

#[test]
fn forced_nonconstant_constant_rejects_above_bound_without_committing() {
    let samples = [1.0, 2.0];
    let mut stream = CompressedStream::new();

    let error = stream
        .try_compress_chunk_bounded_with(&samples, Compressor::Constant, MAX_ERROR, 0)
        .expect_err("nonconstant input must not pass forced Constant at 3%");

    assert!(matches!(
        error,
        EncodeError::ErrorBoundNotMet {
            codec: Compressor::Constant,
            requested,
            actual,
        } if requested == f64::from(MAX_ERROR) && actual > requested
    ));
    assert_stream_is_empty(&stream);
}

#[test]
fn forced_constant_input_stays_byte_and_output_compatible() {
    let samples = [7.0; 16];
    let mut bounded = CompressedStream::new();
    bounded
        .try_compress_chunk_bounded_with(&samples, Compressor::Constant, MAX_ERROR, 0)
        .expect("constant input must meet a zero-error bound");

    assert_eq!(bounded.frame_count(), 1);
    assert_eq!(bounded.sample_count(), samples.len());
    assert_eq!(bounded.header.get_frame_count(), 1);
    assert_eq!(bounded.try_decompress().unwrap(), samples);

    let mut legacy = CompressedStream::new();
    legacy.compress_chunk_with(&samples, Compressor::Constant);
    assert_eq!(bounded.to_bytes(), legacy.to_bytes());
}

#[test]
fn auto_high_frequency_stream_commits_only_a_full_frame_within_bound() {
    let samples = offset_high_frequency_samples();
    let mut stream = CompressedStream::new();

    stream
        .try_compress_chunk_bounded_with(&samples, Compressor::Auto, MAX_ERROR, FASTEST_SELECTION)
        .expect("Auto must use a valid full-frame candidate");

    assert_eq!(stream.frame_count(), 1);
    assert_eq!(stream.sample_count(), samples.len());
    assert_stream_meets_bound(&stream, &samples, MAX_ERROR);
}

#[test]
fn invalid_compression_speed_is_typed_and_transactional() {
    let samples = offset_high_frequency_samples();
    let mut stream = CompressedStream::new();

    let error = stream
        .try_compress_chunk_bounded_with(&samples, Compressor::Auto, MAX_ERROR, usize::MAX)
        .expect_err("invalid compression speed must fail");

    assert!(matches!(
        error,
        EncodeError::InvalidCompressionSpeed {
            index: usize::MAX,
            ..
        }
    ));
    assert_stream_is_empty(&stream);
}

#[test]
fn infallible_bounded_stream_wrapper_panics_without_committing() {
    let mut stream = CompressedStream::new();
    let samples = [1.0, 2.0];

    let panic = catch_unwind(AssertUnwindSafe(|| {
        stream.compress_chunk_bounded_with(&samples, Compressor::RLE, -1.0, 0);
    }))
    .expect_err("compatibility wrapper must panic when the bound is not met");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or_default();

    assert!(message.contains("bounded stream compression failed"));
    assert_stream_is_empty(&stream);
}
