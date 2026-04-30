//! VCF domain primitives: stage IDs, typed params, metrics, and registry materialization.

pub mod artifacts;
pub mod contracts;
pub mod coverage;
pub mod metrics;
pub mod params;
pub mod registry_emit;
pub mod run;
pub mod stage_baseline;
pub mod taxonomy;

pub use artifacts::{
    build_panel_reference_drift_report, build_vcf_scientific_drift_report,
    evaluate_demography_refusal_boundary, evaluate_diploid_calling_boundary,
    evaluate_genotype_likelihood_workflow_boundary, evaluate_imputation_workflow_boundary,
    evaluate_pca_admixture_guardrail, evaluate_phasing_workflow_boundary,
    evaluate_pseudohaploid_calling_boundary, evaluate_roh_ibd_workflow_boundary,
    evaluate_structural_variant_support_boundary, execute_annotation_provenance_workflow,
    execute_cohort_qc_workflow, execute_damage_aware_vcf_filter,
    execute_vcf_filter_with_explainable_consequences, execute_vcf_normalization_and_decomposition,
    execute_vcf_stats_workflow, execute_vcf_validation, resolve_vcf_reference_context,
    VcfAnnotationProvenanceWorkflowSummaryV1, VcfCallingBoundaryV1, VcfCohortQcSampleCaveatV1,
    VcfCohortQcWorkflowSummaryV1, VcfDamageFilterSummaryV1, VcfDemographyRefusalBoundaryV1,
    VcfFilterConsequenceV1, VcfImputationWorkflowBoundaryV1, VcfLikelihoodWorkflowBoundaryV1,
    VcfNormalizationSummaryV1, VcfPanelReferenceDriftReportV1, VcfPanelReferenceSnapshotV1,
    VcfPcaAdmixtureGuardrailV1, VcfPhasingWorkflowBoundaryV1, VcfReferenceContextResolutionV1,
    VcfRohIbdWorkflowBoundaryV1, VcfScientificDriftArtifactDeltaV1, VcfScientificDriftChangeKind,
    VcfScientificDriftMetricDeltaV1, VcfScientificDriftReportV1, VcfScientificDriftSnapshotV1,
    VcfStatsWorkflowSummaryV1, VcfStructuralVariantBoundaryV1, VcfValidationSummaryV1,
    VCF_ANNOTATION_PROVENANCE_WORKFLOW_SCHEMA_VERSION, VCF_COHORT_QC_WORKFLOW_SCHEMA_VERSION,
    VCF_DAMAGE_FILTER_SUMMARY_SCHEMA_VERSION, VCF_DEMOGRAPHY_REFUSAL_BOUNDARY_SCHEMA_VERSION,
    VCF_DIPLOID_CALLING_BOUNDARY_SCHEMA_VERSION, VCF_FILTER_CONSEQUENCE_SCHEMA_VERSION,
    VCF_GL_WORKFLOW_BOUNDARY_SCHEMA_VERSION, VCF_IMPUTATION_WORKFLOW_BOUNDARY_SCHEMA_VERSION,
    VCF_NORMALIZATION_SUMMARY_SCHEMA_VERSION, VCF_PANEL_REFERENCE_DRIFT_REPORT_SCHEMA_VERSION,
    VCF_PCA_ADMIXTURE_GUARDRAIL_SCHEMA_VERSION, VCF_PHASING_WORKFLOW_BOUNDARY_SCHEMA_VERSION,
    VCF_PSEUDOHAPLOID_CALLING_BOUNDARY_SCHEMA_VERSION, VCF_REFERENCE_CONTEXT_SCHEMA_VERSION,
    VCF_ROH_IBD_WORKFLOW_BOUNDARY_SCHEMA_VERSION, VCF_SCIENTIFIC_DRIFT_REPORT_SCHEMA_VERSION,
    VCF_STATS_WORKFLOW_SCHEMA_VERSION, VCF_STRUCTURAL_VARIANT_BOUNDARY_SCHEMA_VERSION,
    VCF_VALIDATION_SUMMARY_SCHEMA_VERSION,
};
pub use metrics::{VcfCallSummaryMetricsV1, VcfFilterBreakdownMetricsV1, VcfStatsMetricsV1};
pub use registry_emit::{param_registry_toml, required_tools_toml};
pub use run::{
    required_vcf_bench_corpus_scenarios, vcf_bench_corpus_datasets, vcf_bench_corpus_manifest,
    vcf_example_suite_manifest, VcfBenchCorpusDatasetManifestEntryV1, VcfBenchCorpusId,
    VcfBenchCorpusManifestV1, VcfBenchDataset, VcfBenchScenario, VcfExampleCaseId,
    VcfExampleCaseManifestEntryV1, VcfExampleSuiteManifestV1,
    VCF_BENCH_CORPUS_MANIFEST_SCHEMA_VERSION, VCF_EXAMPLE_SUITE_SCHEMA_VERSION,
};
pub use stage_baseline::{
    VcfInvariantsPreset, VcfStage, STAGE_CALL, STAGE_FILTER_READS, STAGE_PREFIX, STAGE_STATS,
};
pub use taxonomy::{
    validate_downstream_transition, CoverageRegime, DomainSupportStatus, VcfDomainStage,
    VcfStageKind, VCF_FORBIDDEN_TRANSITIONS, VCF_STAGE_ORDER_DOWNSTREAM, VCF_STAGE_TAXONOMY,
};

pub const VCF_STAGE_ID_CATALOG: &[&str] = &[
    "vcf.admixture",
    "vcf.call",
    "vcf.call_diploid",
    "vcf.call_gl",
    "vcf.call_pseudohaploid",
    "vcf.damage_filter",
    "vcf.demography",
    "vcf.filter",
    "vcf.gl_propagation",
    "vcf.ibd",
    "vcf.imputation",
    "vcf.impute",
    "vcf.pca",
    "vcf.phasing",
    "vcf.population_structure",
    "vcf.postprocess",
    "vcf.prepare_reference_panel",
    "vcf.qc",
    "vcf.roh",
    "vcf.stats",
];
pub const VCF_PARAMS_CATALOG: &[&str] = &[
    "bijux.vcf.call.params",
    "bijux.vcf.filter.params",
    "bijux.vcf.stats.params",
    "bijux.vcf.call_gl.params",
    "bijux.vcf.call_diploid.params",
    "bijux.vcf.call_pseudohaploid.params",
    "bijux.vcf.damage_filter.params",
    "bijux.vcf.gl_propagation.params",
];
pub const VCF_METRICS_CATALOG: &[&str] =
    &["bijux.vcf.call_summary.v1", "bijux.vcf.filter_breakdown.v1", "bijux.vcf.stats.v1"];
pub const VCF_PRODUCTION_TOOLS: &[&str] = &["bcftools"];
