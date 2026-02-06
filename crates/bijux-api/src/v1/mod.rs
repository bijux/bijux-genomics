//! Versioned public API surface (v1).
//!
//! Curated modules only:
//! - plan: pipeline selection + plan building.
//! - run: execution entrypoints + run indexing helpers.
//! - report: report rendering + export helpers.
//! - bench: benchmarking + comparison helpers.

pub mod api;
pub mod bam;
pub mod bench;
pub mod env;
pub mod fastq;
pub mod plan;
pub mod report;
pub mod run;

// Keep this module non-empty to satisfy guardrails and clarify intent.
pub const API_V1_BANNER: &str = "bijux-api v1";
