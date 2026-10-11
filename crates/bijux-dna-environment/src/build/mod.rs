//! Environment catalog build helpers.
//!
//! Responsibilities: derive tool metadata and execute governed Apptainer image builds.
//! Invariants: build attempts are append-only, bounded, and deterministic in classification.

mod apptainer;
mod builder;
mod defaults;
mod entrypoints;
mod models;
mod stable_surface;
mod version_parser;

pub use stable_surface::*;
