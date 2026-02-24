use std::io;

/// Error type for ATSC v2 operations.
#[must_use]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Input data is empty.
    #[error("data is empty")]
    EmptyData,

    /// Input was rejected due to invalid values (strict NaN/Inf mode).
    #[error("input contains NaN or Inf values (strict mode enabled)")]
    InvalidInput,

    /// Decode failed due to truncated input.
    #[error("decode: unexpected end of input at offset {offset}, expected {expected} bytes")]
    UnexpectedEof { offset: u64, expected: u64 },

    /// Decode failed because the stream header magic was invalid.
    #[error("decode: invalid magic bytes")]
    InvalidMagic,

    /// Decode failed because the format version is not supported.
    #[error("decode: unsupported format version {0}")]
    UnsupportedVersion(u8),

    /// Decode failed because the codec id is unknown.
    #[error("decode: unknown codec id {0}")]
    UnknownCodec(u8),

    /// Decode rejected the stream because it exceeds hard resource limits.
    #[error("decode: resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),

    /// Encode rejected the input because the sample count doesn't fit in `u32`.
    #[error("encode: sample count {count} exceeds u32::MAX")]
    SampleCountOverflow { count: u64 },

    /// Codec was unable to satisfy the error bound with the provided budget.
    #[error("error bound not satisfiable: best={best:.6}, target={target:.6}")]
    ErrorBoundNotMet { best: f64, target: f64 },

    /// Precision loss converting `f64` to `f32` would overflow or produce non-finite.
    #[error("precision overflow converting f64 to f32: {0}")]
    PrecisionOverflow(f64),

    /// VSRI rejected a timestamp that is not strictly increasing.
    #[error("vsri: timestamp {ts} is not monotonically increasing (max was {max})")]
    VsriOutOfOrder { ts: i64, max: i64 },

    /// VSRI invariant was violated during encode/decode.
    #[error("vsri invariant violation: {0}")]
    VsriInvariantViolation(String),

    /// Metric computation failed due to mismatched slice lengths.
    #[error("metrics: original and reconstructed lengths differ ({a} vs {b})")]
    LengthMismatch { a: u64, b: u64 },

    /// I/O error.
    #[error(transparent)]
    Io(#[from] io::Error),
}

/// Convenience alias for results returned by this crate.
pub type Result<T> = std::result::Result<T, Error>;
