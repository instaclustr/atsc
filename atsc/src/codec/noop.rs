//! Lossless no-op codec (raw f64 payload).

use crate::codec::{Codec, CompressConfig, CompressedFrame};
use crate::error::{Error, Result};

/// Noop codec (codec id 0): stores raw f64 values as little-endian bytes.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoopCodec;

impl NoopCodec {
    /// Wire format codec id.
    pub const ID: u8 = 0;
}

impl Codec for NoopCodec {
    fn id(&self) -> u8 {
        Self::ID
    }

    fn name(&self) -> &'static str {
        "noop"
    }

    fn compress(&self, data: &[f64], _config: &CompressConfig) -> Result<CompressedFrame> {
        if data.is_empty() {
            return Err(Error::EmptyData);
        }
        for &v in data {
            if !v.is_finite() {
                return Err(Error::InvalidInput);
            }
        }

        let sample_count: u32 = data
            .len()
            .try_into()
            .map_err(|_| Error::SampleCountOverflow {
                count: data.len() as u64,
            })?;

        let mut payload = Vec::with_capacity(data.len() * 8);
        for &v in data {
            payload.extend_from_slice(&v.to_le_bytes());
        }

        Ok(CompressedFrame {
            codec_id: Self::ID,
            sample_count,
            payload,
            measured_error: 0.0,
        })
    }

    fn decompress(&self, payload: &[u8], sample_count: u32) -> Result<Vec<f64>> {
        let len: usize = sample_count
            .try_into()
            .map_err(|_| Error::ResourceLimitExceeded("sample_count does not fit usize".into()))?;

        let expected = len
            .checked_mul(8)
            .ok_or_else(|| Error::ResourceLimitExceeded("payload length overflow".into()))?;

        if payload.len() != expected {
            return Err(Error::UnexpectedEof {
                offset: 0,
                expected: expected as u64,
            });
        }

        let mut out = Vec::with_capacity(len);
        for i in 0..len {
            let start = i * 8;
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&payload[start..start + 8]);
            let v = f64::from_le_bytes(bytes);
            if !v.is_finite() {
                return Err(Error::InvalidInput);
            }
            out.push(v);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_roundtrip() {
        let data = vec![0.0, -1.5, 2.25, 999.0];
        let frame = NoopCodec
            .compress(&data, &CompressConfig::default())
            .unwrap();
        let out = NoopCodec
            .decompress(&frame.payload, frame.sample_count)
            .unwrap();
        assert_eq!(out, data);
    }
}
