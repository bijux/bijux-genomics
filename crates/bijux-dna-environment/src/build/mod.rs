//! Environment catalog build helpers.
//!
//! Responsibilities: derive tool metadata from dockerfiles and curated defaults.
//! Invariants: no resolution side effects; outputs must be deterministic for the same inputs.

mod builder;
mod defaults;
mod models;
mod stable_surface;
mod version_parser;

pub use stable_surface::*;
