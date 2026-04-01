//! Shared contract/policy checks across crates (enforces ownership and interfaces).

mod checks;
mod guardrails;
mod macros;
pub mod policy_diagnostics;
pub mod public_api;
mod source_scan;

pub use public_api::{check, GuardrailConfig};
