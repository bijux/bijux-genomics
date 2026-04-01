//! Path helpers with stable ordering and deterministic outputs.
//!
//! Invariants:
//! - Only path construction helpers (no IO).
//! - Stable, deterministic ordering of returned paths.

mod bench;
mod config;
mod run_layout;

pub use bench::{bench_base_dir, bench_data_dir, bench_suites_dir, bench_tools_dir};
pub use config::{configs_dir, configs_file};
pub use run_layout::{normalize_run_base_dir, pipeline_run_dir, run_layout_paths, run_stage_dir};
