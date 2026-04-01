//! Owner: bijux-dna-bench
//! Public API for benchmark loading, summarization, comparison, and gating.
//! Contract: inputs are typed, outputs are deterministic, and raw JSON is confined to repo/artifacts.

mod artifacts;
mod public_api;
mod repo;
mod workflow;

pub use public_api::*;
