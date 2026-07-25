use atsc::{
    compressor::{
        constant::Constant,
        fft::{FrequencyPoint, FFT},
        noop::Noop,
        polynomial::{Polynomial, PolynomialType},
        rle::IndexRLE,
        BinConfig, Compressor,
    },
    data::CompressedStream,
    decoder::{Decoder, FrameInfo},
    error::{DecodeError, DecodeLimits},
    optimizer::utils::Bitdepth,
};
use bincode::Encode;
use rustfft::num_complex::Complex;
use std::panic::catch_unwind;

const FIXTURES: [(&str, &[u8]); 6] = [
    ("constant", include_bytes!("fixtures/v1/constant.bro")),
    ("noop", include_bytes!("fixtures/v1/noop.bro")),
    ("rle", include_bytes!("fixtures/v1/rle.bro")),
    ("fft", include_bytes!("fixtures/v1/fft.bro")),
    ("polynomial", include_bytes!("fixtures/v1/polynomial.bro")),
    ("idw", include_bytes!("fixtures/v1/idw.bro")),
];
const MAX_V1_POLYNOMIAL_POINTS: usize = 131_072;

fn truncated(mut data: Vec<u8>) -> Vec<u8> {
    data.pop().expect("test payload must not be empty");
    data
}

fn with_trailing_byte(mut data: Vec<u8>) -> Vec<u8> {
    data.push(0);
    data
}

fn noop_payload() -> Vec<u8> {
    let mut noop = Noop::new(1);
    noop.compress(&[1.0]);
    noop.to_bytes()
}

fn polynomial_payload(polynomial_type: PolynomialType) -> Vec<u8> {
    let mut polynomial = Polynomial::new(2, 0.0, 1.0, polynomial_type, Bitdepth::U8);
    polynomial.data_points.extend([0.0, 1.0]);
    polynomial.to_bytes()
}

fn polynomial_point_prefix(
    polynomial_type: PolynomialType,
    bitdepth: Bitdepth,
    point_count: u64,
) -> Vec<u8> {
    let mut payload = Vec::new();
    bincode::encode_into_std_write(polynomial_type, &mut payload, BinConfig::get())
        .expect("polynomial type must encode");
    bincode::encode_into_std_write(bitdepth, &mut payload, BinConfig::get())
        .expect("bitdepth must encode");
    bincode::encode_into_std_write(point_count, &mut payload, BinConfig::get())
        .expect("point count must encode");
    payload
}

fn assert_polynomial_point_cap(error: DecodeError) {
    assert!(matches!(
        error,
        DecodeError::Bincode {
            context: "Polynomial payload",
            source: bincode::error::DecodeError::Other(
                "Polynomial/IDW point count exceeds BRO v1 maximum"
            ),
        }
    ));
}

fn assert_bincode_point_cap(error: bincode::error::DecodeError) {
    assert!(matches!(
        error,
        bincode::error::DecodeError::Other("Polynomial/IDW point count exceeds BRO v1 maximum")
    ));
}

fn maximum_vector_length_prefix() -> Vec<u8> {
    let mut prefix = vec![253];
    prefix.extend_from_slice(&u64::MAX.to_le_bytes());
    prefix
}

fn two_frame_stream() -> CompressedStream {
    let mut stream = CompressedStream::new();
    stream.compress_chunk_with(&[1.0, 1.0, 1.0], Compressor::Constant);
    stream.compress_chunk_with(&[4.0, 5.0], Compressor::Noop);
    stream
}

fn three_frame_stream() -> CompressedStream {
    let mut stream = CompressedStream::new();
    stream.compress_chunk_with(&[1.0, 1.0], Compressor::Constant);
    stream.compress_chunk_with(&[2.0, 3.0, 4.0], Compressor::Noop);
    stream.compress_chunk_with(&[5.0, 5.0, 5.0, 5.0], Compressor::Constant);
    stream
}

#[derive(Encode)]
struct EncodedFrame {
    frame_size: usize,
    sample_count: usize,
    compressor: Compressor,
    data: Vec<u8>,
}

fn stream_from_encoded_frames(frames: Vec<EncodedFrame>) -> CompressedStream {
    stream_from_encoded_frames_with_limits(frames, DecodeLimits::default())
}

fn stream_from_encoded_frames_with_limits(
    frames: Vec<EncodedFrame>,
    limits: DecodeLimits,
) -> CompressedStream {
    let mut bytes = b"BRRO".to_vec();
    bytes.extend_from_slice(&1_u32.to_le_bytes());
    bytes.push(frames.len() as u8);
    bincode::encode_into_std_write(frames, &mut bytes, BinConfig::get())
        .expect("test frames must encode");
    CompressedStream::try_from_bytes_with_limits(&bytes, limits).expect("test stream must parse")
}

fn huge_constant_stream() -> CompressedStream {
    stream_from_encoded_frames_with_limits(
        vec![EncodedFrame {
            frame_size: 0,
            sample_count: usize::MAX,
            compressor: Compressor::Constant,
            data: Constant::new(1, 7.0, Bitdepth::U8).to_bytes(),
        }],
        DecodeLimits {
            max_samples: usize::MAX,
            ..DecodeLimits::default()
        },
    )
}

fn assert_allocation_error<T>(result: Result<T, DecodeError>) {
    assert!(matches!(result, Err(DecodeError::AllocationFailed { .. })));
}

fn stream_with_invalid_outer_frames() -> CompressedStream {
    let invalid_frame = || EncodedFrame {
        frame_size: 0,
        sample_count: 2,
        compressor: Compressor::Auto,
        data: Vec::new(),
    };
    let frames = vec![
        invalid_frame(),
        EncodedFrame {
            frame_size: 0,
            sample_count: 2,
            compressor: Compressor::Constant,
            data: Constant::new(2, 7.0, Bitdepth::U8).to_bytes(),
        },
        invalid_frame(),
    ];

    stream_from_encoded_frames(frames)
}

fn stream_with_invalid_later_frame() -> CompressedStream {
    stream_from_encoded_frames(vec![
        EncodedFrame {
            frame_size: 0,
            sample_count: 2,
            compressor: Compressor::Constant,
            data: Constant::new(2, 7.0, Bitdepth::U8).to_bytes(),
        },
        EncodedFrame {
            frame_size: 0,
            sample_count: 2,
            compressor: Compressor::Auto,
            data: Vec::new(),
        },
    ])
}

#[test]
fn decode_into_preserves_prefix_for_every_codec_fixture() {
    for (name, fixture) in FIXTURES {
        let stream = CompressedStream::try_from_bytes(fixture)
            .unwrap_or_else(|error| panic!("{name} fixture failed to parse: {error}"));
        let expected = stream
            .try_decompress()
            .unwrap_or_else(|error| panic!("{name} fixture failed to decode: {error}"));
        let prefix = [-11.0, -22.0];
        let mut output = prefix.to_vec();
        let mut decoder = Decoder::new();

        let appended = decoder
            .decode_into(&stream, &mut output)
            .unwrap_or_else(|error| panic!("{name} fixture failed to decode into output: {error}"));

        assert_eq!(appended, stream.sample_count(), "{name}");
        assert_eq!(&output[..prefix.len()], &prefix, "{name}");
        assert_eq!(&output[prefix.len()..], expected, "{name}");
    }
}

#[test]
fn decode_into_appends_and_reports_only_new_samples() {
    let stream = two_frame_stream();
    let mut decoder = Decoder::new();
    let mut output = vec![-1.0, -2.0];

    let appended = decoder
        .decode_into(&stream, &mut output)
        .expect("stream must decode");

    assert_eq!(appended, stream.sample_count());
    assert_eq!(output, vec![-1.0, -2.0, 1.0, 1.0, 1.0, 4.0, 5.0]);
}

#[test]
fn decode_into_restores_output_after_later_frame_error() {
    let stream = stream_with_invalid_later_frame();
    let mut decoder = Decoder::new();
    let mut output = vec![-1.0, -2.0];

    assert!(matches!(
        decoder.decode_into(&stream, &mut output),
        Err(DecodeError::InvalidFrame {
            codec: Compressor::Auto,
            ..
        })
    ));
    assert_eq!(output, vec![-1.0, -2.0]);
}

#[test]
fn one_decoder_can_decode_independent_streams() {
    let first = two_frame_stream();
    let mut second = CompressedStream::new();
    second.compress_chunk_with(&[9.0, 9.0], Compressor::Constant);
    let mut decoder = Decoder::new();

    assert_eq!(
        decoder.decode(&first).expect("first stream must decode"),
        vec![1.0, 1.0, 1.0, 4.0, 5.0]
    );
    assert_eq!(
        decoder.decode(&second).expect("second stream must decode"),
        vec![9.0, 9.0]
    );
}

#[test]
fn decode_frame_into_appends_only_the_requested_frame() {
    let stream = two_frame_stream();
    let mut decoder = Decoder::new();
    let mut output = vec![-1.0];

    let appended = decoder
        .decode_frame_into(&stream, 1, &mut output)
        .expect("requested frame must decode");

    assert_eq!(appended, 2);
    assert_eq!(output, vec![-1.0, 4.0, 5.0]);
}

#[test]
fn decode_frame_into_rejects_out_of_bounds_indices() {
    let stream = two_frame_stream();
    let mut decoder = Decoder::new();

    for index in [stream.frame_count(), usize::MAX] {
        let mut output = vec![-1.0];
        assert!(matches!(
            decoder.decode_frame_into(&stream, index, &mut output),
            Err(DecodeError::FrameOutOfBounds { index: actual, frames: 2 })
                if actual == index
        ));
        assert_eq!(output, vec![-1.0]);
    }
}

#[test]
fn decode_frame_into_restores_output_after_decode_error() {
    let stream = stream_with_invalid_outer_frames();
    let mut decoder = Decoder::new();
    let mut output = vec![-1.0, -2.0];

    assert!(matches!(
        decoder.decode_frame_into(&stream, 0, &mut output),
        Err(DecodeError::InvalidFrame {
            codec: Compressor::Auto,
            ..
        })
    ));
    assert_eq!(output, vec![-1.0, -2.0]);
}

#[test]
fn decode_range_spanning_frames_matches_full_decode() {
    let stream = two_frame_stream();
    let mut decoder = Decoder::new();
    let full = decoder.decode(&stream).expect("stream must decode");
    let range = 2..4;

    assert_eq!(
        decoder
            .decode_range(&stream, range.clone())
            .expect("range must decode"),
        full[range.clone()]
    );

    let mut output = vec![-1.0];
    let appended = decoder
        .decode_range_into(&stream, range.clone(), &mut output)
        .expect("range must decode");
    assert_eq!(appended, range.len());
    assert_eq!(&output[1..], &full[range]);
}

#[test]
fn decode_range_into_restores_output_after_later_frame_error() {
    let stream = stream_with_invalid_later_frame();
    let mut decoder = Decoder::new();
    let mut output = vec![-1.0, -2.0];

    assert!(matches!(
        decoder.decode_range_into(&stream, 1..3, &mut output),
        Err(DecodeError::InvalidFrame {
            codec: Compressor::Auto,
            ..
        })
    ));
    assert_eq!(output, vec![-1.0, -2.0]);
}

#[test]
fn decode_range_accepts_empty_ranges_including_stream_end() {
    let stream = two_frame_stream();
    let mut decoder = Decoder::new();

    for position in [0, 3, stream.sample_count()] {
        assert!(decoder
            .decode_range(&stream, position..position)
            .expect("empty range must decode")
            .is_empty());

        let mut output = vec![-1.0];
        assert_eq!(
            decoder
                .decode_range_into(&stream, position..position, &mut output)
                .expect("empty range must decode"),
            0
        );
        assert_eq!(output, vec![-1.0]);
    }
}

#[test]
fn decode_range_rejects_reversed_and_out_of_bounds_ranges() {
    let stream = two_frame_stream();
    let mut decoder = Decoder::new();

    for range in [
        std::ops::Range { start: 4, end: 3 },
        0..6,
        6..6,
        usize::MAX..usize::MAX,
    ] {
        let start = range.start;
        let end = range.end;
        assert!(matches!(
            decoder.decode_range(&stream, range.clone()),
            Err(DecodeError::RangeOutOfBounds {
                start: actual_start,
                end: actual_end,
                samples: 5,
            }) if actual_start == start && actual_end == end
        ));

        let mut output = vec![-1.0];
        assert!(matches!(
            decoder.decode_range_into(&stream, range, &mut output),
            Err(DecodeError::RangeOutOfBounds {
                start: actual_start,
                end: actual_end,
                samples: 5,
            }) if actual_start == start && actual_end == end
        ));
        assert_eq!(output, vec![-1.0]);
    }
}

#[test]
fn decode_range_skips_non_intersecting_frames() {
    let stream = stream_with_invalid_outer_frames();
    let mut decoder = Decoder::new();

    assert_eq!(
        decoder
            .decode_range(&stream, 2..4)
            .expect("only the valid middle frame should be decoded"),
        vec![7.0, 7.0]
    );
    assert!(matches!(
        decoder.decode(&stream),
        Err(DecodeError::InvalidFrame {
            codec: Compressor::Auto,
            ..
        })
    ));
}

#[test]
fn frame_info_sequential_collect_preserves_offsets() {
    let stream = two_frame_stream();
    let infos = stream.frame_info();
    assert_eq!(infos.len(), stream.frame_count());
    let infos: Vec<FrameInfo> = infos.collect();

    assert_eq!(stream.frame_count(), 2);
    assert_eq!(stream.sample_count(), 5);
    assert_eq!(infos[0].index, 0);
    assert_eq!(infos[0].sample_offset, 0);
    assert_eq!(infos[0].sample_count, 3);
    assert_eq!(infos[0].compressor, Compressor::Constant);
    assert!(infos[0].payload_bytes > 0);
    assert_eq!(infos[1].index, 1);
    assert_eq!(
        infos[0].sample_offset.checked_add(infos[0].sample_count),
        Some(infos[1].sample_offset)
    );
    assert_eq!(infos[1].sample_count, 2);
    assert_eq!(infos[1].compressor, Compressor::Noop);
    assert!(infos[1].payload_bytes > 0);
    assert_eq!(
        infos[1].sample_offset.checked_add(infos[1].sample_count),
        Some(stream.sample_count())
    );
}

#[test]
fn frame_info_nth_consumes_preceding_offsets() {
    let stream = three_frame_stream();

    let second = stream
        .frame_info()
        .nth(1)
        .expect("stream must contain a second frame");

    assert_eq!(second.index, 1);
    assert_eq!(second.sample_offset, 2);
    assert_eq!(second.sample_count, 3);
}

#[test]
fn frame_info_last_consumes_preceding_offsets() {
    let stream = three_frame_stream();

    let last = stream
        .frame_info()
        .last()
        .expect("stream must contain a last frame");

    assert_eq!(last.index, 2);
    assert_eq!(last.sample_offset, 5);
    assert_eq!(last.sample_count, 4);
}

#[test]
fn frame_info_size_hint_and_len_track_partial_consumption() {
    let stream = three_frame_stream();
    let mut infos = stream.frame_info();

    assert_eq!(infos.size_hint(), (3, Some(3)));
    assert_eq!(infos.len(), 3);
    assert_eq!(
        infos
            .next()
            .expect("stream must contain a first frame")
            .index,
        0
    );
    assert_eq!(infos.size_hint(), (2, Some(2)));
    assert_eq!(infos.len(), 2);
    assert_eq!(
        infos
            .next()
            .expect("stream must contain a second frame")
            .sample_offset,
        2
    );
    assert_eq!(infos.size_hint(), (1, Some(1)));
    assert_eq!(infos.len(), 1);
}

#[test]
fn frame_info_mixed_consumption_preserves_final_offset() {
    let stream = three_frame_stream();
    let mut infos = stream.frame_info();
    assert_eq!(
        infos
            .next()
            .expect("stream must contain a first frame")
            .index,
        0
    );

    let last = infos
        .nth(1)
        .expect("stream must contain a third frame after one skip");

    assert_eq!(last.index, 2);
    assert_eq!(
        last.sample_offset.checked_add(last.sample_count),
        Some(stream.sample_count())
    );
    assert_eq!(infos.len(), 0);
}

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
fn frame_limit_preflight_wins_over_truncated_outer_frame_data() {
    let mut data = b"BRRO".to_vec();
    data.extend_from_slice(&1_u32.to_le_bytes());
    data.push(1);
    data.push(1);

    assert!(matches!(
        CompressedStream::try_from_bytes_with_limits(
            &data,
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
fn outer_frame_count_mismatch_is_rejected_before_frame_allocation() {
    for (header, body) in [(1_u8, 2_u8), (2_u8, 1_u8)] {
        let mut data = b"BRRO".to_vec();
        data.extend_from_slice(&1_u32.to_le_bytes());
        data.push(header);
        data.push(body);

        assert!(matches!(
            CompressedStream::try_from_bytes(&data),
            Err(DecodeError::FrameCountMismatch {
                header: actual_header,
                body: actual_body,
            }) if actual_header == usize::from(header) && actual_body == usize::from(body)
        ));
    }
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
fn outer_frame_vector_length_prefix_is_bounded() {
    let mut data = b"BRRO".to_vec();
    data.extend_from_slice(&1_u32.to_le_bytes());
    data.push(1);
    data.extend(maximum_vector_length_prefix());

    assert!(matches!(
        CompressedStream::try_from_bytes(&data),
        Err(DecodeError::FrameLimitExceeded {
            actual: usize::MAX,
            limit: 255
        })
    ));
}

#[test]
fn nested_noop_vector_length_prefix_is_bounded() {
    let mut data = vec![250];
    data.extend(maximum_vector_length_prefix());

    assert!(matches!(
        Noop::try_decompress(&data),
        Err(DecodeError::Bincode { .. })
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
fn codec_payloads_reject_non_finite_reconstruction_parameters() {
    let constant = Constant::new(1, f64::NAN, Bitdepth::F64);
    assert!(matches!(
        Constant::try_decompress(&constant.to_bytes()),
        Err(DecodeError::InvalidFrame {
            codec: Compressor::Constant,
            ..
        })
    ));

    let mut rle = IndexRLE::new(&[1.5], Bitdepth::F64);
    rle.rle[0].0 = f64::INFINITY;
    assert!(matches!(
        IndexRLE::try_decompress(&rle.to_bytes()),
        Err(DecodeError::InvalidFrame {
            codec: Compressor::RLE,
            ..
        })
    ));

    let mut fft = FFT::new(1, 0.0, 1.0);
    fft.max_value = f32::NAN;
    assert!(matches!(
        FFT::try_decompress(&fft.to_bytes()),
        Err(DecodeError::InvalidFrame {
            codec: Compressor::FFT,
            ..
        })
    ));

    let mut fft = FFT::new(1, 0.0, 1.0);
    fft.frequencies
        .push(FrequencyPoint::from_complex_with_position(
            Complex {
                re: f32::INFINITY,
                im: 0.0,
            },
            0,
        ));
    assert!(matches!(
        FFT::try_decompress(&fft.to_bytes()),
        Err(DecodeError::InvalidFrame {
            codec: Compressor::FFT,
            ..
        })
    ));

    for polynomial_type in [PolynomialType::Polynomial, PolynomialType::Idw] {
        let codec = match polynomial_type {
            PolynomialType::Polynomial => Compressor::Polynomial,
            PolynomialType::Idw => Compressor::Idw,
        };
        let mut polynomial = Polynomial::new(2, 0.0, 1.0, polynomial_type, Bitdepth::F64);
        polynomial.data_points.extend([0.0, f64::NEG_INFINITY]);
        assert!(matches!(
            Polynomial::try_decompress(&polynomial.to_bytes()),
            Err(DecodeError::InvalidFrame {
                codec: actual,
                ..
            }) if actual == codec
        ));
    }
}

#[test]
fn truncated_constant_payload_returns_bincode_error() {
    assert!(matches!(
        Constant::try_decompress(&truncated(Constant::new(1, 1.0, Bitdepth::U8).to_bytes())),
        Err(DecodeError::Bincode { .. })
    ));
}

#[test]
fn trailing_constant_payload_returns_trailing_bytes() {
    assert!(matches!(
        Constant::try_decompress(&with_trailing_byte(
            Constant::new(1, 1.0, Bitdepth::U8).to_bytes()
        )),
        Err(DecodeError::TrailingBytes { remaining: 1 })
    ));
}

#[test]
fn truncated_noop_payload_returns_bincode_error() {
    assert!(matches!(
        Noop::try_decompress(&truncated(noop_payload())),
        Err(DecodeError::Bincode { .. })
    ));
}

#[test]
fn trailing_noop_payload_returns_trailing_bytes() {
    assert!(matches!(
        Noop::try_decompress(&with_trailing_byte(noop_payload())),
        Err(DecodeError::TrailingBytes { remaining: 1 })
    ));
}

#[test]
fn truncated_rle_payload_returns_bincode_error() {
    assert!(matches!(
        IndexRLE::try_decompress(&truncated(IndexRLE::new(&[1.0], Bitdepth::U8).to_bytes())),
        Err(DecodeError::Bincode { .. })
    ));
}

#[test]
fn trailing_rle_payload_returns_trailing_bytes() {
    assert!(matches!(
        IndexRLE::try_decompress(&with_trailing_byte(
            IndexRLE::new(&[1.0], Bitdepth::U8).to_bytes()
        )),
        Err(DecodeError::TrailingBytes { remaining: 1 })
    ));
}

#[test]
fn truncated_fft_payload_returns_bincode_error() {
    assert!(matches!(
        FFT::try_decompress(&truncated(FFT::new(1, 0.0, 1.0).to_bytes())),
        Err(DecodeError::Bincode { .. })
    ));
}

#[test]
fn trailing_fft_payload_returns_trailing_bytes() {
    assert!(matches!(
        FFT::try_decompress(&with_trailing_byte(FFT::new(1, 0.0, 1.0).to_bytes())),
        Err(DecodeError::TrailingBytes { remaining: 1 })
    ));
}

#[test]
fn truncated_polynomial_payload_returns_bincode_error() {
    assert!(matches!(
        Polynomial::try_decompress(&truncated(polynomial_payload(PolynomialType::Polynomial))),
        Err(DecodeError::Bincode { .. })
    ));
}

#[test]
fn trailing_polynomial_payload_returns_trailing_bytes() {
    assert!(matches!(
        Polynomial::try_decompress(&with_trailing_byte(polynomial_payload(
            PolynomialType::Polynomial
        ))),
        Err(DecodeError::TrailingBytes { remaining: 1 })
    ));
}

#[test]
fn truncated_idw_payload_returns_bincode_error() {
    assert!(matches!(
        Polynomial::try_decompress(&truncated(polynomial_payload(PolynomialType::Idw))),
        Err(DecodeError::Bincode { .. })
    ));
}

#[test]
fn trailing_idw_payload_returns_trailing_bytes() {
    assert!(matches!(
        Polynomial::try_decompress(&with_trailing_byte(polynomial_payload(PolynomialType::Idw))),
        Err(DecodeError::TrailingBytes { remaining: 1 })
    ));
}

#[test]
fn integer_polynomial_and_idw_point_counts_are_capped_before_scalar_storage() {
    for polynomial_type in [PolynomialType::Polynomial, PolynomialType::Idw] {
        for bitdepth in [Bitdepth::U8, Bitdepth::I16, Bitdepth::I32] {
            let payload = polynomial_point_prefix(
                polynomial_type.clone(),
                bitdepth,
                (MAX_V1_POLYNOMIAL_POINTS + 1) as u64,
            );

            assert_polynomial_point_cap(
                Polynomial::try_decompress(&payload)
                    .expect_err("direct payload decode must reject an oversized point vector"),
            );
            assert_bincode_point_cap(
                bincode::borrow_decode_from_slice::<Polynomial, _>(&payload, BinConfig::get())
                    .expect_err("borrowed payload decode must reject an oversized point vector"),
            );
        }
    }
}

#[test]
fn huge_polynomial_point_prefix_is_typed_for_decode_and_borrow_decode() {
    let payload = polynomial_point_prefix(PolynomialType::Polynomial, Bitdepth::U8, u64::MAX);

    assert_polynomial_point_cap(
        Polynomial::try_decompress(&payload)
            .expect_err("direct payload decode must reject the maximum length prefix"),
    );
    let borrowed = catch_unwind(|| {
        bincode::borrow_decode_from_slice::<Polynomial, _>(&payload, BinConfig::get())
    });
    assert!(
        matches!(borrowed, Ok(Err(error)) if matches!(
            error,
            bincode::error::DecodeError::Other(
                "Polynomial/IDW point count exceeds BRO v1 maximum"
            )
        )),
        "borrowed decode must return the typed cap error without panicking"
    );
}

#[test]
fn polynomial_widened_storage_honors_the_bincode_decoder_limit() {
    let payload = polynomial_point_prefix(PolynomialType::Polynomial, Bitdepth::U8, 2);

    assert!(matches!(
        bincode::decode_from_slice::<Polynomial, _>(
            &payload,
            bincode::config::standard().with_limit::<24>(),
        ),
        Err(bincode::error::DecodeError::LimitExceeded)
    ));
    assert!(matches!(
        bincode::borrow_decode_from_slice::<Polynomial, _>(
            &payload,
            bincode::config::standard().with_limit::<24>(),
        ),
        Err(bincode::error::DecodeError::LimitExceeded)
    ));
}

#[test]
fn frame_dispatch_rejects_many_points_before_reading_a_one_sample_payload() {
    for (polynomial_type, codec) in [
        (PolynomialType::Polynomial, Compressor::Polynomial),
        (PolynomialType::Idw, Compressor::Idw),
    ] {
        let payload = polynomial_point_prefix(polynomial_type, Bitdepth::U8, 1024);

        assert!(matches!(
            codec.try_decompress(1, &payload),
            Err(DecodeError::InvalidFrame {
                codec: actual_codec,
                reason,
            }) if actual_codec == codec
                && reason.contains("encoded 1024 points exceed 1 declared samples")
        ));
    }
}

#[test]
fn direct_and_borrowed_polynomial_decodes_preserve_valid_integer_payloads() {
    for polynomial_type in [PolynomialType::Polynomial, PolynomialType::Idw] {
        for (bitdepth, points) in [
            (Bitdepth::U8, vec![1.0, 2.0, 3.0]),
            (Bitdepth::I16, vec![-2.0, 0.0, 2.0]),
            (Bitdepth::I32, vec![-40_000.0, 0.0, 40_000.0]),
        ] {
            let mut expected = Polynomial::new(
                points.len(),
                points[0],
                points[points.len() - 1],
                polynomial_type.clone(),
                bitdepth,
            );
            expected.data_points = points.clone();
            let payload = expected.to_bytes();

            let decoded = Polynomial::try_decompress(&payload)
                .expect("valid direct Polynomial/IDW payload must decode");
            assert_eq!(decoded, expected);

            let (borrowed, consumed) =
                bincode::borrow_decode_from_slice::<Polynomial, _>(&payload, BinConfig::get())
                    .expect("valid borrowed Polynomial/IDW payload must decode");
            assert_eq!(consumed, payload.len());
            assert_eq!(borrowed, expected);

            let output = match &polynomial_type {
                PolynomialType::Polynomial => {
                    Compressor::Polynomial.try_decompress(points.len(), &payload)
                }
                PolynomialType::Idw => Compressor::Idw.try_decompress(points.len(), &payload),
            }
            .expect("valid frame-aware Polynomial/IDW payload must decode");
            assert_eq!(output, points);
        }
    }
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
fn malformed_fft_fixture_with_out_of_range_position_is_rejected() {
    let mut fft = FFT::new(1, 0.0, 1.0);
    fft.frequencies
        .push(FrequencyPoint::from_complex_with_position(
            Complex { re: 1.0, im: 0.0 },
            u16::MAX,
        ));
    let malformed_fixture = fft.to_bytes();

    assert!(matches!(
        Compressor::FFT.try_decompress(1, &malformed_fixture),
        Err(DecodeError::InvalidFrame {
            codec: Compressor::FFT,
            ..
        })
    ));
}

#[test]
fn fft_overflowing_sample_count_returns_typed_error_without_allocating() {
    let fft = FFT::new(1, 0.0, 1.0);

    assert!(matches!(
        Compressor::FFT.try_decompress(usize::MAX, &fft.to_bytes()),
        Err(DecodeError::InvalidFrame {
            codec: Compressor::FFT,
            ..
        })
    ));
}

#[test]
fn constant_decode_reports_capacity_failure_instead_of_panicking() {
    let payload = Constant::new(1, 7.0, Bitdepth::U8).to_bytes();

    assert_allocation_error(Compressor::Constant.try_decompress(usize::MAX, &payload));
}

#[test]
fn rle_decode_reports_output_capacity_failure_instead_of_panicking() {
    let payload = IndexRLE::new(&[7.0], Bitdepth::U8).to_bytes();

    assert_allocation_error(Compressor::RLE.try_decompress(usize::MAX, &payload));
}

#[test]
fn fft_decode_reports_scratch_capacity_failure_instead_of_panicking() {
    let samples = 3_usize
        .checked_mul(1_usize << (usize::BITS - 3))
        .and_then(|value| value.checked_sub(1))
        .expect("test sample count must fit");
    let mut fft = FFT::new(1, 0.0, 1.0);
    fft.frequencies
        .push(FrequencyPoint::from_complex_with_position(
            Complex { re: 1.0, im: 0.0 },
            0,
        ));

    assert_allocation_error(Compressor::FFT.try_decompress(samples, &fft.to_bytes()));
}

#[test]
fn decoder_allocation_failures_are_typed_and_transactional_for_all_public_paths() {
    let stream = huge_constant_stream();
    let mut decoder = Decoder::new();

    assert_allocation_error(decoder.decode(&stream));

    let prefix = vec![-1.0, -2.0];
    let mut full_output = prefix.clone();
    assert_allocation_error(decoder.decode_into(&stream, &mut full_output));
    assert_eq!(full_output, prefix);

    let mut frame_output = prefix.clone();
    assert_allocation_error(decoder.decode_frame_into(&stream, 0, &mut frame_output));
    assert_eq!(frame_output, prefix);

    let mut range_output = prefix.clone();
    assert_allocation_error(decoder.decode_range_into(&stream, 0..usize::MAX, &mut range_output));
    assert_eq!(range_output, prefix);

    let mut partial_range_output = prefix.clone();
    assert_allocation_error(decoder.decode_range_into(&stream, 0..1, &mut partial_range_output));
    assert_eq!(partial_range_output, prefix);
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
fn forced_constant_polynomial_roundtrips() {
    let samples = [7.0; 16];
    let mut stream = CompressedStream::new();
    stream.compress_chunk_with(&samples, Compressor::Polynomial);

    let parsed = CompressedStream::try_from_bytes(&stream.to_bytes())
        .expect("constant Polynomial stream must parse");
    assert_eq!(
        parsed
            .try_decompress()
            .expect("constant Polynomial stream must decompress"),
        samples
    );
}

#[test]
fn forced_constant_idw_roundtrips() {
    let samples = [7.0; 16];
    let mut stream = CompressedStream::new();
    stream.compress_chunk_with(&samples, Compressor::Idw);

    let parsed = CompressedStream::try_from_bytes(&stream.to_bytes())
        .expect("constant IDW stream must parse");
    assert_eq!(
        parsed
            .try_decompress()
            .expect("constant IDW stream must decompress"),
        samples
    );
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
fn auto_max_sample_count_returns_typed_error_without_allocating() {
    assert!(matches!(
        Compressor::Auto.try_decompress(usize::MAX, &[]),
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

#[test]
fn frame_size_is_opaque_legacy_metadata_and_does_not_affect_decoding() {
    let stream = stream_from_encoded_frames(vec![EncodedFrame {
        frame_size: usize::MAX,
        sample_count: 3,
        compressor: Compressor::Constant,
        data: Constant::new(3, 7.0, Bitdepth::U8).to_bytes(),
    }]);

    assert_eq!(stream.try_decompress().unwrap(), [7.0, 7.0, 7.0]);
}
