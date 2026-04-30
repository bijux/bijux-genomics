//! BAM domain definitions and contracts.
//!
//! Owns: BAM stage semantics, effective params, and canonical metrics schema.
//! Must NOT depend on: bijux-dna-engine or runtime/container execution logic.

mod artifacts;
pub mod alignment;
pub mod defaults;
pub mod invariants;
pub mod metrics;
pub mod params;
pub mod pipeline_contract;
pub mod prelude;
pub mod stage_specs;
pub mod types;

pub use invariants::bam_invariant_specs;
pub use artifacts::{
    bam_artifact_inventory_from_outputs, bam_sample_identity, bam_workflow_template_by_id,
    bam_workflow_templates, BamAdvisoryBoundaryV1, BamAlignmentProvenanceV1,
    BamArtifactEntryV1, BamArtifactInventoryV1, BamCoverageSummaryV1, BamDuplicatePolicyV1,
    BamFlagstatCountsV1, BamMapqRegimeV1, BamMappingSummaryV1, BamReferenceAssetIdentityV1,
    BamReferencePreflightV1, BamSampleIdentityV1, BamValidationSummaryV1,
    BamWorkflowModeV1, BamWorkflowTemplateV1, BAM_ADVISORY_BOUNDARY_SCHEMA_VERSION,
    BAM_ALIGNMENT_PROVENANCE_SCHEMA_VERSION, BAM_ARTIFACT_INVENTORY_SCHEMA_VERSION,
    BAM_COVERAGE_SUMMARY_SCHEMA_VERSION, BAM_DUPLICATE_POLICY_SCHEMA_VERSION,
    BAM_MAPPING_SUMMARY_SCHEMA_VERSION, BAM_REFERENCE_PREFLIGHT_SCHEMA_VERSION,
    BAM_SAMPLE_IDENTITY_SCHEMA_VERSION, BAM_VALIDATION_SUMMARY_SCHEMA_VERSION,
    BAM_WORKFLOW_TEMPLATE_SCHEMA_VERSION,
};
pub use stage_specs::{
    contract_for_stage, required_audit_artifacts, stage_contract_hash, stage_contract_json,
    stage_spec, stage_spec_opt, stage_specs, ArtifactPolicy, AuditArtifact, BamArtifactKind,
    BamStage, BamStageContract, BamStageSpec, StageSpec, STAGE_PREFIX,
};
pub use types::{
    BamInvariantsPreset, BAM_METRICS_CATALOG, BAM_PARAMS_CATALOG, BAM_STAGE_ID_CATALOG,
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
            | BamStage::MappingSummary
            | BamStage::QcPre
            | BamStage::Filter
            | BamStage::Markdup
            | BamStage::Complexity
            | BamStage::Coverage
            | BamStage::InsertSize
            | BamStage::GcBias
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
            | BamStage::MappingSummary
            | BamStage::QcPre
            | BamStage::Filter
            | BamStage::Markdup
            | BamStage::Complexity
            | BamStage::Coverage
            | BamStage::InsertSize
            | BamStage::GcBias
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
            | BamStage::MappingSummary
            | BamStage::QcPre
            | BamStage::Filter
            | BamStage::Coverage
            | BamStage::InsertSize
            | BamStage::GcBias
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
            | BamStage::MappingSummary
            | BamStage::QcPre
            | BamStage::Filter
            | BamStage::Coverage
            | BamStage::InsertSize
            | BamStage::GcBias
            | BamStage::Damage
    )
}
