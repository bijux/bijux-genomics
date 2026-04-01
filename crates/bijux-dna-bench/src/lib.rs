//! Owner: bijux-dna-bench
//! Public API for benchmark loading, summarization, comparison, and gating.
//! Contract: inputs are typed, outputs are deterministic, and raw JSON is confined to repo/artifacts.

mod artifacts;
mod repo;
mod summary;
pub mod public_api;

pub use public_api::*;
