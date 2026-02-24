//! Codec trait and common codec types.

use crate::error::Result;

pub mod constant;
pub mod fft;
pub mod noop;
pub mod polynomial;

/// A compression codec implementation.
pub trait Codec: Send + Sync {
    /// Unique identifier for this codec (wire format codec id).
    fn id(&self) -> u8;

    /// Human-readable codec name.
    fn name(&self) -> &'static str;

    /// Compress a slice of values into a single frame payload.
    ///
    /// # Errors
    /// - Returns `Err` if compression fails.
    /// - Returns `Err(Error::ErrorBoundNotMet { .. })` when `max_error` cannot be met
    ///   within `max_iterations`. The optimizer is expected to catch this and select
    ///   the best available result across codecs.
    fn compress(&self, data: &[f64], config: &CompressConfig) -> Result<CompressedFrame>;

    /// Decompress a frame payload back into values.
    ///
    /// # Errors
    /// Returns `Err` if the payload is invalid or truncated.
    fn decompress(&self, payload: &[u8], sample_count: u32) -> Result<Vec<f64>>;
}

/// Configuration for compression.
#[derive(Clone, Debug)]
pub struct CompressConfig {
    /// Maximum allowed error (codec should try to meet or beat it).
    pub max_error: Option<f64>,
    /// Budget for bounded/iterative codecs.
    pub max_iterations: u32,
    /// If true, reject NaN/Inf inputs instead of filtering them.
    pub reject_nan_inf: bool,
}

impl Default for CompressConfig {
    fn default() -> Self {
        Self {
            max_error: Some(0.05),
            max_iterations: 22,
            reject_nan_inf: false,
        }
    }
}

/// Result of compressing one frame.
#[derive(Clone, Debug)]
pub struct CompressedFrame {
    /// Codec ID for this payload.
    pub codec_id: u8,
    /// Number of samples in this frame.
    pub sample_count: u32,
    /// Codec-specific payload bytes.
    pub payload: Vec<u8>,
    /// Error measured against the original input for this frame.
    pub measured_error: f64,
}
