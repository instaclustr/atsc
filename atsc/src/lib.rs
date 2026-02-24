//! ATSC v2 time-series compression library.
//!
//! This crate is a library-first redesign of ATSC with a manual wire format and
//! hardened decode paths. See `PLAN.md` for the full architecture and format
//! specification.

pub mod codec;
pub mod error;
pub mod format;
pub mod metrics;
pub mod optimizer;
pub mod vsri;

pub use error::{Error, Result};
