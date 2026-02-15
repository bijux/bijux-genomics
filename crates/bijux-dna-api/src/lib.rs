//! Public API entrypoints for planning and running bijux pipelines.
//!
//! API policy:
//! - Expose only stable, versioned namespaces (e.g. v1) with curated modules.
//! - Avoid re-exporting internal crates wholesale; keep public surface small.
//! - Any power-user/internal exports must be behind a feature gate.

#![allow(hidden_glob_reexports)]
#![allow(clippy::match_same_arms, clippy::too_many_lines)]

pub(crate) mod explain;
pub(crate) mod fastq_stats_neutral;
pub(crate) mod internal;
pub(crate) mod api_internal {
    use crate::internal as internal_mod;
    pub(crate) use internal_mod::handlers;
}
pub(crate) mod cross_runtime;
pub(crate) mod execution_kernel;
pub(crate) mod input_validation;
pub(crate) mod qa;
pub(crate) mod reference_resolution;
pub(crate) mod request_args;
pub(crate) mod run;
pub(crate) mod tooling;
pub(crate) mod writers;

pub mod v1;
