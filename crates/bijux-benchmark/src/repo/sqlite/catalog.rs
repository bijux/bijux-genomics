//! Owner: bijux-benchmark
//! SQLite query catalog for benchmark repositories.
//! Owns the list of query modules for traceability.
//! Must not execute queries directly.

#[allow(dead_code)]
pub const QUERY_MODULES: &[&str] = &["run_index"];
