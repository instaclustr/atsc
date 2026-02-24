//! Constant-value codec.

use crate::codec::{Codec, CompressConfig, CompressedFrame};
use crate::error::{Error, Result};

/// Constant codec (codec id 1).
#[derive(Clone, Copy, Debug, Default)]
pub struct ConstantCodec;

impl ConstantCodec {
    /// Wire format codec id.
    pub const ID: u8 = 1;
}

impl Codec for ConstantCodec {
    fn id(&self) -> u8 {
        Self::ID
    }

    fn name(&self) -> &'static str {
        "constant"
    }

    fn compress(&self, data: &[f64], config: &CompressConfig) -> Result<CompressedFrame> {
        let (first, rest) = data.split_first().ok_or(Error::EmptyData)?;
        if !first.is_finite() {
            return Err(Error::InvalidInput);
        }

        let mut min = *first;
        let mut max = *first;
        let mut sum_sq = 0.0f64;
        for &v in rest {
            if !v.is_finite() {
                return Err(Error::InvalidInput);
            }
            if v < min {
                min = v;
            }
            if v > max {
                max = v;
            }
            let d = v - first;
            sum_sq += d * d;
        }

        let sample_count: u32 = data
            .len()
            .try_into()
            .map_err(|_| Error::SampleCountOverflow {
                count: data.len() as u64,
            })?;

        let mse = sum_sq / data.len() as f64;
        let rmse = mse.sqrt();
        let denom = max - min;
        let measured_error = if denom == 0.0 {
            if mse == 0.0 {
                0.0
            } else {
                f64::INFINITY
            }
        } else {
            rmse / denom
        };

        if let Some(target) = config.max_error {
            if measured_error > target {
                return Err(Error::ErrorBoundNotMet {
                    best: measured_error,
                    target,
                });
            }
        }

        Ok(CompressedFrame {
            codec_id: Self::ID,
            sample_count,
            payload: first.to_le_bytes().to_vec(),
            measured_error,
        })
    }

    fn decompress(&self, payload: &[u8], sample_count: u32) -> Result<Vec<f64>> {
        if payload.len() != 8 {
            return Err(Error::UnexpectedEof {
                offset: 0,
                expected: 8,
            });
        }
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&payload[0..8]);
        let value = f64::from_le_bytes(bytes);
        if !value.is_finite() {
            return Err(Error::InvalidInput);
        }

        let len: usize = sample_count
            .try_into()
            .map_err(|_| Error::ResourceLimitExceeded("sample_count does not fit usize".into()))?;
        Ok(vec![value; len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_roundtrip() {
        let data = vec![2.0; 1024];
        let cfg = CompressConfig {
            max_error: Some(0.0),
            ..Default::default()
        };
        let frame = ConstantCodec.compress(&data, &cfg).unwrap();
        assert_eq!(frame.codec_id, ConstantCodec::ID);
        assert_eq!(frame.sample_count, 1024);
        assert_eq!(frame.payload.len(), 8);
        assert_eq!(frame.measured_error, 0.0);

        let out = ConstantCodec
            .decompress(&frame.payload, frame.sample_count)
            .unwrap();
        assert_eq!(out, data);
    }
}
