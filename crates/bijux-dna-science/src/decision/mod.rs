//! Deterministic scientific acceptance decisions over measured evidence.

mod benchmark_gate;
mod qualification;

pub use benchmark_gate::{evaluate_benchmark_gate_policy, validate_benchmark_gate_policy};
pub use qualification::{
    evaluate_scientific_acceptance, ScientificAcceptanceObservation, ScientificAssessmentStatus,
    ScientificExecutionStatus, SCIENTIFIC_ACCEPTANCE_RULE_V1,
};
