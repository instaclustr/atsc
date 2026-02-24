//! FFT codec.
//!
//! This codec stores the top-N frequency components from a real-valued FFT.
//! Frequency positions are encoded as a bitpacked index to avoid storing a
//! per-frequency `pos` field.

use crate::codec::{Codec, CompressConfig, CompressedFrame};
use crate::error::{Error, Result};
use crate::metrics::nrmse;
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// FFT codec storing f32 frequencies (codec id 2).
#[derive(Clone, Copy, Debug, Default)]
pub struct FftF32Codec;

impl FftF32Codec {
    /// Wire format codec id.
    pub const ID: u8 = 2;
}

impl Codec for FftF32Codec {
    fn id(&self) -> u8 {
        Self::ID
    }

    fn name(&self) -> &'static str {
        "fft-f32"
    }

    fn compress(&self, data: &[f64], config: &CompressConfig) -> Result<CompressedFrame> {
        let (min, max) = finite_min_max(data)?;
        let sample_count: u32 = data
            .len()
            .try_into()
            .map_err(|_| Error::SampleCountOverflow {
                count: data.len() as u64,
            })?;

        let (fft_len, prefix_len) = frozen_fft_len_and_prefix(data.len());
        let padded = pad_edges(data, fft_len, prefix_len)?;

        let ctx = CompressCtx {
            padded: &padded,
            fft_len,
            prefix_len,
            original: data,
            min,
            max,
        };

        let base_freq = (data.len() / 100).max(3);

        let (payload, measured_error) = match config.max_error {
            None => compress_once_f32(&ctx, base_freq)?,
            Some(target) => compress_bounded_f32(&ctx, base_freq, target, config.max_iterations)?,
        };

        Ok(CompressedFrame {
            codec_id: Self::ID,
            sample_count,
            payload,
            measured_error,
        })
    }

    fn decompress(&self, payload: &[u8], sample_count: u32) -> Result<Vec<f64>> {
        let (fft_len, prefix_len) = frozen_fft_len_and_prefix(sample_count as usize);
        let mut off = 0usize;

        let min_value = take_f32_le(payload, &mut off)? as f64;
        let max_value = take_f32_le(payload, &mut off)? as f64;
        let freq_count = take_u16_le(payload, &mut off)? as usize;

        let bits_per_pos = bits_per_pos(fft_len);
        let pos_bytes = pos_index_len(freq_count, bits_per_pos);
        let pos_buf = take_bytes(payload, &mut off, pos_bytes)?;
        let positions = unpack_positions(pos_buf, freq_count, bits_per_pos)?;

        let mut freqs = Vec::with_capacity(freq_count);
        for _ in 0..freq_count {
            let re = take_f32_le(payload, &mut off)?;
            let im = take_f32_le(payload, &mut off)?;
            freqs.push(Complex { re, im });
        }
        if off != payload.len() {
            // We treat extra bytes as an error to keep decode strict.
            return Err(Error::ResourceLimitExceeded(
                "fft payload has trailing bytes".into(),
            ));
        }

        let mut spectrum = vec![Complex::<f32>::default(); fft_len];
        for (pos, c) in positions.into_iter().zip(freqs) {
            let p = usize::try_from(pos).map_err(|_| {
                Error::ResourceLimitExceeded("frequency position does not fit usize".into())
            })?;
            if p >= fft_len {
                return Err(Error::ResourceLimitExceeded(
                    "frequency position out of range".into(),
                ));
            }
            spectrum[p] = c;

            // Mirror for real input (skip DC and Nyquist).
            if p != 0 && p * 2 != fft_len {
                spectrum[fft_len - p] = Complex {
                    re: c.re,
                    im: -c.im,
                };
            }
        }

        let mut planner = FftPlanner::<f32>::new();
        let ifft = planner.plan_fft_inverse(fft_len);
        ifft.process(&mut spectrum);

        let norm = fft_len as f64;
        let out_len: usize = sample_count
            .try_into()
            .map_err(|_| Error::ResourceLimitExceeded("sample_count does not fit usize".into()))?;
        let start = prefix_len;
        let end = start + out_len;
        if end > spectrum.len() {
            return Err(Error::ResourceLimitExceeded(
                "decoded frame exceeds spectrum length".into(),
            ));
        }

        let mut out = Vec::with_capacity(out_len);
        for c in &spectrum[start..end] {
            let v = (c.re as f64) / norm;
            let clamped = v.clamp(min_value, max_value);
            out.push(clamped);
        }
        Ok(out)
    }
}

#[derive(Clone, Copy, Debug)]
struct HeapItem {
    pos: u32,
    c: Complex<f32>,
}

impl PartialEq for HeapItem {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos && self.c.re == other.c.re && self.c.im == other.c.im
    }
}

impl Eq for HeapItem {}

impl PartialOrd for HeapItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HeapItem {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering (max-heap by magnitude).
        let a = self.c.norm_sqr();
        let b = other.c.norm_sqr();
        a.partial_cmp(&b).unwrap_or(Ordering::Equal)
    }
}

#[derive(Clone, Copy, Debug)]
struct CompressCtx<'a> {
    padded: &'a [f64],
    fft_len: usize,
    prefix_len: usize,
    original: &'a [f64],
    min: f64,
    max: f64,
}

fn compress_bounded_f32(
    ctx: &CompressCtx<'_>,
    base_freq: usize,
    target: f64,
    max_iterations: u32,
) -> Result<(Vec<u8>, f64)> {
    if max_iterations == 0 {
        return Err(Error::ErrorBoundNotMet {
            best: f64::INFINITY,
            target,
        });
    }

    let mut best_payload = None;
    let mut best_error = f64::INFINITY;
    let mut jump = 0usize;

    let eps = 1e-12;
    let max_iter = max_iterations as usize;
    for i in 0..max_iter {
        let keep = base_freq + jump;
        let (payload, err) = compress_once_f32(ctx, keep)?;
        if err < best_error {
            best_error = err;
            best_payload = Some(payload);
        }
        if best_error <= target || (best_error - target).abs() < eps {
            return Ok((best_payload.expect("best_payload set"), best_error));
        }

        // Convergence schedule (ported from v1 with iteration budget controlled by config).
        if i < 17 {
            jump = jump.saturating_add((base_freq / 2).max(1));
        } else {
            jump = jump.saturating_add((base_freq / 10).max(1));
        }
    }

    Err(Error::ErrorBoundNotMet {
        best: best_error,
        target,
    })
}

fn compress_once_f32(ctx: &CompressCtx<'_>, keep: usize) -> Result<(Vec<u8>, f64)> {
    let (positions, freqs) = top_frequencies_f32(ctx.padded, ctx.fft_len, keep)?;
    let reconstructed = reconstruct_f32(
        ctx.fft_len,
        ctx.prefix_len,
        ctx.original.len(),
        ctx.min,
        ctx.max,
        &positions,
        &freqs,
    )?;
    let err = nrmse(ctx.original, &reconstructed)?;
    let payload = encode_payload_f32(ctx.min, ctx.max, &positions, &freqs, ctx.fft_len)?;
    Ok((payload, err))
}

fn top_frequencies_f32(
    padded: &[f64],
    fft_len: usize,
    keep: usize,
) -> Result<(Vec<u32>, Vec<Complex<f32>>)> {
    let mut buffer = Vec::with_capacity(fft_len);
    for &v in padded {
        buffer.push(Complex {
            re: f64_to_f32(v)?,
            im: 0.0,
        });
    }

    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(fft_len);
    fft.process(&mut buffer);

    // Real input: only half + 1 unique frequencies.
    let unique = (fft_len / 2) + 1;
    buffer.truncate(unique);

    let mut heap = build_heap(&buffer)?;

    let mut positions = Vec::with_capacity(keep.min(unique));
    let mut freqs = Vec::with_capacity(keep.min(unique));
    for _ in 0..keep {
        let Some(item) = heap.pop() else { break };
        if item.c.re == 0.0 && item.c.im == 0.0 {
            break;
        }
        positions.push(item.pos);
        freqs.push(item.c);
    }

    Ok((positions, freqs))
}

fn build_heap(buffer: &[Complex<f32>]) -> Result<BinaryHeap<HeapItem>> {
    let mut heap = BinaryHeap::with_capacity(buffer.len());
    for (pos, &c) in buffer.iter().enumerate() {
        if !(c.re.is_finite() && c.im.is_finite()) {
            return Err(Error::ResourceLimitExceeded(
                "non-finite value in FFT spectrum".into(),
            ));
        }
        heap.push(HeapItem { pos: pos as u32, c });
    }
    Ok(heap)
}

fn reconstruct_f32(
    fft_len: usize,
    prefix_len: usize,
    out_len: usize,
    min: f64,
    max: f64,
    positions: &[u32],
    freqs: &[Complex<f32>],
) -> Result<Vec<f64>> {
    if positions.len() != freqs.len() {
        return Err(Error::ResourceLimitExceeded(
            "positions and freqs length mismatch".into(),
        ));
    }

    let mut spectrum = vec![Complex::<f32>::default(); fft_len];
    for (&pos, &c) in positions.iter().zip(freqs) {
        let p = pos as usize;
        if p >= fft_len {
            return Err(Error::ResourceLimitExceeded(
                "frequency position out of range".into(),
            ));
        }
        spectrum[p] = c;
        if p != 0 && p * 2 != fft_len {
            spectrum[fft_len - p] = Complex {
                re: c.re,
                im: -c.im,
            };
        }
    }

    let mut planner = FftPlanner::<f32>::new();
    let ifft = planner.plan_fft_inverse(fft_len);
    ifft.process(&mut spectrum);

    let norm = fft_len as f64;
    let start = prefix_len;
    let end = start + out_len;
    if end > spectrum.len() {
        return Err(Error::ResourceLimitExceeded(
            "reconstructed output exceeds spectrum length".into(),
        ));
    }

    Ok(spectrum[start..end]
        .iter()
        .map(|c| ((c.re as f64) / norm).clamp(min, max))
        .collect())
}

fn encode_payload_f32(
    min: f64,
    max: f64,
    positions: &[u32],
    freqs: &[Complex<f32>],
    fft_len: usize,
) -> Result<Vec<u8>> {
    let min_f32 = f64_to_f32(min)?;
    let max_f32 = f64_to_f32(max)?;
    let freq_count: u16 = positions
        .len()
        .try_into()
        .map_err(|_| Error::ResourceLimitExceeded("freq_count exceeds u16::MAX".into()))?;

    let bits = bits_per_pos(fft_len);
    let pos_buf = pack_positions(positions, bits)?;

    let mut out = Vec::with_capacity(4 + 4 + 2 + pos_buf.len() + freqs.len() * 8);
    out.extend_from_slice(&min_f32.to_le_bytes());
    out.extend_from_slice(&max_f32.to_le_bytes());
    out.extend_from_slice(&freq_count.to_le_bytes());
    out.extend_from_slice(&pos_buf);
    for &c in freqs {
        out.extend_from_slice(&c.re.to_le_bytes());
        out.extend_from_slice(&c.im.to_le_bytes());
    }
    Ok(out)
}

fn frozen_fft_len_and_prefix(sample_count: usize) -> (usize, usize) {
    if sample_count < 128 {
        return (sample_count.max(1), 0);
    }
    let fft_len = next_smooth(sample_count.max(1));
    let added = fft_len - sample_count;
    let prefix = added / 2;
    (fft_len, prefix)
}

fn pad_edges(data: &[f64], fft_len: usize, prefix_len: usize) -> Result<Vec<f64>> {
    let (first, _) = data.split_first().ok_or(Error::EmptyData)?;
    let last = data.last().copied().ok_or(Error::EmptyData)?;
    if !first.is_finite() || !last.is_finite() {
        return Err(Error::InvalidInput);
    }

    let mut out = Vec::with_capacity(fft_len);
    out.resize(prefix_len, *first);
    out.extend_from_slice(data);
    let suffix_len = fft_len - prefix_len - data.len();
    out.resize(fft_len, last);
    debug_assert_eq!(out.len(), fft_len);
    debug_assert_eq!(suffix_len, fft_len - prefix_len - data.len());
    Ok(out)
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

fn f64_to_f32(x: f64) -> Result<f32> {
    let y = x as f32;
    if !(x.is_finite() && y.is_finite()) {
        return Err(Error::PrecisionOverflow(x));
    }
    Ok(y)
}

fn bits_per_pos(fft_len: usize) -> u32 {
    if fft_len <= 1 {
        return 0;
    }
    let v = (fft_len as u64) - 1;
    64 - v.leading_zeros()
}

fn pos_index_len(freq_count: usize, bits_per_pos: u32) -> usize {
    if bits_per_pos == 0 || freq_count == 0 {
        return 0;
    }
    let total_bits = (freq_count as u64) * (bits_per_pos as u64);
    ((total_bits + 7) / 8) as usize
}

fn pack_positions(positions: &[u32], bits_per_pos: u32) -> Result<Vec<u8>> {
    if bits_per_pos == 0 {
        return Ok(Vec::new());
    }

    let out_len = pos_index_len(positions.len(), bits_per_pos);
    let mut out = vec![0u8; out_len];

    let mut bit_cursor: u64 = 0;
    for &pos in positions {
        let max_pos = if bits_per_pos == 32 {
            u32::MAX
        } else {
            (1u32 << bits_per_pos) - 1
        };
        if pos > max_pos {
            return Err(Error::ResourceLimitExceeded(
                "position does not fit bits_per_pos".into(),
            ));
        }
        for i in 0..bits_per_pos {
            let bit = (pos >> i) & 1;
            let byte_idx = (bit_cursor / 8) as usize;
            let bit_idx = (bit_cursor % 8) as u8;
            out[byte_idx] |= (bit as u8) << bit_idx;
            bit_cursor += 1;
        }
    }

    Ok(out)
}

fn unpack_positions(buf: &[u8], count: usize, bits_per_pos: u32) -> Result<Vec<u32>> {
    if bits_per_pos == 0 {
        return Ok(vec![0u32; count]);
    }

    let expected = pos_index_len(count, bits_per_pos);
    if buf.len() != expected {
        return Err(Error::UnexpectedEof {
            offset: 0,
            expected: expected as u64,
        });
    }

    let mut out = vec![0u32; count];
    let mut bit_cursor: u64 = 0;
    for item in out.iter_mut() {
        let mut pos = 0u32;
        for i in 0..bits_per_pos {
            let byte_idx = (bit_cursor / 8) as usize;
            let bit_idx = (bit_cursor % 8) as u8;
            let bit = (buf[byte_idx] >> bit_idx) & 1;
            pos |= (bit as u32) << i;
            bit_cursor += 1;
        }
        *item = pos;
    }
    Ok(out)
}

fn next_smooth(n: usize) -> usize {
    let n = n as u64;
    let mut best = u64::MAX;
    let mut pow3 = 1u64;
    loop {
        if pow3 >= best {
            break;
        }
        let mut pow2 = 1u64;
        while let Some(v) = pow2.checked_mul(pow3) {
            if v >= n {
                if v < best {
                    best = v;
                }
                break;
            }
            pow2 = match pow2.checked_mul(2) {
                Some(p) => p,
                None => break,
            };
        }
        pow3 = match pow3.checked_mul(3) {
            Some(p) => p,
            None => break,
        };
    }
    best as usize
}

fn take_bytes<'a>(buf: &'a [u8], offset: &mut usize, len: usize) -> Result<&'a [u8]> {
    if *offset > buf.len() || buf.len() - *offset < len {
        return Err(Error::UnexpectedEof {
            offset: *offset as u64,
            expected: len as u64,
        });
    }
    let out = &buf[*offset..*offset + len];
    *offset += len;
    Ok(out)
}

fn take_u16_le(buf: &[u8], offset: &mut usize) -> Result<u16> {
    let bytes = take_bytes(buf, offset, 2)?;
    let mut arr = [0u8; 2];
    arr.copy_from_slice(bytes);
    Ok(u16::from_le_bytes(arr))
}

fn take_f32_le(buf: &[u8], offset: &mut usize) -> Result<f32> {
    let bytes = take_bytes(buf, offset, 4)?;
    let mut arr = [0u8; 4];
    arr.copy_from_slice(bytes);
    Ok(f32::from_le_bytes(arr))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positions_bitpack_roundtrip() {
        let fft_len = 2187; // 3^7 (smooth)
        let bits = bits_per_pos(fft_len);
        let positions = vec![0u32, 1, 2, 7, 1024, 2186];
        let packed = pack_positions(&positions, bits).unwrap();
        let unpacked = unpack_positions(&packed, positions.len(), bits).unwrap();
        assert_eq!(unpacked, positions);
    }

    #[test]
    fn fft_roundtrip_no_error_bound() {
        let data: Vec<f64> = (0..1024)
            .map(|i| ((i as f64) * 0.01).sin() + 0.1 * ((i as f64) * 0.03).cos())
            .collect();
        let cfg = CompressConfig {
            max_error: None,
            ..Default::default()
        };
        let frame = FftF32Codec.compress(&data, &cfg).unwrap();
        let out = FftF32Codec
            .decompress(&frame.payload, frame.sample_count)
            .unwrap();
        assert_eq!(out.len(), data.len());
        // Not asserting strict value equality (lossy) — just sanity that it produced finite output.
        assert!(out.iter().all(|v| v.is_finite()));
    }

    #[test]
    fn heap_builder_rejects_non_finite_complex() {
        let buf = [Complex::<f32> {
            re: f32::NAN,
            im: 0.0,
        }];
        let err = build_heap(&buf).unwrap_err();
        assert!(matches!(err, Error::ResourceLimitExceeded(_)));
    }
}
