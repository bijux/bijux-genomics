//! BAM domain definitions and contracts.
//!
//! Owns: BAM stage semantics, effective params, and canonical metrics schema.
//! Must NOT depend on: bijux-engine or runtime/container execution logic.

pub mod invariants;
pub mod metrics;
pub mod params;
pub mod pipeline_contract;
pub mod stage_registry;
pub mod types;

pub use stage_registry::{
    contract_for_stage, required_audit_artifacts, stage_spec, stage_specs, ArtifactPolicy,
    AuditArtifact, BamArtifactKind, BamStage, BamStageContract, BamStageSpec,
};

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy)]
pub struct StageCompleteness {
    pub has_args_builder: bool,
    pub has_artifact_contract: bool,
    pub has_parser_fixtures: bool,
    pub has_invariants: bool,
}

impl StageCompleteness {
    #[must_use]
    pub const fn is_complete(self) -> bool {
        self.has_args_builder
            && self.has_artifact_contract
            && self.has_parser_fixtures
            && self.has_invariants
    }
}

#[must_use]
pub fn bam_stage_is_complete(stage: BamStage) -> bool {
    bam_stage_completeness(stage).is_complete()
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
pub fn bam_stage_completeness(stage: BamStage) -> StageCompleteness {
    let spec = stage_spec(stage);
    let has_artifacts = !required_audit_artifacts(stage).is_empty()
        && !spec.artifact_policy.required_outputs.is_empty();
    let has_args_builder = matches!(
        stage,
        BamStage::Align
            | BamStage::Validate
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
            | BamStage::Recalibration
    );
    let has_parser_fixtures = matches!(
        stage,
        BamStage::Validate
            | BamStage::QcPre
            | BamStage::Filter
            | BamStage::Coverage
            | BamStage::Damage
    );
    let has_invariants = bam_stage_has_invariants(stage);
    StageCompleteness {
        has_args_builder,
        has_artifact_contract: has_artifacts,
        has_parser_fixtures,
        has_invariants,
    }
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
