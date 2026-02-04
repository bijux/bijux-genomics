mod core;
mod metrics;
mod specs;
mod verdicts;

pub use core::{thresholds_from_env, InvariantEvaluation, InvariantThresholds};
pub use metrics::evaluate_invariants;
pub use specs::fastq_invariant_specs;
