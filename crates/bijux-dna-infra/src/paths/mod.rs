//! Path helpers with stable ordering and deterministic outputs.
//!
//! Invariants:
//! - Only path construction helpers (no IO).
//! - Stable, deterministic ordering of returned paths.

mod bench;
mod config;
mod config_aliases;
mod run_layout;
mod stable_surface;

pub use stable_surface::*;
