//! Owner: bijux-benchmark
//! Legacy benchmarking APIs (compat layer).
//! Owns compatibility adapters for legacy benchmark inputs.
//! Must not introduce new contracts; use for deprecation-only support.

pub mod fastq;

#[allow(dead_code)]
pub const LEGACY_MODULES: &[&str] = &["fastq"];
