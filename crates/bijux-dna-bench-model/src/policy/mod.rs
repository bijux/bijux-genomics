//! Owner: bijux-dna-bench
//! Policy engine for bench gating.

pub mod gate_policy;
pub mod outcomes;

pub use gate_policy::{GatePolicy, GatePolicyOverrides};
pub use outcomes::{GateDecision, GateViolation};
