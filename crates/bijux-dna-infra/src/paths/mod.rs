//! Path helpers with stable ordering and deterministic outputs.
//!
//! Invariants:
//! - Only path construction helpers (no IO).
//! - Stable, deterministic ordering of returned paths.

mod bench;
mod config;

pub use bench::{bench_base_dir, bench_data_dir, bench_suites_dir, bench_tools_dir};
pub use config::{configs_dir, configs_file};
