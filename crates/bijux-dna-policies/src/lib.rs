//! Shared contract/policy checks across crates (enforces ownership and interfaces).

mod assertions;
mod checks;
mod guardrails;
pub mod policy_diagnostics;
pub mod public_api;
mod source_scan;

pub use public_api::{check, GuardrailConfig};
