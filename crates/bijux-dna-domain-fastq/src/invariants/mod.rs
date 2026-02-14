//! Owner: bijux-dna-domain-fastq
//! Invariants layering: core thresholds, metric invariants, and spec wiring.
//! Core defines thresholds; metrics enforce per-stage invariants; specs bind them to stages.

mod evaluation;
mod edna;
mod metrics;
mod specs;
mod verdicts;

pub use evaluation::{thresholds_from_env, InvariantEvaluation, InvariantThresholds};
pub use edna::validate_edna_table;
pub use metrics::evaluate_invariants;
pub use specs::fastq_invariant_specs;
