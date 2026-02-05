//! Public API entrypoints for planning and running bijux pipelines.
//!
//! API policy:
//! - Expose only stable, versioned namespaces (e.g. v1) with curated modules.
//! - Avoid re-exporting internal crates wholesale; keep public surface small.
//! - Any power-user/internal exports must be behind a feature gate.

#![allow(hidden_glob_reexports)]

pub(crate) mod args;
pub(crate) mod bam_router;
pub(crate) mod cross_router;
pub(crate) mod fastq_router;
pub(crate) mod fastq_stats_neutral;
pub(crate) mod run;
pub(crate) mod tooling;

pub mod prelude;
pub mod v1;
pub use v1::*;

#[cfg(feature = "api_internal")]
pub mod api_internal;
