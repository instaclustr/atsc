//! Polynomial (CatmullRom) codec.
//!
//! These codecs store a subset of samples and reconstruct the remaining points
//! using interpolation. The payload layout follows `PLAN.md` (v2 wire format).

use crate::bytes::{take_f64_le, take_u32_le, take_u8};
use crate::codec::{Codec, CompressConfig, CompressedFrame};
use crate::error::{Error, Result};
use crate::metrics::nrmse;
use splines::{Interpolation, Key, Spline};

/// Polynomial codec (CatmullRom) (codec id 4).
#[derive(Clone, Copy, Debug, Default)]
pub struct PolynomialCodec;

impl PolynomialCodec {
    /// Wire format codec id.
    pub const ID: u8 = 4;
}

impl Codec for PolynomialCodec {
    fn id(&self) -> u8 {
        Self::ID
    }

    fn name(&self) -> &'static str {
        "polynomial"
    }

    fn compress(&self, data: &[f64], config: &CompressConfig) -> Result<CompressedFrame> {
        compress_impl(Self::ID, data, config)
    }

    fn decompress(&self, payload: &[u8], sample_count: u32) -> Result<Vec<f64>> {
        decompress_impl(payload, sample_count)
    }
}

const BITDEPTH_F64: u8 = 3;

fn compress_impl(codec_id: u8, data: &[f64], config: &CompressConfig) -> Result<CompressedFrame> {
    let (min, max) = finite_min_max(data)?;
    let sample_count: u32 = data
        .len()
        .try_into()
        .map_err(|_| Error::SampleCountOverflow {
            count: data.len() as u64,
        })?;

    let baseline_points = (data.len() / 100).max(3).max(1);
    let ctx = CompressCtx { min, max, data };

    let (payload, measured_error) = match config.max_error {
        None => compress_once(&ctx, baseline_points)?,
        Some(target) => compress_bounded(&ctx, target, config.max_iterations, baseline_points)?,
    };

    Ok(CompressedFrame {
        codec_id,
        sample_count,
        payload,
        measured_error,
    })
}

#[derive(Clone, Copy, Debug)]
struct CompressCtx<'a> {
    min: f64,
    max: f64,
    data: &'a [f64],
}

fn compress_bounded(
    ctx: &CompressCtx<'_>,
    target: f64,
    max_iterations: u32,
    baseline_points: usize,
) -> Result<(Vec<u8>, f64)> {
    if max_iterations == 0 {
        return Err(Error::ErrorBoundNotMet {
            best: f64::INFINITY,
            target,
        });
    }

    let mut best_payload = None;
    let mut best_error = f64::INFINITY;
    let eps = 1e-12;

    let data_len = ctx.data.len();
    let mut jump = 0usize;
    for i in 0..(max_iterations as usize) {
        let mut points = baseline_points + jump;
        if points >= data_len {
            points = data_len;
        }

        let (payload, err) = compress_once(ctx, points)?;
        if err < best_error {
            best_error = err;
            best_payload = Some(payload);
        }
        if best_error <= target || (best_error - target).abs() < eps {
            return Ok((best_payload.expect("best_payload set"), best_error));
        }

        if i < 17 {
            jump = jump.saturating_add((data_len / 10).max(1));
        } else {
            jump = jump.saturating_add((data_len / 100).max(1));
        }
    }

    Err(Error::ErrorBoundNotMet {
        best: best_error,
        target,
    })
}

fn compress_once(ctx: &CompressCtx<'_>, points: usize) -> Result<(Vec<u8>, f64)> {
    let (point_step, samples) = select_samples(ctx.data, points);

    let reconstructed = reconstruct(ctx.min, ctx.max, point_step, &samples, ctx.data.len())?;
    let err = nrmse(ctx.data, &reconstructed)?;
    let payload = encode_payload(ctx.min, ctx.max, point_step, &samples)?;
    Ok((payload, err))
}

fn select_samples(data: &[f64], points: usize) -> (u32, Vec<f64>) {
    let len = data.len();
    let points = points.max(1).min(len.max(1));
    let step = (len / points).max(1);

    let mut values: Vec<f64> = (0..len).step_by(step).map(|idx| data[idx]).collect();
    if len > 0 && (len - 1) % step != 0 {
        values.push(data[len - 1]);
    }
    (step as u32, values)
}

fn reconstruct(
    min: f64,
    max: f64,
    point_step: u32,
    samples: &[f64],
    frame_size: usize,
) -> Result<Vec<f64>> {
    if frame_size == 0 {
        return Err(Error::EmptyData);
    }
    let step = usize::try_from(point_step)
        .map_err(|_| Error::ResourceLimitExceeded("point_step does not fit usize".into()))?;
    if step == 0 {
        return Err(Error::ResourceLimitExceeded(
            "point_step must be >= 1".into(),
        ));
    }

    let mut positions: Vec<usize> = (0..frame_size).step_by(step).collect();
    if positions.last().copied() != Some(frame_size - 1) {
        positions.push(frame_size - 1);
    }
    if positions.len() != samples.len() {
        return Err(Error::ResourceLimitExceeded(
            "payload point_count does not match implied positions".into(),
        ));
    }

    let out = interpolate_catmull_rom(&positions, samples, frame_size)?;

    Ok(out.into_iter().map(|v| v.clamp(min, max)).collect())
}

fn interpolate_catmull_rom(
    positions: &[usize],
    samples: &[f64],
    frame_size: usize,
) -> Result<Vec<f64>> {
    let mut keys = Vec::with_capacity(positions.len());
    for (idx, (&x, &y)) in positions.iter().zip(samples).enumerate() {
        let interp = if idx > 0 && (positions.len() - idx) > 2 {
            Interpolation::CatmullRom
        } else {
            Interpolation::Linear
        };
        keys.push(Key::new(x as f64, y, interp));
    }
    let spline = Spline::from_vec(keys);

    let mut out = Vec::with_capacity(frame_size);
    let mut prev = *samples.first().ok_or(Error::EmptyData)?;
    for i in 0..frame_size {
        let v = spline.clamped_sample(i as f64).unwrap_or(prev);
        prev = v;
        out.push(v);
    }
    Ok(out)
}

fn encode_payload(min: f64, max: f64, point_step: u32, samples: &[f64]) -> Result<Vec<u8>> {
    let point_count: u32 = samples
        .len()
        .try_into()
        .map_err(|_| Error::ResourceLimitExceeded("point_count exceeds u32::MAX".into()))?;

    let mut out = Vec::with_capacity(8 + 8 + 4 + 1 + 4 + samples.len() * 8);
    out.extend_from_slice(&min.to_le_bytes());
    out.extend_from_slice(&max.to_le_bytes());
    out.extend_from_slice(&point_step.to_le_bytes());
    out.push(BITDEPTH_F64);
    out.extend_from_slice(&point_count.to_le_bytes());
    for &v in samples {
        out.extend_from_slice(&v.to_le_bytes());
    }
    Ok(out)
}

fn decompress_impl(payload: &[u8], sample_count: u32) -> Result<Vec<f64>> {
    let frame_size: usize = sample_count
        .try_into()
        .map_err(|_| Error::ResourceLimitExceeded("sample_count does not fit usize".into()))?;
    let mut off = 0usize;
    let min = take_f64_le(payload, &mut off)?;
    let max = take_f64_le(payload, &mut off)?;
    let point_step = take_u32_le(payload, &mut off)?;
    let bitdepth = take_u8(payload, &mut off)?;
    let point_count_u32 = take_u32_le(payload, &mut off)?;
    let point_count: usize = point_count_u32
        .try_into()
        .map_err(|_| Error::ResourceLimitExceeded("point_count does not fit usize".into()))?;

    if bitdepth != BITDEPTH_F64 {
        return Err(Error::PayloadCorrupt(
            "unsupported polynomial bitdepth".into(),
        ));
    }

    let mut samples = Vec::with_capacity(point_count);
    for _ in 0..point_count {
        samples.push(take_f64_le(payload, &mut off)?);
    }
    if off != payload.len() {
        return Err(Error::PayloadCorrupt(
            "polynomial payload has trailing bytes".into(),
        ));
    }

    reconstruct(min, max, point_step, &samples, frame_size)
}

fn finite_min_max(data: &[f64]) -> Result<(f64, f64)> {
    let (first, rest) = data.split_first().ok_or(Error::EmptyData)?;
    if !first.is_finite() {
        return Err(Error::InvalidInput);
    }
    let mut min = *first;
    let mut max = *first;
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
    }
    Ok((min, max))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn polynomial_roundtrip_sanity() {
        let data: Vec<f64> = (0..1024).map(|i| i as f64).collect();
        let cfg = CompressConfig {
            max_error: None,
            ..Default::default()
        };
        let frame = PolynomialCodec.compress(&data, &cfg).unwrap();
        let out = PolynomialCodec
            .decompress(&frame.payload, frame.sample_count)
            .unwrap();
        assert_eq!(out.len(), data.len());
        assert!(out.iter().all(|v| v.is_finite()));
    }

    #[test]
    fn rejects_unsupported_bitdepth() {
        let mut payload = Vec::new();
        payload.extend_from_slice(&0.0f64.to_le_bytes()); // min
        payload.extend_from_slice(&1.0f64.to_le_bytes()); // max
        payload.extend_from_slice(&1u32.to_le_bytes()); // point_step
        payload.push(0); // bitdepth (unsupported)
        payload.extend_from_slice(&1u32.to_le_bytes()); // point_count
        payload.extend_from_slice(&0.5f64.to_le_bytes()); // one sample

        assert!(matches!(
            PolynomialCodec.decompress(&payload, 1),
            Err(Error::PayloadCorrupt(_))
        ));
    }
}
