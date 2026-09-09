//! Owner: bijux-dna-bench-model
//! Policy engine for bench gating.

pub mod fastq_scoring;
pub mod gate_policy;
pub mod outcomes;

pub use gate_policy::{GatePolicy, GatePolicyOverrides};
pub use outcomes::{gate_decision_schema, GateDecision, GateViolation};
