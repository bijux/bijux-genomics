//! Shared contract/policy checks across crates (enforces ownership and interfaces).

mod checks;
mod guardrails;
mod macros;
pub mod policy_diagnostics;
mod source_scan;

pub use guardrails::{check, GuardrailConfig};
