//! Versioned public API surface (v1).
//!
//! Curated modules only:
//! - plan: pipeline selection + plan building.
//! - run: execution entrypoints + run indexing helpers.
//! - report: report rendering + export helpers.
//! - bench: benchmarking + comparison helpers.

pub mod bench;
pub mod bam;
pub mod fastq;
pub mod plan;
pub mod report;
pub mod run;
