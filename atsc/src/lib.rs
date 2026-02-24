//! ATSC v2 time-series compression library.
//!
//! This crate is a library-first redesign of ATSC with a manual wire format and
//! hardened decode paths. See `PLAN.md` for the full architecture and format
//! specification.

pub mod codec;
mod bytes;
pub mod error;
pub mod format;
pub mod metrics;
pub mod optimizer;
pub mod vsri;

pub use error::{Error, Result};

use crate::codec::fft::FftF32Codec;
use crate::codec::noop::NoopCodec;
use crate::codec::polynomial::{IdwCodec, PolynomialCodec};
use crate::codec::{Codec, CompressConfig, CompressedFrame};
use crate::format::header::Header;
use crate::format::stream::{decode_stream, encode_stream};
use crate::optimizer::chunker::Plan;
use crate::optimizer::select_codec;
use crate::vsri::Vsri;

const VSRI_CODEC_ID: u8 = 128;
const FLAG_INLINE_VSRI: u8 = 1 << 0;

/// Compress f64 time-series data into an ATSC v2 stream.
///
/// # Errors
/// - Returns `Err(Error::EmptyData)` when the (filtered) input is empty.
/// - Returns `Err(Error::InvalidInput)` in strict mode when NaN/Inf are present.
#[must_use = "handle the Result to observe compression errors"]
pub fn compress(data: &[f64], config: &CompressConfig) -> Result<Vec<u8>> {
    let filtered = filter_nan_inf(data, config.reject_nan_inf)?;
    if filtered.is_empty() {
        return Err(Error::EmptyData);
    }

    let codecs = default_codecs();
    let plan = Plan::new(filtered.len());
    let mut frames = Vec::with_capacity(plan.chunks.len());

    for range in plan.chunks {
        let frame = select_codec(&filtered[range], &codecs, config)?;
        frames.push(frame);
    }

    let header = Header {
        version: 1,
        flags: 0,
    };
    encode_stream(&header, &frames)
}

/// Decompress an ATSC v2 stream back into f64 values.
///
/// # Errors
/// Returns `Err` when the input is truncated/invalid or uses an unknown codec id.
#[must_use = "handle the Result to observe decompression errors"]
pub fn decompress(bytes: &[u8]) -> Result<Vec<f64>> {
    let (_header, frames) = decode_stream(bytes)?;
    let mut out = Vec::new();

    for frame in frames {
        if frame.codec_id == VSRI_CODEC_ID {
            return Err(Error::VsriInvariantViolation(
                "vsri frame present in values-only stream".into(),
            ));
        }
        let mut decoded = decompress_frame(&frame)?;
        out.append(&mut decoded);
    }
    Ok(out)
}

/// Compress values and timestamps together (inline VSRI as first frame).
///
/// # Errors
/// - Returns `Err` when lengths differ or timestamps are not strictly increasing.
/// - Returns `Err` when compression fails.
#[must_use = "handle the Result to observe compression errors"]
pub fn compress_with_timestamps(
    timestamps: &[i64],
    values: &[f64],
    config: &CompressConfig,
) -> Result<Vec<u8>> {
    if timestamps.len() != values.len() {
        return Err(Error::VsriInvariantViolation(
            "timestamps and values length mismatch".into(),
        ));
    }

    let filtered_values = filter_nan_inf(values, config.reject_nan_inf)?;
    if filtered_values.len() != timestamps.len() {
        // Filtering would desync timestamps/values; require strict mode or pre-filtering by caller.
        return Err(Error::InvalidInput);
    }

    let vsri = Vsri::from_timestamps(timestamps)?;
    let vsri_payload = vsri.encode()?;

    let sample_count: u32 = values
        .len()
        .try_into()
        .map_err(|_| Error::SampleCountOverflow {
            count: values.len() as u64,
        })?;

    let mut frames = Vec::new();
    frames.push(CompressedFrame {
        codec_id: VSRI_CODEC_ID,
        sample_count,
        payload: vsri_payload,
        measured_error: 0.0,
    });

    let codecs = default_codecs();
    let plan = Plan::new(filtered_values.len());
    for range in plan.chunks {
        let frame = select_codec(&filtered_values[range], &codecs, config)?;
        frames.push(frame);
    }

    let header = Header {
        version: 1,
        flags: FLAG_INLINE_VSRI,
    };
    encode_stream(&header, &frames)
}

/// Decompress values and timestamps from an ATSC v2 stream with inline VSRI.
///
/// # Errors
/// Returns `Err` when invariants are violated, input is invalid, or codecs are unknown.
#[must_use = "handle the Result to observe decompression errors"]
pub fn decompress_with_timestamps(bytes: &[u8]) -> Result<(Vec<i64>, Vec<f64>)> {
    let (header, frames) = decode_stream(bytes)?;

    if (header.flags & FLAG_INLINE_VSRI) == 0 {
        return Err(Error::VsriInvariantViolation(
            "stream header does not indicate inline VSRI".into(),
        ));
    }

    let (first, rest) = frames
        .split_first()
        .ok_or_else(|| Error::VsriInvariantViolation("missing VSRI frame".into()))?;
    if first.codec_id != VSRI_CODEC_ID {
        return Err(Error::VsriInvariantViolation(
            "first frame must be VSRI when flag is set".into(),
        ));
    }
    let vsri = Vsri::decode(&first.payload)?;
    let timestamps = vsri.timestamps()?;

    let mut values = Vec::new();
    for frame in rest {
        if frame.codec_id == VSRI_CODEC_ID {
            return Err(Error::VsriInvariantViolation(
                "unexpected VSRI frame after first frame".into(),
            ));
        }
        let mut decoded = decompress_frame(frame)?;
        values.append(&mut decoded);
    }

    if values.len() != timestamps.len() {
        return Err(Error::VsriInvariantViolation(
            "vsri sample count does not match values".into(),
        ));
    }

    Ok((timestamps, values))
}

/// Stream writer for incremental frame construction.
pub struct StreamWriter {
    config: CompressConfig,
    frames: Vec<CompressedFrame>,
    has_vsri: bool,
}

impl StreamWriter {
    /// Create a new stream writer.
    #[must_use]
    pub fn new(config: CompressConfig) -> Self {
        Self {
            config,
            frames: Vec::new(),
            has_vsri: false,
        }
    }

    /// Prepend an inline VSRI frame.
    ///
    /// # Errors
    /// Returns `Err` if timestamps are invalid or VSRI is already present.
    pub fn set_timestamps(&mut self, timestamps: &[i64]) -> Result<()> {
        if self.has_vsri || !self.frames.is_empty() {
            return Err(Error::VsriInvariantViolation(
                "VSRI must be set before pushing value frames".into(),
            ));
        }
        let vsri = Vsri::from_timestamps(timestamps)?;
        let payload = vsri.encode()?;
        let sample_count: u32 =
            timestamps
                .len()
                .try_into()
                .map_err(|_| Error::SampleCountOverflow {
                    count: timestamps.len() as u64,
                })?;
        self.frames.push(CompressedFrame {
            codec_id: VSRI_CODEC_ID,
            sample_count,
            payload,
            measured_error: 0.0,
        });
        self.has_vsri = true;
        Ok(())
    }

    /// Compress and append one value frame.
    ///
    /// # Errors
    /// Returns `Err` on compression failure.
    pub fn push(&mut self, data: &[f64]) -> Result<()> {
        let filtered = filter_nan_inf(data, self.config.reject_nan_inf)?;
        if filtered.is_empty() {
            return Err(Error::EmptyData);
        }
        let codecs = default_codecs();
        let frame = select_codec(&filtered, &codecs, &self.config)?;
        self.frames.push(frame);
        Ok(())
    }

    /// Finish the stream and return encoded bytes.
    ///
    /// # Errors
    /// Returns `Err` on encode failure.
    pub fn finish(self) -> Result<Vec<u8>> {
        let header = Header {
            version: 1,
            flags: if self.has_vsri { FLAG_INLINE_VSRI } else { 0 },
        };
        encode_stream(&header, &self.frames)
    }
}

/// Stream reader (eager decode).
pub struct StreamReader<'a> {
    header: Header,
    frames: Vec<CompressedFrame>,
    _bytes: &'a [u8],
}

impl<'a> StreamReader<'a> {
    /// Create a new reader over an encoded stream.
    ///
    /// # Errors
    /// Returns `Err` on decode failure.
    #[must_use = "handle the Result to observe decode failures"]
    pub fn new(bytes: &'a [u8]) -> Result<Self> {
        let (header, frames) = decode_stream(bytes)?;
        Ok(Self {
            header,
            frames,
            _bytes: bytes,
        })
    }

    /// Borrow the decoded header.
    #[must_use]
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// Number of frames in this stream (scanned if footer absent).
    #[must_use]
    pub fn frame_count(&self) -> u32 {
        self.frames.len() as u32
    }

    /// Borrow decoded frames.
    #[must_use]
    pub fn frames(&self) -> &[CompressedFrame] {
        &self.frames
    }
}

/// Inspect an encoded stream without decompressing payloads.
///
/// # Errors
/// Returns `Err` on decode failures.
#[must_use = "handle the Result to observe inspect failures"]
pub fn inspect(bytes: &[u8]) -> Result<StreamInfo> {
    let (header, frames) = decode_stream(bytes)?;
    let mut total_samples: u64 = 0;
    let mut infos = Vec::with_capacity(frames.len());
    for f in &frames {
        total_samples = total_samples.saturating_add(f.sample_count as u64);
        infos.push(FrameInfo {
            codec_name: codec_name(f.codec_id),
            codec_id: f.codec_id,
            sample_count: f.sample_count,
            payload_bytes: f.payload.len().try_into().map_err(|_| {
                Error::ResourceLimitExceeded("payload_bytes exceeds u32::MAX".into())
            })?,
        });
    }

    Ok(StreamInfo {
        version: header.version,
        frame_count: frames.len() as u32,
        total_samples,
        has_vsri: (header.flags & FLAG_INLINE_VSRI) != 0,
        frames: infos,
    })
}

/// Stream metadata returned by `inspect()`.
pub struct StreamInfo {
    pub version: u8,
    pub frame_count: u32,
    pub total_samples: u64,
    pub has_vsri: bool,
    pub frames: Vec<FrameInfo>,
}

/// Frame metadata returned by `inspect()`.
pub struct FrameInfo {
    pub codec_name: &'static str,
    pub codec_id: u8,
    pub sample_count: u32,
    pub payload_bytes: u32,
}

fn default_codecs() -> [&'static dyn Codec; 4] {
    [&NoopCodec, &FftF32Codec, &PolynomialCodec, &IdwCodec]
}

fn codec_name(codec_id: u8) -> &'static str {
    match codec_id {
        0 => "noop",
        1 => "constant",
        2 => "fft-f32",
        4 => "polynomial",
        5 => "idw",
        128 => "vsri",
        _ => "unknown",
    }
}

fn decompress_frame(frame: &CompressedFrame) -> Result<Vec<f64>> {
    match frame.codec_id {
        0 => NoopCodec.decompress(&frame.payload, frame.sample_count),
        1 => crate::codec::constant::ConstantCodec.decompress(&frame.payload, frame.sample_count),
        2 => FftF32Codec.decompress(&frame.payload, frame.sample_count),
        4 => PolynomialCodec.decompress(&frame.payload, frame.sample_count),
        5 => IdwCodec.decompress(&frame.payload, frame.sample_count),
        _ => Err(Error::UnknownCodec(frame.codec_id)),
    }
}

fn filter_nan_inf(data: &[f64], reject: bool) -> Result<Vec<f64>> {
    let mut removed = 0usize;
    let mut out = Vec::with_capacity(data.len());
    for &v in data {
        if v.is_finite() {
            out.push(v);
        } else if reject {
            return Err(Error::InvalidInput);
        } else {
            removed += 1;
        }
    }
    if removed > 0 {
        log::warn!("filtered {removed} NaN/Inf samples from input");
    }
    Ok(out)
}

#[cfg(test)]
mod api_tests {
    use super::*;

    #[test]
    fn compress_decompress_roundtrip_length() {
        let data: Vec<f64> = (0..4096).map(|i| (i as f64).sin()).collect();
        let bytes = compress(&data, &CompressConfig::default()).unwrap();
        let out = decompress(&bytes).unwrap();
        assert_eq!(out.len(), data.len());
    }

    #[test]
    fn compress_with_timestamps_roundtrip_lengths() {
        let values: Vec<f64> = (0..1024).map(|i| (i as f64).cos()).collect();
        let timestamps: Vec<i64> = (0..1024).map(|i| 1_000 + i as i64 * 10).collect();
        let bytes =
            compress_with_timestamps(&timestamps, &values, &CompressConfig::default()).unwrap();
        let (ts2, v2) = decompress_with_timestamps(&bytes).unwrap();
        assert_eq!(ts2, timestamps);
        assert_eq!(v2.len(), values.len());
    }
}
