//! BAM domain definitions and contracts.
//!
//! Owns: BAM stage semantics, effective params, and canonical metrics schema.
//! Must NOT depend on: bijux-dna-engine or runtime/container execution logic.
#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::expect_used,
    clippy::fn_params_excessive_bools,
    clippy::format_push_string,
    clippy::if_not_else,
    clippy::large_enum_variant,
    clippy::map_unwrap_or,
    clippy::match_same_arms,
    clippy::missing_panics_doc,
    clippy::similar_names,
    clippy::single_match_else,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    clippy::unnecessary_semicolon
)]

pub mod alignment;
mod artifacts;
pub mod defaults;
pub mod invariants;
pub mod metrics;
pub mod params;
pub mod pipeline_contract;
pub mod prelude;
pub mod stage_specs;
pub mod types;

pub use artifacts::{
    align_fastq_to_bam_bowtie2_style, align_fastq_to_bam_bwa_style,
    apply_duplicate_policy_tiny_bam, bam_adna_workflow_contract, bam_alignment_strategies,
    bam_alignment_strategy_for_tool, bam_artifact_inventory_from_outputs,
    bam_bench_corpus_manifest, bam_contamination_workflow_contract, bam_post_alignment_chain,
    bam_sample_identity, bam_scientific_report_contract_for_stage, bam_scientific_report_contracts,
    bam_workflow_template_by_id, bam_workflow_templates, classify_bam_coverage_regime,
    compare_bam_duplicate_methods, correct_tiny_bam_overlaps, estimate_bam_stage_resources,
    estimate_bam_stage_resources_with_origin, estimate_endogenous_content,
    evaluate_bam_merge_compatibility, evaluate_haplogroup_readiness,
    evaluate_kinship_prerequisites, evaluate_sex_inference_par_aware,
    execute_ancient_damage_evidence, execute_bam_validation,
    execute_mitochondrial_contamination_workflow, execute_nuclear_contamination_workflow,
    execute_pmd_authenticity_advisory, filter_tiny_bam, filter_tiny_bam_by_length,
    filter_tiny_bam_by_mapq, merge_tiny_bam_with_conflict_refusal, propagate_bam_sample_identity,
    required_bam_bench_corpus_scenarios, sort_and_index_tiny_bam, summarize_bam_bias_mitigation,
    summarize_bam_complexity, summarize_bam_duplication_metrics, summarize_bam_endogenous_content,
    summarize_bam_gc_bias, summarize_bam_insert_size, summarize_bam_markdup,
    summarize_bam_overlap_correction,
    summarize_tiny_bam_authenticity_advisory, summarize_tiny_bam_complexity,
    summarize_tiny_bam_bias_mitigation, summarize_tiny_bam_coverage,
    summarize_tiny_bam_coverage_regions, summarize_tiny_bam_damage_evidence,
    summarize_tiny_bam_duplication_metrics, summarize_tiny_bam_endogenous_content,
    summarize_tiny_bam_gc_bias, summarize_tiny_bam_insert_size, summarize_tiny_bam_mapping,
    summarize_tiny_bam_overlap_correction_outputs, summarize_tiny_bam_qc_pre,
    summarize_tiny_bam_sex, BamAdnaWorkflowV1, BamAdvisoryBoundaryV1, BamAlignmentProvenanceV1,
    BamAlignmentStrategyV1, BamAlignmentSuitabilityV1, BamArtifactEntryV1, BamArtifactInventoryV1,
    BamAuthenticityAdvisoryV1, BamBenchCorpusDatasetManifestEntryV1, BamBenchCorpusManifestV1,
    BamBenchDatasetScenarioV1, BamBiasMitigationSummaryV1, BamComplexitySummaryV1,
    BamContaminationEvidenceV1, BamContaminationToolContractV1, BamContaminationWorkflowV1,
    BamCoverageRegimeClassV1, BamCoverageRegimeV1, BamCoverageRegionSummaryV1,
    BamCoverageSummaryV1, BamDamageEvidenceV1, BamDuplicateComparisonV1,
    BamDuplicateMethodMetricsV1, BamDuplicatePolicyV1, BamDuplicationMetricsSummaryV1,
    BamEndogenousContentEstimateV1, BamFilterSummaryV1, BamFlagstatCountsV1,
    BamGcBiasBinSummaryV1, BamGcBiasSummaryV1, BamHaplogroupReadinessV1, BamInputOriginV1,
    BamInputScaleV1, BamInsertSizeSummaryV1, BamKinshipPrerequisitesV1,
    BamLengthFilterSummaryV1, BamMappingSummaryV1, BamMapqFilterSummaryV1, BamMapqRegimeV1,
    BamMarkdupSummaryV1, BamMergeCompatibilityV1, BamMergeInputIdentityV1,
    BamOverlapCorrectionSummaryV1, BamPostAlignmentChainV1, BamQcPreSummaryV1,
    BamReadGroupMappingCountV1, BamReferenceAssetIdentityV1, BamReferencePreflightV1,
    BamSampleIdentityV1, BamScientificReportContractV1, BamScientificReportIdV1,
    BamSexInferenceEvidenceV1, BamSexSummaryV1, BamStageResourcePlanV1, BamValidationSummaryV1,
    BamWorkflowModeV1, BamWorkflowTemplateV1, BAM_ADNA_WORKFLOW_SCHEMA_VERSION,
    BAM_ADVISORY_BOUNDARY_SCHEMA_VERSION, BAM_ALIGNMENT_PROVENANCE_SCHEMA_VERSION,
    BAM_ALIGNMENT_STRATEGY_SCHEMA_VERSION, BAM_ARTIFACT_INVENTORY_SCHEMA_VERSION,
    BAM_AUTHENTICITY_ADVISORY_SCHEMA_VERSION, BAM_BENCH_CORPUS_MANIFEST_SCHEMA_VERSION,
    BAM_BIAS_MITIGATION_SUMMARY_SCHEMA_VERSION, BAM_COMPLEXITY_SUMMARY_SCHEMA_VERSION,
    BAM_CONTAMINATION_EVIDENCE_SCHEMA_VERSION, BAM_CONTAMINATION_WORKFLOW_SCHEMA_VERSION,
    BAM_COVERAGE_REGIME_SCHEMA_VERSION, BAM_COVERAGE_SUMMARY_SCHEMA_VERSION,
    BAM_DAMAGE_EVIDENCE_SCHEMA_VERSION, BAM_DUPLICATE_COMPARISON_SCHEMA_VERSION,
    BAM_DUPLICATE_POLICY_SCHEMA_VERSION, BAM_DUPLICATION_METRICS_SUMMARY_SCHEMA_VERSION,
    BAM_ENDOGENOUS_CONTENT_SCHEMA_VERSION, BAM_FILTER_SUMMARY_SCHEMA_VERSION,
    BAM_GC_BIAS_SUMMARY_SCHEMA_VERSION, BAM_HAPLOGROUP_READINESS_SCHEMA_VERSION,
    BAM_INSERT_SIZE_SUMMARY_SCHEMA_VERSION, BAM_KINSHIP_PREREQUISITES_SCHEMA_VERSION,
    BAM_LENGTH_FILTER_SUMMARY_SCHEMA_VERSION, BAM_MAPPING_SUMMARY_SCHEMA_VERSION,
    BAM_MAPQ_FILTER_SUMMARY_SCHEMA_VERSION, BAM_MARKDUP_SUMMARY_SCHEMA_VERSION,
    BAM_MERGE_COMPATIBILITY_SCHEMA_VERSION, BAM_OVERLAP_CORRECTION_SUMMARY_SCHEMA_VERSION,
    BAM_POST_ALIGNMENT_CHAIN_SCHEMA_VERSION, BAM_QC_PRE_SUMMARY_SCHEMA_VERSION,
    BAM_REFERENCE_PREFLIGHT_SCHEMA_VERSION, BAM_RESOURCE_PLAN_SCHEMA_VERSION,
    BAM_SAMPLE_IDENTITY_SCHEMA_VERSION, BAM_SCIENTIFIC_REPORT_SCHEMA_VERSION,
    BAM_SEX_EVIDENCE_SCHEMA_VERSION, BAM_SEX_SUMMARY_SCHEMA_VERSION,
    BAM_VALIDATION_SUMMARY_SCHEMA_VERSION, BAM_WORKFLOW_TEMPLATE_SCHEMA_VERSION,
};
pub use invariants::bam_invariant_specs;
pub use stage_specs::{
    contract_for_stage, required_audit_artifacts, stage_contract_hash, stage_contract_json,
    stage_spec, stage_spec_opt, stage_specs, ArtifactPolicy, AuditArtifact, BamArtifactKind,
    BamStage, BamStageContract, BamStageSpec, StageSpec, STAGE_PREFIX,
};
pub use types::{
    BamInvariantsPreset, BAM_LOCAL_BENCH_STAGE_ID_CATALOG, BAM_METRICS_CATALOG, BAM_PARAMS_CATALOG,
    BAM_STAGE_ID_CATALOG,
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
