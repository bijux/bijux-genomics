//! Owner: bijux-bench
//! Repository layer for benchmark inputs.
//! Owns fetching run manifests/metrics without filesystem crawling.
//! Must not perform compare/gate logic.
//! Invariants: repository paths are explicit and deterministic.

mod run_repository;

pub use run_repository::*;
