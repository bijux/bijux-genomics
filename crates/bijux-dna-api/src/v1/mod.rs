//! Versioned public API surface (v1).
//!
//! Curated modules only:
//! - plan: pipeline selection + plan building.
//! - run: execution entrypoints + run indexing helpers.
//! - report: report rendering + export helpers.
//! - bench: benchmarking + comparison helpers.

pub mod api;
pub(crate) mod bam;
pub(crate) mod bench;
pub(crate) mod env;
pub(crate) mod fastq;
pub(crate) mod plan;
pub(crate) mod report;
pub(crate) mod run;

// Keep this module non-empty to satisfy guardrails and clarify intent.
pub const API_V1_BANNER: &str = "bijux-dna-api v1";
