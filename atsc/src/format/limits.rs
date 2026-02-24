//! Hard resource limits for decoding untrusted input.

/// Maximum number of frames a stream may contain.
pub const MAX_FRAMES: u32 = 1_000_000;
/// Maximum payload size per frame, in bytes.
pub const MAX_PAYLOAD_BYTES: u32 = 256 * 1024 * 1024; // 256 MiB
/// Maximum total number of samples across all frames.
pub const MAX_TOTAL_SAMPLES: u64 = 1_000_000_000;
/// Maximum number of VSRI segments.
pub const MAX_VSRI_SEGMENTS: u32 = 10_000_000;
