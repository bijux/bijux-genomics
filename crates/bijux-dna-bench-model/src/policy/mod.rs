//! Owner: bijux-dna-bench-model
//! Policy engine for bench gating.

pub mod fastq_scoring;
pub mod gate_policy;
pub mod outcomes;

pub use gate_policy::{GatePolicy, GatePolicyOverrides};
pub use outcomes::{gate_decision_schema, GateDecision, GateViolation};

pub mod bam_scoring;

pub mod vcf_scoring;

pub mod output_audit;

pub mod operational_readiness;

pub mod hpc_campaign;

pub(crate) mod campaign_monitor;

pub(crate) mod campaign_copy_back;
