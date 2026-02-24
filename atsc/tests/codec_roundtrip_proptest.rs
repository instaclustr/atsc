use atsc::codec::Codec;
use proptest::prelude::*;

proptest! {
    #[test]
    fn noop_roundtrip_is_lossless(data in proptest::collection::vec(-1.0e6f64..1.0e6f64, 1..2048)) {
        let frame = atsc::codec::noop::NoopCodec
            .compress(&data, &atsc::codec::CompressConfig::default())
            .unwrap();

        let out = atsc::codec::noop::NoopCodec
            .decompress(&frame.payload, frame.sample_count)
            .unwrap();

        prop_assert_eq!(out, data);
    }
}
