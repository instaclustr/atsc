//! Stream frame structure.

/// Fixed frame header size in bytes.
pub const FRAME_HEADER_LEN: usize = 9;

/// Frame header (without payload).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameHeader {
    /// Codec ID.
    pub codec_id: u8,
    /// Number of samples in this frame.
    pub sample_count: u32,
    /// Payload length in bytes.
    pub payload_len: u32,
}
