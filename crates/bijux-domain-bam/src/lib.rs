//! BAM domain definitions and contracts.
//!
//! Owns: BAM stage semantics, effective params, and canonical metrics schema.
//! Must NOT depend on: bijux-engine or runtime/container execution logic.

mod bam_stage_registry;
mod invariants;
pub mod metrics;
pub mod params;
pub mod types;

pub use bam_stage_registry::{
    contract_for_stage, required_audit_artifacts, stage_spec, stage_specs, ArtifactPolicy,
    AuditArtifact, BamArtifactKind, BamStage, BamStageContract, BamStageSpec,
};
