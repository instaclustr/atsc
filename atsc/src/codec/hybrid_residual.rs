//! Hybrid residual-point codec.
//!
//! This codec stores a simple polynomial baseline plus sparse residual patches
//! at the highest-error positions.

use crate::bytes::{take_bytes, take_f32_le, take_f64_le, take_u16_le, take_u32_le};
use crate::codec::{Codec, CompressConfig, CompressedFrame};
use crate::error::{Error, Result};
use crate::metrics::nrmse;
use splines::{Interpolation, Key, Spline};
use std::cmp::Ordering;

const HYBRID_BOUNDED_ROUGHNESS_MAX: f64 = 0.0015;

/// Hybrid codec combining simple interpolation with sparse residual patches.
#[derive(Clone, Copy, Debug, Default)]
pub struct HybridResidualCodec;

impl HybridResidualCodec {
    /// Wire format codec id.
    pub const ID: u8 = 5;
}

impl Codec for HybridResidualCodec {
    fn id(&self) -> u8 {
        Self::ID
    }

    fn name(&self) -> &'static str {
        "hybrid-residual"
    }

    fn compress(&self, data: &[f64], config: &CompressConfig) -> Result<CompressedFrame> {
        let (min, max) = finite_min_max(data)?;
        let sample_count: u32 = data
            .len()
            .try_into()
            .map_err(|_| Error::SampleCountOverflow {
                count: data.len() as u64,
            })?;
        if let Some(target) = config.max_error {
            if config.max_iterations == 0 {
                return Err(Error::ErrorBoundNotMet {
                    best: f64::INFINITY,
                    target,
                    best_payload: None,
                });
            }
        }

        let base_points = (data.len() / 200).max(3).max(1).min(data.len().max(1));
        let (point_step, base_samples) = select_samples(data, base_points);
        let base_reconstructed = reconstruct(min, max, point_step, &base_samples, data.len())?;
        let base_payload = encode_payload(min, max, point_step, &base_samples, &[])?;
        let base_error = nrmse(data, &base_reconstructed)?;
        let roughness = normalized_roughness(data, min, max);
        if let Some(target) = config.max_error {
            if roughness > HYBRID_BOUNDED_ROUGHNESS_MAX {
                return Err(Error::ErrorBoundNotMet {
                    best: base_error,
                    target,
                    best_payload: Some(base_payload),
                });
            }
        }

        let ranked = rank_residuals(data, &base_reconstructed)?;
        let mut prefix_sq = Vec::with_capacity(ranked.len() + 1);
        prefix_sq.push(0.0);
        for item in &ranked {
            let prev = *prefix_sq.last().unwrap_or(&0.0);
            prefix_sq.push(prev + (item.residual * item.residual));
        }
        let total_sse = *prefix_sq.last().unwrap_or(&0.0);

        let candidate_ks = candidate_k_values(
            data.len(),
            ranked.len(),
            config.max_iterations,
            config.max_error.is_some(),
        );
        let mut best_frame: Option<CompressedFrame> = None;
        let mut best_bound_met = false;

        for &k in &candidate_ks {
            let estimated_error =
                estimated_nrmse_after_patches(min, max, data.len(), total_sse, &prefix_sq, k);
            let estimated_bytes = estimate_payload_bytes(base_samples.len(), &ranked[..k]);

            // Skip obviously expensive options once we already have a bound-meeting winner.
            if best_bound_met {
                if let Some(best) = &best_frame {
                    if estimated_bytes >= best.payload.len() {
                        continue;
                    }
                }
            }

            let payload = encode_payload(min, max, point_step, &base_samples, &ranked[..k])?;
            let reconstructed = apply_patches(&base_reconstructed, min, max, &ranked[..k])?;
            let measured_error = nrmse(data, &reconstructed)?;
            let frame = CompressedFrame {
                codec_id: Self::ID,
                sample_count,
                payload,
                measured_error,
            };

            let bound_met = match config.max_error {
                None => true,
                Some(target) => measured_error <= target || (measured_error - target).abs() < 1e-12,
            };

            match (&best_frame, best_bound_met, bound_met) {
                (None, _, _) => {
                    best_bound_met = bound_met;
                    best_frame = Some(frame);
                }
                (Some(_), false, true) => {
                    best_bound_met = true;
                    best_frame = Some(frame);
                }
                (Some(existing), true, true) => {
                    if frame.payload.len() < existing.payload.len()
                        || (frame.payload.len() == existing.payload.len()
                            && frame.measured_error < existing.measured_error)
                    {
                        best_frame = Some(frame);
                    }
                }
                (Some(existing), false, false) => {
                    if frame.measured_error < existing.measured_error
                        || (frame.measured_error == existing.measured_error
                            && frame.payload.len() < existing.payload.len())
                    {
                        best_frame = Some(frame);
                    }
                }
                _ => {}
            }

            if config.max_error.is_none() && k == 0 {
                // No-bound mode optimizes for bytes, so baseline-only is definitive.
                break;
            }
            if let Some(target) = config.max_error {
                if estimated_error <= target && best_bound_met {
                    break;
                }
            }
        }

        if let Some(frame) = best_frame {
            if let Some(target) = config.max_error {
                if frame.measured_error > target && (frame.measured_error - target).abs() >= 1e-12 {
                    return Err(Error::ErrorBoundNotMet {
                        best: frame.measured_error,
                        target,
                        best_payload: Some(frame.payload),
                    });
                }
            }
            return Ok(frame);
        }

        Err(Error::ResourceLimitExceeded(
            "hybrid-residual could not produce a frame".into(),
        ))
    }

    fn decompress(&self, payload: &[u8], sample_count: u32) -> Result<Vec<f64>> {
        let frame_size: usize = sample_count
            .try_into()
            .map_err(|_| Error::ResourceLimitExceeded("sample_count does not fit usize".into()))?;
        let mut off = 0usize;
        let min = take_f64_le(payload, &mut off)?;
        let max = take_f64_le(payload, &mut off)?;
        let point_step = take_u32_le(payload, &mut off)?;
        let base_count_u32 = take_u32_le(payload, &mut off)?;
        let base_count: usize = base_count_u32
            .try_into()
            .map_err(|_| Error::ResourceLimitExceeded("base_count does not fit usize".into()))?;

        let mut base_samples = Vec::with_capacity(base_count);
        for _ in 0..base_count {
            base_samples.push(take_f64_le(payload, &mut off)?);
        }

        let patch_scale = take_f32_le(payload, &mut off)? as f64;
        if !patch_scale.is_finite() || patch_scale < 0.0 {
            return Err(Error::PayloadCorrupt(
                "hybrid patch scale must be finite and non-negative".into(),
            ));
        }
        let patch_count = take_u16_le(payload, &mut off)? as usize;
        let mut patches = Vec::with_capacity(patch_count);
        let mut index = 0u32;
        for _ in 0..patch_count {
            let delta = take_u32_varint(payload, &mut off)?;
            index = index
                .checked_add(delta)
                .ok_or_else(|| Error::ResourceLimitExceeded("patch index overflow".into()))?;
            let q_bytes = take_bytes(payload, &mut off, 2)?;
            let mut arr = [0u8; 2];
            arr.copy_from_slice(q_bytes);
            let quantized = i16::from_le_bytes(arr);
            patches.push((index, quantized));
        }
        if off != payload.len() {
            return Err(Error::PayloadCorrupt(
                "hybrid payload has trailing bytes".into(),
            ));
        }

        let mut out = reconstruct(min, max, point_step, &base_samples, frame_size)?;
        for (idx, quantized) in patches {
            let i: usize = idx.try_into().map_err(|_| {
                Error::ResourceLimitExceeded("patch index does not fit usize".into())
            })?;
            if i >= out.len() {
                return Err(Error::PayloadCorrupt("patch index out of range".into()));
            }
            let corrected = (out[i] + (quantized as f64) * patch_scale).clamp(min, max);
            out[i] = corrected;
        }
        Ok(out)
    }
}

#[derive(Clone, Copy, Debug)]
struct RankedResidual {
    index: u32,
    residual: f64,
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
            "base point_count does not match implied positions".into(),
        ));
    }

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
        out.push(v.clamp(min, max));
    }
    Ok(out)
}

fn rank_residuals(original: &[f64], reconstructed: &[f64]) -> Result<Vec<RankedResidual>> {
    if original.len() != reconstructed.len() {
        return Err(Error::LengthMismatch {
            a: original.len() as u64,
            b: reconstructed.len() as u64,
        });
    }
    let mut ranked = Vec::with_capacity(original.len());
    for (idx, (&a, &b)) in original.iter().zip(reconstructed).enumerate() {
        let residual = a - b;
        let index: u32 = idx
            .try_into()
            .map_err(|_| Error::ResourceLimitExceeded("index exceeds u32::MAX".into()))?;
        ranked.push(RankedResidual { index, residual });
    }
    ranked.sort_unstable_by(|a, b| {
        b.residual
            .abs()
            .partial_cmp(&a.residual.abs())
            .unwrap_or(Ordering::Equal)
    });
    Ok(ranked)
}

fn normalized_roughness(data: &[f64], min: f64, max: f64) -> f64 {
    if data.len() <= 1 {
        return 0.0;
    }
    let range = max - min;
    if range == 0.0 {
        return 0.0;
    }
    let mut total_variation = 0.0;
    for w in data.windows(2) {
        total_variation += (w[1] - w[0]).abs();
    }
    total_variation / ((data.len() as f64) * range)
}

fn candidate_k_values(
    len: usize,
    ranked_len: usize,
    max_iterations: u32,
    bounded: bool,
) -> Vec<usize> {
    if !bounded {
        return vec![0];
    }
    if max_iterations == 0 {
        return Vec::new();
    }

    let abs_cap = if len <= 2048 { 32 } else { 64 };
    let rel_cap = ((len as f64) * 0.01).ceil() as usize;
    let cap = abs_cap.min(rel_cap.max(1)).min(ranked_len);
    if cap == 0 {
        return vec![0];
    }
    let mut out = vec![0usize];
    let seeds = [4usize, 8, 16, 32, 64];
    for &k in &seeds {
        if k <= cap {
            out.push(k);
        }
    }
    if *out.last().unwrap_or(&0) != cap {
        out.push(cap);
    }
    out.sort_unstable();
    out.dedup();
    out.truncate(max_iterations as usize);
    out
}

fn estimated_nrmse_after_patches(
    min: f64,
    max: f64,
    len: usize,
    total_sse: f64,
    prefix_sq: &[f64],
    k: usize,
) -> f64 {
    let corrected_sse = if k < prefix_sq.len() {
        total_sse - prefix_sq[k]
    } else {
        0.0
    };
    let mse = if len == 0 {
        0.0
    } else {
        corrected_sse.max(0.0) / (len as f64)
    };
    let rmse = mse.sqrt();
    let range = max - min;
    if range == 0.0 {
        if rmse == 0.0 {
            return 0.0;
        }
        return f64::INFINITY;
    }
    rmse / range
}

fn estimate_payload_bytes(base_count: usize, patches: &[RankedResidual]) -> usize {
    let mut bytes = 8 + 8 + 4 + 4 + (base_count * 8) + 4 + 2;
    let mut sorted: Vec<u32> = patches.iter().map(|p| p.index).collect();
    sorted.sort_unstable();
    let mut prev = 0u32;
    for idx in sorted {
        let delta = idx.saturating_sub(prev);
        bytes += varint_len_u32(delta);
        bytes += 2;
        prev = idx;
    }
    bytes
}

fn apply_patches(
    base: &[f64],
    min: f64,
    max: f64,
    ranked_prefix: &[RankedResidual],
) -> Result<Vec<f64>> {
    let mut out = base.to_vec();
    for p in ranked_prefix {
        let i: usize = p
            .index
            .try_into()
            .map_err(|_| Error::ResourceLimitExceeded("patch index does not fit usize".into()))?;
        if i >= out.len() {
            return Err(Error::ResourceLimitExceeded(
                "patch index out of range".into(),
            ));
        }
        out[i] = (out[i] + p.residual).clamp(min, max);
    }
    Ok(out)
}

fn encode_payload(
    min: f64,
    max: f64,
    point_step: u32,
    base_samples: &[f64],
    ranked_prefix: &[RankedResidual],
) -> Result<Vec<u8>> {
    let base_count: u32 = base_samples
        .len()
        .try_into()
        .map_err(|_| Error::ResourceLimitExceeded("base_count exceeds u32::MAX".into()))?;
    let patch_count: u16 = ranked_prefix
        .len()
        .try_into()
        .map_err(|_| Error::ResourceLimitExceeded("patch_count exceeds u16::MAX".into()))?;

    let mut sorted: Vec<RankedResidual> = ranked_prefix.to_vec();
    sorted.sort_unstable_by_key(|p| p.index);
    let max_abs = sorted
        .iter()
        .map(|p| p.residual.abs())
        .fold(0.0_f64, f64::max);
    let scale = if max_abs == 0.0 {
        0.0_f64
    } else {
        max_abs / (i16::MAX as f64)
    };
    let scale_f32 = {
        let s = scale as f32;
        if !s.is_finite() {
            return Err(Error::PrecisionOverflow(scale));
        }
        s
    };

    let mut out = Vec::with_capacity(estimate_payload_bytes(base_samples.len(), &sorted));
    out.extend_from_slice(&min.to_le_bytes());
    out.extend_from_slice(&max.to_le_bytes());
    out.extend_from_slice(&point_step.to_le_bytes());
    out.extend_from_slice(&base_count.to_le_bytes());
    for &v in base_samples {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out.extend_from_slice(&scale_f32.to_le_bytes());
    out.extend_from_slice(&patch_count.to_le_bytes());

    let mut prev = 0u32;
    for p in sorted {
        let delta = p.index.saturating_sub(prev);
        encode_u32_varint(delta, &mut out);
        let q = if scale == 0.0 {
            0i16
        } else {
            let v = (p.residual / scale).round();
            v.clamp(i16::MIN as f64, i16::MAX as f64) as i16
        };
        out.extend_from_slice(&q.to_le_bytes());
        prev = p.index;
    }
    Ok(out)
}

fn encode_u32_varint(mut v: u32, out: &mut Vec<u8>) {
    while v >= 0x80 {
        out.push(((v as u8) & 0x7F) | 0x80);
        v >>= 7;
    }
    out.push(v as u8);
}

fn take_u32_varint(buf: &[u8], offset: &mut usize) -> Result<u32> {
    let mut shift = 0u32;
    let mut value = 0u32;
    loop {
        if shift >= 35 {
            return Err(Error::PayloadCorrupt("varint exceeds u32 width".into()));
        }
        let byte = crate::bytes::take_u8(buf, offset)?;
        value |= ((byte & 0x7F) as u32) << shift;
        if (byte & 0x80) == 0 {
            break;
        }
        shift += 7;
    }
    Ok(value)
}

fn varint_len_u32(mut v: u32) -> usize {
    let mut len = 1usize;
    while v >= 0x80 {
        v >>= 7;
        len += 1;
    }
    len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hybrid_roundtrip_sanity() {
        let data: Vec<f64> = (0..2048)
            .map(|i| {
                let x = i as f64;
                (x * 0.01).sin() + 0.15 * (x * 0.07).cos()
            })
            .collect();
        let cfg = CompressConfig {
            max_error: None,
            ..Default::default()
        };
        let frame = HybridResidualCodec.compress(&data, &cfg).unwrap();
        let out = HybridResidualCodec
            .decompress(&frame.payload, frame.sample_count)
            .unwrap();
        assert_eq!(out.len(), data.len());
        assert!(out.iter().all(|v| v.is_finite()));
    }

    #[test]
    fn hybrid_bounded_zero_iterations_returns_bound_error() {
        let data: Vec<f64> = (0..512).map(|i| (i as f64).sin()).collect();
        let cfg = CompressConfig {
            max_error: Some(0.001),
            max_iterations: 0,
            ..Default::default()
        };
        let err = HybridResidualCodec.compress(&data, &cfg).unwrap_err();
        assert!(matches!(err, Error::ErrorBoundNotMet { .. }));
    }

    #[test]
    fn hybrid_decompress_rejects_truncated_payload() {
        let err = HybridResidualCodec.decompress(&[0u8; 9], 4).unwrap_err();
        assert!(matches!(err, Error::UnexpectedEof { .. }));
    }

    #[test]
    fn varint_roundtrip() {
        let mut buf = Vec::new();
        let values = [0u32, 1, 127, 128, 16_384, u32::MAX];
        for &v in &values {
            encode_u32_varint(v, &mut buf);
        }
        let mut off = 0usize;
        let mut out = Vec::new();
        for _ in 0..values.len() {
            out.push(take_u32_varint(&buf, &mut off).unwrap());
        }
        assert_eq!(values.to_vec(), out);
    }

    #[test]
    fn roughness_is_lower_for_smooth_signal_than_noise() {
        let smooth: Vec<f64> = (0..1024).map(|i| (i as f64 * 0.001).sin()).collect();
        let noise: Vec<f64> = (0..1024)
            .map(|i| if i % 2 == 0 { -1.0 } else { 1.0 })
            .collect();
        let (smin, smax) = finite_min_max(&smooth).unwrap();
        let (nmin, nmax) = finite_min_max(&noise).unwrap();
        let sr = normalized_roughness(&smooth, smin, smax);
        let nr = normalized_roughness(&noise, nmin, nmax);
        assert!(sr < nr);
    }
}
