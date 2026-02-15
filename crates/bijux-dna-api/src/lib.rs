//! Public API entrypoints for planning and running bijux pipelines.
//!
//! API policy:
//! - Expose only stable, versioned namespaces (e.g. v1) with curated modules.
//! - Avoid re-exporting internal crates wholesale; keep public surface small.
//! - Any power-user/internal exports must be behind a feature gate.

#![allow(hidden_glob_reexports)]

pub(crate) mod explain;
#[path = "internal/fastq/fastq_stats_neutral.rs"]
pub(crate) mod fastq_stats_neutral;
pub(crate) mod internal;
pub(crate) mod api_internal {
    use crate::internal as internal_mod;
    pub(crate) use internal_mod::handlers;
}
#[path = "runtime/cross_runtime.rs"]
pub(crate) mod cross_runtime;
#[path = "runtime/execution_kernel.rs"]
pub(crate) mod execution_kernel;
pub(crate) mod input_validation;
pub(crate) mod request_args;
#[path = "runtime/run.rs"]
pub(crate) mod run;
pub(crate) mod support;
pub(crate) mod writers;
pub(crate) use support::qa;
pub(crate) use support::reference_resolution;
pub(crate) use support::tooling;

pub mod v1;
