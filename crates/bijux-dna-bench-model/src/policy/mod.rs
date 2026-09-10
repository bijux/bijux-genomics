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

pub(crate) mod command_audit;

pub(crate) mod command_artifacts;

pub(crate) mod micro_report;

pub(crate) mod hpc_array;

pub(crate) mod hpc_resources;

pub(crate) mod hardening_campaign;

pub(crate) mod hpc_asset_staging;

pub(crate) mod hpc_scratch;

pub(crate) mod runtime_resolution;

pub(crate) mod tool_execution_modes;
