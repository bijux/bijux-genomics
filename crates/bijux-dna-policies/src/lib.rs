//! Shared contract/policy checks across crates (enforces ownership and interfaces).

mod content_rules;
mod file_scan;
mod guardrails;
mod macros;
pub mod policy_diagnostics;
mod tree_rules;

pub use guardrails::{check, GuardrailConfig};
