//! Owner: bijux-domain-fastq
//! Invariants layering: core thresholds, metric invariants, and spec wiring.
//! Core defines thresholds; metrics enforce per-stage invariants; specs bind them to stages.

mod core;
mod metrics;
mod specs;
mod verdicts;

pub use core::{thresholds_from_env, InvariantEvaluation, InvariantThresholds};
pub use metrics::evaluate_invariants;
pub use specs::fastq_invariant_specs;
