//! SQLite-backed repository implementations.

pub mod catalog;
pub mod queries;

#[allow(dead_code)]
pub const SQLITE_MODULES: &[&str] = &["catalog", "queries"];
