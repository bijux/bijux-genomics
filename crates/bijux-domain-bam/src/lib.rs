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

#[must_use]
pub fn bam_stage_is_complete(stage: BamStage) -> bool {
    let spec = stage_spec(stage);
    let audit = required_audit_artifacts(stage);
    if audit.is_empty() {
        return false;
    }
    if spec.artifact_policy.required_outputs.is_empty() {
        return false;
    }
    bam_stage_has_invariants(stage)
}

#[must_use]
pub fn bam_stage_has_invariants(stage: BamStage) -> bool {
    matches!(
        stage,
        BamStage::Validate
            | BamStage::QcPre
            | BamStage::Filter
            | BamStage::Markdup
            | BamStage::Complexity
            | BamStage::Coverage
            | BamStage::Damage
            | BamStage::Authenticity
            | BamStage::Contamination
            | BamStage::Sex
            | BamStage::BiasMitigation
    )
}

#[must_use]
pub fn bam_stage_is_stable(stage: BamStage) -> bool {
    matches!(
        stage,
        BamStage::Validate
            | BamStage::QcPre
            | BamStage::Filter
            | BamStage::Coverage
            | BamStage::Damage
    )
}
