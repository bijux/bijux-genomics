//! Pure persisted BAM artifact models, grouped by scientific responsibility.

mod alignment;
mod ancient_dna;
mod corpus;
mod coverage;
mod duplicates;
mod filtering;
mod identity;
mod population;
mod recalibration;
mod resources;
mod workflow;

pub use alignment::{
    BamAlignmentProvenanceV1, BamAlignmentStrategyV1, BamAlignmentSuitabilityV1,
    BamAlignmentTruthCigarClassV1, BamAlignmentTruthMapqClassV1, BamAlignmentTruthPositionV1,
    BamAlignmentTruthRecordV1, BamAlignmentTruthSummaryV1, BamFlagstatCountsV1,
    BamMappingSummaryV1, BamMapqRegimeV1, BamMergeCompatibilityV1, BamMergeInputIdentityV1,
    BamPostAlignmentChainV1, BamQcPreSummaryV1, BamReadGroupMappingCountV1,
    BamTinyAlignmentInspectionV1, BamValidationSummaryV1, BAM_ALIGNMENT_PROVENANCE_SCHEMA_VERSION,
    BAM_ALIGNMENT_STRATEGY_SCHEMA_VERSION, BAM_ALIGNMENT_TRUTH_SUMMARY_SCHEMA_VERSION,
    BAM_MAPPING_SUMMARY_SCHEMA_VERSION, BAM_MERGE_COMPATIBILITY_SCHEMA_VERSION,
    BAM_POST_ALIGNMENT_CHAIN_SCHEMA_VERSION, BAM_QC_PRE_SUMMARY_SCHEMA_VERSION,
    BAM_VALIDATION_SUMMARY_SCHEMA_VERSION,
};
pub use ancient_dna::{
    BamAdnaContaminationTruthSummaryV1, BamAdnaDamageTruthSummaryV1, BamAdnaWorkflowV1,
    BamAuthenticityAdvisoryV1, BamContaminationEvidenceV1, BamContaminationToolContractV1,
    BamContaminationWorkflowV1, BamDamageEvidenceV1,
    BAM_ADNA_CONTAMINATION_TRUTH_SUMMARY_SCHEMA_VERSION,
    BAM_ADNA_DAMAGE_TRUTH_SUMMARY_SCHEMA_VERSION, BAM_ADNA_WORKFLOW_SCHEMA_VERSION,
    BAM_AUTHENTICITY_ADVISORY_SCHEMA_VERSION, BAM_CONTAMINATION_EVIDENCE_SCHEMA_VERSION,
    BAM_CONTAMINATION_WORKFLOW_SCHEMA_VERSION, BAM_DAMAGE_EVIDENCE_SCHEMA_VERSION,
};
pub use corpus::{
    BamBenchCorpusDatasetManifestEntryV1, BamBenchCorpusManifestV1, BamBenchDatasetScenarioV1,
    BAM_BENCH_CORPUS_MANIFEST_SCHEMA_VERSION,
};
pub use coverage::{
    BamBiasMitigationSummaryV1, BamCoverageRegimeClassV1, BamCoverageRegimeV1,
    BamCoverageRegionSummaryV1, BamCoverageSummaryV1, BamCoverageTruthSummaryV1,
    BamGcBiasBinSummaryV1, BamGcBiasSummaryV1, BamGcBiasTruthSummaryV1,
    BamInsertSizeHistogramBinV1, BamInsertSizeSummaryV1,
    BAM_BIAS_MITIGATION_SUMMARY_SCHEMA_VERSION, BAM_COVERAGE_REGIME_SCHEMA_VERSION,
    BAM_COVERAGE_SUMMARY_SCHEMA_VERSION, BAM_COVERAGE_TRUTH_SUMMARY_SCHEMA_VERSION,
    BAM_GC_BIAS_SUMMARY_SCHEMA_VERSION, BAM_GC_BIAS_TRUTH_SUMMARY_SCHEMA_VERSION,
    BAM_INSERT_SIZE_SUMMARY_SCHEMA_VERSION,
};
pub use duplicates::{
    BamComplexitySummaryV1, BamDuplicateComparisonV1, BamDuplicateFamilyHistogramBinV1,
    BamDuplicateInsertTruthSummaryV1, BamDuplicateMethodMetricsV1, BamDuplicatePolicyV1,
    BamDuplicationMetricsSummaryV1, BamMarkdupSummaryV1, BAM_COMPLEXITY_SUMMARY_SCHEMA_VERSION,
    BAM_DUPLICATE_COMPARISON_SCHEMA_VERSION, BAM_DUPLICATE_INSERT_TRUTH_SUMMARY_SCHEMA_VERSION,
    BAM_DUPLICATE_POLICY_SCHEMA_VERSION, BAM_DUPLICATION_METRICS_SUMMARY_SCHEMA_VERSION,
    BAM_MARKDUP_SUMMARY_SCHEMA_VERSION,
};
pub use filtering::{
    BamFilterSummaryV1, BamLengthFilterSummaryV1, BamMapqFilterSummaryV1,
    BamOverlapCorrectionSummaryV1, BAM_FILTER_SUMMARY_SCHEMA_VERSION,
    BAM_LENGTH_FILTER_SUMMARY_SCHEMA_VERSION, BAM_MAPQ_FILTER_SUMMARY_SCHEMA_VERSION,
    BAM_OVERLAP_CORRECTION_SUMMARY_SCHEMA_VERSION,
};
pub use identity::{
    BamArtifactEntryV1, BamArtifactInventoryV1, BamReferenceAssetIdentityV1,
    BamReferencePreflightV1, BamSampleIdentityV1, BAM_ARTIFACT_INVENTORY_SCHEMA_VERSION,
    BAM_REFERENCE_PREFLIGHT_SCHEMA_VERSION, BAM_SAMPLE_IDENTITY_SCHEMA_VERSION,
};
pub use population::{
    BamEndogenousContentEstimateV1, BamEndogenousTruthSummaryV1, BamHaplogroupReadinessV1,
    BamHaplogroupTruthSummaryV1, BamKinshipPairResultV1, BamKinshipPrerequisitesV1,
    BamKinshipSummaryV1, BamScientificReportContractV1, BamScientificReportIdV1,
    BamSexInferenceEvidenceV1, BamSexSummaryV1, BAM_ENDOGENOUS_CONTENT_SCHEMA_VERSION,
    BAM_ENDOGENOUS_TRUTH_SUMMARY_SCHEMA_VERSION, BAM_HAPLOGROUP_READINESS_SCHEMA_VERSION,
    BAM_HAPLOGROUP_TRUTH_SUMMARY_SCHEMA_VERSION, BAM_KINSHIP_PREREQUISITES_SCHEMA_VERSION,
    BAM_KINSHIP_SUMMARY_SCHEMA_VERSION, BAM_SCIENTIFIC_REPORT_SCHEMA_VERSION,
    BAM_SEX_EVIDENCE_SCHEMA_VERSION, BAM_SEX_SUMMARY_SCHEMA_VERSION,
};
pub use recalibration::{
    BamRecalibrationCoverageGateV1, BamRecalibrationSummaryV1,
    BAM_RECALIBRATION_SUMMARY_SCHEMA_VERSION,
};
pub use resources::{
    BamInputOriginV1, BamInputScaleV1, BamStageResourcePlanV1, BAM_RESOURCE_PLAN_SCHEMA_VERSION,
};
pub use workflow::{
    BamAdvisoryBoundaryV1, BamWorkflowModeV1, BamWorkflowTemplateV1,
    BAM_ADVISORY_BOUNDARY_SCHEMA_VERSION, BAM_WORKFLOW_TEMPLATE_SCHEMA_VERSION,
};
