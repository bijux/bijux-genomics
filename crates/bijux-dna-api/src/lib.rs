//! Public API entrypoints for planning and running bijux pipelines.
//!
//! API policy:
//! - Expose only stable, versioned namespaces (e.g. v1) with curated modules.
//! - Avoid re-exporting internal crates wholesale; keep public surface small.
//! - Any power-user/internal exports must be behind a feature gate.

#![allow(hidden_glob_reexports)]

pub(crate) mod explain;
pub(crate) mod input_validation;
pub(crate) mod internal;
pub(crate) mod runtime;
pub(crate) mod surface;
pub(crate) mod support;
pub(crate) mod writers;
pub(crate) use internal::public_bridge;
pub(crate) use surface::request_contracts as request_args;
pub(crate) use runtime::{cross_runtime, execution_kernel, run};
pub(crate) use support::qa;
pub(crate) use support::reference_resolution;
pub(crate) use support::tooling;

pub mod v1;
