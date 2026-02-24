//! Stream footer.

/// Fixed footer size in bytes.
pub const FOOTER_LEN: usize = 8;

/// Footer magic (reverse of `ATSC`).
pub const FOOTER_MAGIC: [u8; 4] = *b"CSTA";

/// Stream footer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Footer {
    /// Number of frames in the stream.
    pub frame_count: u32,
}
