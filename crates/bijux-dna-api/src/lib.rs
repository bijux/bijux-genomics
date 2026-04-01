//! Public API entrypoints for planning and running bijux pipelines.
//!
//! API policy:
//! - Expose only stable, versioned namespaces (e.g. v1) with curated modules.
//! - Avoid re-exporting internal crates wholesale; keep public surface small.
//! - Any power-user/internal exports must be behind a feature gate.

#![allow(hidden_glob_reexports)]

pub(crate) mod internal;
pub(crate) mod runtime;
pub(crate) mod support;
pub(crate) mod surface;
pub(crate) use internal::public_bridge;
pub(crate) use runtime::persistence as writers;
pub(crate) use runtime::validation as input_validation;
pub(crate) use runtime::{cross_runtime, execution_kernel, run};
pub(crate) use support::qa;
pub(crate) use support::reference_resolution;
pub(crate) use support::tool_selection;
pub(crate) use support::tooling;
pub(crate) use surface::explain;
pub(crate) use surface::request_contracts as request_args;

pub mod v1;
