//! Stream header.

/// Fixed header size in bytes.
pub const HEADER_LEN: usize = 6;

/// Stream magic for ATSC v2.
pub const MAGIC: [u8; 4] = *b"ATSC";

/// Stream header.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Header {
    /// Format version (starts at 1).
    pub version: u8,
    /// Header flags (bit 0: inline VSRI as first frame).
    pub flags: u8,
}
