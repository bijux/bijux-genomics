//! Path helpers with stable ordering and deterministic outputs.
//!
//! Invariants:
//! - Only path construction helpers (no IO).
//! - Stable, deterministic ordering of returned paths.

mod bench;

pub use bench::{bench_base_dir, bench_tools_dir};
