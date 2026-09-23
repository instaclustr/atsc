use std::panic::{AssertUnwindSafe, catch_unwind};

use atsc::{
    compressor::Compressor, data::CompressedStream, error::EncodeError, frame::CompressorFrame,
    optimizer::OptimizerPlan, utils::error::calculate_error,
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

fn panic_message(panic: &(dyn std::any::Any + Send)) -> &str {
    panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or_default()
}

fn samples_with_zeros() -> Vec<f64> {
    (0..1024).map(|index| f64::from(index % 7)).collect()
}

#[test]
fn infallible_bounded_stream_wrapper_commits_legacy_bytes_when_bound_is_missed() {
    for (codec, samples) in [
        (Compressor::RLE, vec![1.0, 2.0]),
        (Compressor::Constant, vec![1.0, 2.0]),
    ] {
        let mut stream = CompressedStream::new();
        stream.compress_chunk_bounded_with(&samples, codec, -1.0, 0);

        let mut legacy = CompressedStream::new();
        legacy.compress_chunk_with(&samples, codec);
        assert_eq!(stream.to_bytes(), legacy.to_bytes(), "{codec:?}");
    }
}

#[test]
fn infallible_bounded_stream_wrapper_commits_forced_codec_when_error_is_undefined() {
    let samples = samples_with_zeros();
    for codec in [Compressor::FFT, Compressor::Polynomial, Compressor::Idw] {
        let mut strict = CompressedStream::new();
        let error = strict
            .try_compress_chunk_bounded_with(&samples, codec, MAX_ERROR, 0)
            .expect_err("MAPE is undefined for zero-valued samples");
        assert!(
            matches!(error, EncodeError::ErrorBoundNotMet { actual, .. } if !actual.is_finite()),
            "{error:?}"
        );
        assert_stream_is_empty(&strict);

        let mut stream = CompressedStream::new();
        stream.compress_chunk_bounded_with(&samples, codec, MAX_ERROR, 0);
        assert_eq!(stream.frame_count(), 1);
        assert_eq!(stream.frame_info().next().unwrap().compressor, codec);
        let decoded = stream
            .try_decompress()
            .expect("best-effort frame must decode");
        assert_eq!(decoded.len(), samples.len());
        assert!(decoded.iter().all(|value| value.is_finite()));
    }
}

#[test]
fn infallible_codec_bounded_wrapper_returns_bounded_bytes_when_bound_is_missed() {
    let samples = samples_with_zeros();
    for codec in [
        Compressor::Noop,
        Compressor::FFT,
        Compressor::Idw,
        Compressor::Constant,
        Compressor::Polynomial,
        Compressor::RLE,
    ] {
        let expected = codec.get_compress_bounded_results(&samples, -1.0);
        assert_eq!(
            codec.compress_bounded(&samples, -1.0),
            expected.compressed_data,
            "{codec:?}"
        );
    }

    assert!(catch_unwind(|| Compressor::Auto.compress_bounded(&[1.0], 0.03)).is_err());
    assert!(catch_unwind(|| Compressor::Noop.compress_bounded(&[f64::NAN], 0.03)).is_err());
    assert!(catch_unwind(|| Compressor::Noop.compress_bounded(&[], 0.03)).is_err());
}

#[test]
fn infallible_auto_stream_wrapper_commits_a_frame_when_no_candidate_meets_bound() {
    let samples = samples_with_zeros();
    let mut stream = CompressedStream::new();

    stream.compress_chunk_bounded_with(&samples, Compressor::Auto, -1.0, FASTEST_SELECTION);

    assert_eq!(stream.frame_count(), 1);
    assert_eq!(stream.header.get_frame_count(), 1);
    assert_eq!(stream.try_decompress().unwrap().len(), samples.len());
}

#[test]
fn infallible_bounded_stream_wrapper_still_panics_on_non_bound_errors_without_committing() {
    let mut stream = CompressedStream::new();
    let panic = catch_unwind(AssertUnwindSafe(|| {
        stream.compress_chunk_bounded_with(&[1.0, f64::NAN], Compressor::RLE, MAX_ERROR, 0);
    }))
    .expect_err("non-finite input is not a bound failure");
    assert!(panic_message(panic.as_ref()).contains("bounded stream compression failed"));
    assert_stream_is_empty(&stream);

    for _ in 0..u8::MAX {
        stream.compress_chunk_bounded_with(&[7.0], Compressor::Constant, MAX_ERROR, 0);
    }
    let serialized = stream.clone().to_bytes();
    catch_unwind(AssertUnwindSafe(|| {
        stream.compress_chunk_bounded_with(&[9.0], Compressor::Constant, -1.0, 0);
    }))
    .expect_err("the compatibility wrapper must panic at the BRO v1 frame limit");
    assert_eq!(stream.frame_count(), usize::from(u8::MAX));
    assert_eq!(stream.clone().to_bytes(), serialized);
}

#[test]
fn fractional_noop_is_rejected_by_a_zero_error_bound_without_changing_payload_bytes() {
    let samples = [0.0, 1.5, -2.0];
    let legacy_payload = Compressor::Noop.compress(&samples);

    let error = Compressor::Noop
        .try_compress_bounded(&samples, 0.0)
        .expect_err("rounded fractional input is not lossless");

    assert!(matches!(
        error,
        EncodeError::ErrorBoundNotMet {
            codec: Compressor::Noop,
            requested: 0.0,
            actual,
        } if actual.is_finite() && actual > 0.0
    ));
    assert_eq!(Compressor::Noop.compress(&samples), legacy_payload);
}

#[test]
fn exact_integer_noop_meets_a_zero_error_bound_with_legacy_bytes() {
    let samples = [0.0, 1.0, -2.0, 9.0];
    let legacy_payload = Compressor::Noop.compress(&samples);

    let bounded_payload = Compressor::Noop
        .try_compress_bounded(&samples, 0.0)
        .expect("integer-valued Noop input must reconstruct exactly");

    assert_eq!(bounded_payload, legacy_payload);
}

#[test]
fn every_fallible_bounded_entry_point_rejects_empty_input() {
    assert_eq!(
        Compressor::Noop.try_compress_bounded(&[], 0.0),
        Err(EncodeError::EmptyInput)
    );

    let mut forced_frame = CompressorFrame::new(Some(Compressor::Noop));
    assert_eq!(
        forced_frame.try_compress_bounded(&[], 0.0),
        Err(EncodeError::EmptyInput)
    );

    let mut auto_frame = CompressorFrame::new(Some(Compressor::Auto));
    assert_eq!(
        auto_frame.try_compress_best(&[], 0.0, 0),
        Err(EncodeError::EmptyInput)
    );

    let mut stream = CompressedStream::new();
    assert_eq!(
        stream.try_compress_chunk_bounded_with(&[], Compressor::Noop, 0.0, 0),
        Err(EncodeError::EmptyInput)
    );
    assert_stream_is_empty(&stream);

    assert!(matches!(
        OptimizerPlan::try_plan(&[]),
        Err(EncodeError::EmptyInput)
    ));
}

#[test]
fn every_fallible_bounded_entry_point_reports_the_non_finite_index() {
    let samples = [1.0, 2.0, f64::NAN, f64::INFINITY];
    let expected = EncodeError::NonFiniteInput { index: 2 };

    assert_eq!(
        Compressor::Noop.try_compress_bounded(&samples, 1.0),
        Err(expected)
    );

    let mut forced_frame = CompressorFrame::new(Some(Compressor::Noop));
    assert_eq!(
        forced_frame.try_compress_bounded(&samples, 1.0),
        Err(EncodeError::NonFiniteInput { index: 2 })
    );

    let mut auto_frame = CompressorFrame::new(Some(Compressor::Auto));
    assert_eq!(
        auto_frame.try_compress_best(&samples, 1.0, 0),
        Err(EncodeError::NonFiniteInput { index: 2 })
    );

    let mut stream = CompressedStream::new();
    assert_eq!(
        stream.try_compress_chunk_bounded_with(&samples, Compressor::Auto, 1.0, 0),
        Err(EncodeError::NonFiniteInput { index: 2 })
    );
    assert_stream_is_empty(&stream);

    assert!(matches!(
        OptimizerPlan::try_plan(&samples),
        Err(EncodeError::NonFiniteInput { index: 2 })
    ));
}

#[test]
fn optimizer_plan_compatibility_wrapper_drops_non_finite_samples() {
    let samples = [1.0, f64::NAN, 2.0, f64::NEG_INFINITY, 3.0, f64::INFINITY];

    let plan = OptimizerPlan::plan(&samples);

    assert_eq!(plan.data, [1.0, 2.0, 3.0]);
    assert_eq!(plan.chunk_sizes, [3]);
    assert_eq!(plan.get_execution().len(), 1);
    assert!(matches!(
        OptimizerPlan::try_plan(&samples),
        Err(EncodeError::NonFiniteInput { index: 1 })
    ));
}

#[test]
fn the_256th_fallible_frame_is_rejected_before_stream_mutation() {
    let mut stream = CompressedStream::new();
    for value in 0..u8::MAX {
        stream
            .try_compress_chunk_bounded_with(&[f64::from(value)], Compressor::Constant, 0.0, 0)
            .expect("the first 255 frames fit BRO v1");
    }
    let frame_count = stream.frame_count();
    let sample_count = stream.sample_count();
    let header_count = stream.header.get_frame_count();
    let serialized = stream.clone().to_bytes();

    let error = stream
        .try_compress_chunk_bounded_with(&[999.0], Compressor::Constant, 0.0, 0)
        .expect_err("BRO v1 cannot represent a 256th frame");

    assert_eq!(
        error,
        EncodeError::FrameLimitExceeded {
            limit: usize::from(u8::MAX)
        }
    );
    assert_eq!(stream.frame_count(), frame_count);
    assert_eq!(stream.sample_count(), sample_count);
    assert_eq!(stream.header.get_frame_count(), header_count);
    assert_eq!(stream.clone().to_bytes(), serialized);
}

#[test]
fn infallible_stream_wrapper_panics_before_a_256th_frame_mutates_state() {
    let mut stream = CompressedStream::new();
    for _ in 0..u8::MAX {
        stream.compress_chunk_with(&[7.0], Compressor::Constant);
    }
    let serialized = stream.clone().to_bytes();

    catch_unwind(AssertUnwindSafe(|| {
        stream.compress_chunk_with(&[9.0], Compressor::Constant);
    }))
    .expect_err("the compatibility wrapper must panic at the BRO v1 frame limit");

    assert_eq!(stream.frame_count(), usize::from(u8::MAX));
    assert_eq!(stream.header.get_frame_count(), u8::MAX);
    assert_eq!(stream.clone().to_bytes(), serialized);
}
