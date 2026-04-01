//! Execution engine for Bijux.
//!
//! Owns: execution services, validation gates, and observability hooks.
//! Must NOT depend on: bijux-dna-domain-* crates or domain semantics.
//! Policy: engine must not spawn processes (see clippy disallowed methods/types).

#![deny(clippy::disallowed_methods, clippy::disallowed_types)]
#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::implicit_hasher,
    clippy::must_use_candidate,
    clippy::new_without_default
)]

mod errors;
mod executor;
mod engine_config;
mod engine_driver;
mod control;
mod observability;
pub mod public_api;

pub use control::CancellationToken;
pub use engine_driver::Engine;
pub use engine_config::EngineConfig;
pub use observability::{EngineEvent, EngineHooks};

pub use public_api::*;
