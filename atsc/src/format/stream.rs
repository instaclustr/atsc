//! Stream encode/decode entry points.

use crate::codec::CompressedFrame;
use crate::bytes::{take_bytes, take_u32_le, take_u8};
use crate::error::{Error, Result};
use crate::format::footer::{Footer, FOOTER_LEN, FOOTER_MAGIC};
use crate::format::frame::FRAME_HEADER_LEN;
use crate::format::header::{Header, HEADER_LEN, MAGIC};
use crate::format::limits::{MAX_FRAMES, MAX_PAYLOAD_BYTES, MAX_TOTAL_SAMPLES};

/// Encode a complete stream in one shot.
///
/// # Errors
/// Returns `Err` on encode failure.
#[must_use = "handle the Result to observe encode failures"]
pub fn encode_stream(header: &Header, frames: &[CompressedFrame]) -> Result<Vec<u8>> {
    let frame_count: u32 = frames
        .len()
        .try_into()
        .map_err(|_| Error::ResourceLimitExceeded("frame count exceeds u32::MAX".to_string()))?;

    let mut out = Vec::new();
    out.extend_from_slice(&MAGIC);
    out.push(header.version);
    out.push(header.flags);

    for frame in frames {
        out.push(frame.codec_id);
        out.extend_from_slice(&frame.sample_count.to_le_bytes());
        let payload_len: u32 = frame.payload.len().try_into().map_err(|_| {
            Error::ResourceLimitExceeded("payload length exceeds u32::MAX".to_string())
        })?;
        out.extend_from_slice(&payload_len.to_le_bytes());
        out.extend_from_slice(&frame.payload);
    }

    out.extend_from_slice(&FOOTER_MAGIC);
    out.extend_from_slice(&frame_count.to_le_bytes());
    Ok(out)
}

/// Decode the stream header.
///
/// # Errors
/// Returns `Err` if the header is invalid, truncated, or uses an unsupported version.
#[must_use = "handle the Result to observe decode failures"]
pub fn decode_header(buf: &[u8]) -> Result<Header> {
    if buf.len() < HEADER_LEN {
        return Err(Error::UnexpectedEof {
            offset: 0,
            expected: HEADER_LEN as u64,
        });
    }
    if buf[0..4] != MAGIC {
        return Err(Error::InvalidMagic);
    }
    let version = buf[4];
    if version != 1 {
        return Err(Error::UnsupportedVersion(version));
    }
    let flags = buf[5];
    Ok(Header { version, flags })
}

/// Decode a frame starting at `*offset`, advancing `offset` past the payload.
///
/// # Errors
/// Returns `Err` if the frame header/payload is truncated or violates resource limits.
#[must_use = "handle the Result to observe decode failures"]
pub fn decode_frame(buf: &[u8], offset: &mut usize) -> Result<CompressedFrame> {
    let codec_id = take_u8(buf, offset)?;
    let sample_count = take_u32_le(buf, offset)?;
    let payload_len = take_u32_le(buf, offset)?;

    if payload_len > MAX_PAYLOAD_BYTES {
        return Err(Error::ResourceLimitExceeded(format!(
            "payload_len {payload_len} exceeds MAX_PAYLOAD_BYTES"
        )));
    }

    let payload = take_bytes(buf, offset, payload_len as usize)?.to_vec();
    Ok(CompressedFrame {
        codec_id,
        sample_count,
        payload,
        measured_error: 0.0,
    })
}

/// Decode the footer (if present).
///
/// # Errors
/// Returns `Err` if the buffer is too short to contain a footer when the footer magic matches.
#[must_use = "handle the Result to observe decode failures"]
pub fn decode_footer(buf: &[u8]) -> Result<Option<Footer>> {
    if buf.len() < FOOTER_LEN {
        return Ok(None);
    }

    let start = buf.len() - FOOTER_LEN;
    if buf[start..start + 4] != FOOTER_MAGIC {
        return Ok(None);
    }

    let mut count_bytes = [0u8; 4];
    count_bytes.copy_from_slice(&buf[start + 4..start + 8]);
    let frame_count = u32::from_le_bytes(count_bytes);

    Ok(Some(Footer { frame_count }))
}

/// Decode a complete stream in one shot.
///
/// If the footer is present, it is used as a fast-path to avoid scanning EOF.
///
/// # Errors
/// Returns `Err` on invalid/truncated input or resource limit violations.
#[must_use = "handle the Result to observe decode failures"]
pub fn decode_stream(buf: &[u8]) -> Result<(Header, Vec<CompressedFrame>)> {
    let header = decode_header(buf)?;
    let footer = decode_footer(buf)?;
    let end = footer.map(|_| buf.len() - FOOTER_LEN).unwrap_or(buf.len());

    let mut offset = HEADER_LEN;
    let mut frames = Vec::new();
    let mut total_samples: u64 = 0;

    while offset < end {
        if frames.len() as u32 >= MAX_FRAMES {
            return Err(Error::ResourceLimitExceeded(format!(
                "frame count exceeds MAX_FRAMES ({MAX_FRAMES})"
            )));
        }

        let frame = decode_frame_end_limited(buf, &mut offset, end)?;
        total_samples = total_samples.saturating_add(frame.sample_count as u64);
        if total_samples > MAX_TOTAL_SAMPLES {
            return Err(Error::ResourceLimitExceeded(format!(
                "total samples {total_samples} exceeds MAX_TOTAL_SAMPLES"
            )));
        }
        frames.push(frame);
    }

    Ok((header, frames))
}

fn decode_frame_end_limited(buf: &[u8], offset: &mut usize, end: usize) -> Result<CompressedFrame> {
    if end < *offset + FRAME_HEADER_LEN {
        return Err(Error::UnexpectedEof {
            offset: *offset as u64,
            expected: FRAME_HEADER_LEN as u64,
        });
    }

    let mut tmp_off = *offset;
    let codec_id = take_u8(buf, &mut tmp_off)?;
    let sample_count = take_u32_le(buf, &mut tmp_off)?;
    let payload_len = take_u32_le(buf, &mut tmp_off)?;

    if payload_len > MAX_PAYLOAD_BYTES {
        return Err(Error::ResourceLimitExceeded(format!(
            "payload_len {payload_len} exceeds MAX_PAYLOAD_BYTES"
        )));
    }

    let payload_len_usize = payload_len as usize;
    if end < tmp_off + payload_len_usize {
        return Err(Error::UnexpectedEof {
            offset: tmp_off as u64,
            expected: payload_len_usize as u64,
        });
    }

    let payload = buf[tmp_off..tmp_off + payload_len_usize].to_vec();
    tmp_off += payload_len_usize;
    *offset = tmp_off;

    Ok(CompressedFrame {
        codec_id,
        sample_count,
        payload,
        measured_error: 0.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_roundtrip() {
        let h = Header {
            version: 1,
            flags: 0b1010_0001,
        };
        let bytes = encode_stream(&h, &[]).unwrap();
        let decoded = decode_header(&bytes).unwrap();
        assert_eq!(decoded, h);
    }

    #[test]
    fn decode_header_rejects_bad_magic() {
        let mut buf = vec![0u8; HEADER_LEN];
        buf[..4].copy_from_slice(b"NOPE");
        buf[4] = 1;
        buf[5] = 0;
        assert!(matches!(decode_header(&buf), Err(Error::InvalidMagic)));
    }

    #[test]
    fn decode_footer_absent_is_ok() {
        let h = Header {
            version: 1,
            flags: 0,
        };
        let bytes = {
            let mut out = Vec::new();
            out.extend_from_slice(&MAGIC);
            out.push(h.version);
            out.push(h.flags);
            out
        };
        assert!(decode_footer(&bytes).unwrap().is_none());
    }

    #[test]
    fn decode_frame_truncated_payload_is_eof() {
        let h = Header {
            version: 1,
            flags: 0,
        };
        let mut out = Vec::new();
        out.extend_from_slice(&MAGIC);
        out.push(h.version);
        out.push(h.flags);

        out.push(0); // codec_id
        out.extend_from_slice(&1u32.to_le_bytes()); // sample_count
        out.extend_from_slice(&10u32.to_le_bytes()); // payload_len
        out.extend_from_slice(&[0u8; 3]); // payload (truncated)

        let mut off = HEADER_LEN;
        let err = decode_frame(&out, &mut off).unwrap_err();
        assert!(matches!(err, Error::UnexpectedEof { .. }));
    }

    #[test]
    fn golden_constant_stream_bytes() {
        let header = Header {
            version: 1,
            flags: 0,
        };
        let payload = 2.5f64.to_le_bytes().to_vec();
        let frames = [CompressedFrame {
            codec_id: 1,
            sample_count: 3,
            payload,
            measured_error: 0.0,
        }];

        let bytes = encode_stream(&header, &frames).unwrap();

        let expected: [u8; 31] = [
            b'A', b'T', b'S', b'C', 1, 0, // header
            1, // codec_id
            3, 0, 0, 0, // sample_count
            8, 0, 0, 0, // payload_len
            0, 0, 0, 0, 0, 0, 4, 64, // f64 2.5
            b'C', b'S', b'T', b'A', 1, 0, 0, 0, // footer
        ];
        assert_eq!(bytes, expected);

        let (h2, decoded) = decode_stream(&bytes).unwrap();
        assert_eq!(h2, header);
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].codec_id, 1);
        assert_eq!(decoded[0].sample_count, 3);
        assert_eq!(decoded[0].payload.len(), 8);
    }

    #[test]
    fn decode_rejects_oversize_payload_len_without_alloc() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&MAGIC);
        bytes.push(1);
        bytes.push(0);

        bytes.push(0); // codec_id
        bytes.extend_from_slice(&1u32.to_le_bytes()); // sample_count
        bytes.extend_from_slice(&(MAX_PAYLOAD_BYTES + 1).to_le_bytes()); // payload_len

        let err = decode_stream(&bytes).unwrap_err();
        assert!(matches!(err, Error::ResourceLimitExceeded(_)));
    }
}
