//! Execution engine for Bijux.
//!
//! Owns: planning, execution services, validation gates, and observability hooks.
//! Must NOT depend on: bijux-domain-* crates or domain semantics.

#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::implicit_hasher,
    clippy::must_use_candidate,
    clippy::new_without_default
)]

pub mod api;
pub mod core;
pub mod internal;
pub mod services;

pub use bijux_env_runtime::api::ResolvedImage;
