//! Internal wiring and helpers (non-public).

pub mod fastq;
pub mod handlers;

#[cfg(feature = "api_internal")]
pub mod api_internal;
