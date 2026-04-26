//! Owner: bijux-dna-bench
//! SQLite-backed benchmark repository adapters.

pub mod catalog;
pub mod queries;

#[allow(dead_code)]
pub const SQLITE_MODULES: &[&str] = &["catalog", "queries"];
