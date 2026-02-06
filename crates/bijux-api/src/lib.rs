//! Public API entrypoints for planning and running bijux pipelines.
//!
//! API policy:
//! - Expose only stable, versioned namespaces (e.g. v1) with curated modules.
//! - Avoid re-exporting internal crates wholesale; keep public surface small.
//! - Any power-user/internal exports must be behind a feature gate.

#![allow(hidden_glob_reexports)]

pub(crate) mod args;
pub(crate) mod explain;
pub(crate) mod fastq_stats_neutral;
pub(crate) mod internal;
pub(crate) mod run;
pub(crate) mod tooling;

pub mod v1;
