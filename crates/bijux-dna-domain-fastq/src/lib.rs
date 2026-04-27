//! FASTQ domain definitions and contracts.
//!
//! Owns: FASTQ stage semantics, invariants, and contracts.
//! Must NOT depend on: bijux-dna-engine or runtime/container execution logic.
//! Reading order: `stages`, `params`, `metrics`, `invariants`, `banks`, then `observer`.
//! Structural layout is documented in `docs/ARCHITECTURE.md`.
mod artifacts;
pub mod banks;
mod bench;
pub mod bench_repository;
mod comparison_contract;
mod domain_adapter;
pub mod execution_support;
pub mod id_catalog;
mod integration_matrix;
pub mod invariants;
pub mod metrics;
pub mod observer;
pub mod params;
pub mod pipeline_contract;
pub mod prelude;
mod qc_contract;
pub mod run;
mod stage_tool_governance;
pub mod stages;
pub mod types;

pub use artifacts::{ClusterOtusReportV1, CLUSTER_OTUS_REPORT_SCHEMA_VERSION};
pub use artifacts::{CorrectErrorsReportV1, CORRECT_ERRORS_REPORT_SCHEMA_VERSION};
pub use artifacts::{DepleteHostReportV1, DEPLETE_HOST_REPORT_SCHEMA_VERSION};
pub use artifacts::{
    DepleteReferenceContaminantsReportV1, DEPLETE_REFERENCE_CONTAMINANTS_REPORT_SCHEMA_VERSION,
};
pub use artifacts::{DepleteRrnaReportV1, DEPLETE_RRNA_REPORT_SCHEMA_VERSION};
pub use artifacts::{DetectAdaptersReportV1, DETECT_ADAPTERS_REPORT_SCHEMA_VERSION};
pub use artifacts::{
    DuplicateClassEntryV1, RemoveDuplicatesProvenanceV1, RemoveDuplicatesReportV1,
    REMOVE_DUPLICATES_PROVENANCE_SCHEMA_VERSION, REMOVE_DUPLICATES_REPORT_SCHEMA_VERSION,
};
pub use artifacts::{ExtractUmisReportV1, EXTRACT_UMIS_REPORT_SCHEMA_VERSION};
pub use artifacts::{FilterLowComplexityReportV1, FILTER_LOW_COMPLEXITY_REPORT_SCHEMA_VERSION};
pub use artifacts::{FilterReadsReportV1, FILTER_READS_REPORT_SCHEMA_VERSION};
pub use artifacts::{GovernedQcContributorV1, ReportQcReportV1, REPORT_QC_REPORT_SCHEMA_VERSION};
pub use artifacts::{
    IndexReferenceFileEntryV1, IndexReferenceReportV1, INDEX_REFERENCE_REPORT_SCHEMA_VERSION,
};
pub use artifacts::{InferAsvsReportV1, INFER_ASVS_REPORT_SCHEMA_VERSION};
pub use artifacts::{MergePairsReportV1, MERGE_PAIRS_REPORT_SCHEMA_VERSION};
pub use artifacts::{NormalizeAbundanceReportV1, NORMALIZE_ABUNDANCE_REPORT_SCHEMA_VERSION};
pub use artifacts::{NormalizePrimersReportV1, NORMALIZE_PRIMERS_REPORT_SCHEMA_VERSION};
pub use artifacts::{
    OverrepresentedSequenceRowV1, ProfileOverrepresentedReportV1,
    PROFILE_OVERREPRESENTED_REPORT_SCHEMA_VERSION,
};
pub use artifacts::{
    ProfileReadLengthBinV1, ProfileReadLengthsReportV1, PROFILE_READ_LENGTHS_REPORT_SCHEMA_VERSION,
};
pub use artifacts::{
    ProfileReadsHistogramBinV1, ProfileReadsMateSummaryV1, ProfileReadsReportV1,
    PROFILE_READS_REPORT_SCHEMA_VERSION,
};
pub use artifacts::{RemoveChimerasReportV1, REMOVE_CHIMERAS_REPORT_SCHEMA_VERSION};
pub use artifacts::{
    ScreenTaxonomyReportV1, TaxonomyScreenSummaryEntryV1, SCREEN_TAXONOMY_REPORT_SCHEMA_VERSION,
};
pub use artifacts::{TerminalDamageReportV1, TERMINAL_DAMAGE_REPORT_SCHEMA_VERSION};
pub use artifacts::{TrimPolygReportV1, TRIM_POLYG_REPORT_SCHEMA_VERSION};
pub use artifacts::{TrimReadsReportV1, TRIM_READS_REPORT_SCHEMA_VERSION};
pub use artifacts::{
    ValidateFailureClass, ValidatedReadsManifestV1, ValidationReportV1,
    VALIDATED_READS_MANIFEST_SCHEMA_VERSION, VALIDATION_REPORT_SCHEMA_VERSION,
};
pub use banks::{
    adapter_bank_path, adapter_categories, adapter_presets_path, adapters_by_category,
    load_adapter_bank, load_adapter_presets, resolve_adapter_preset, AdapterBankV1, AdapterEntryV1,
    AdapterPresetV1, AdapterPresetsV1, EffectiveAdapterSet, ReadScope,
};
pub use banks::{
    contaminant_motifs_path, contaminant_presets_path, contaminant_references_dir,
    load_contaminant_motifs, load_contaminant_presets, resolve_contaminant_preset,
    ContaminantMotifBankV1, ContaminantMotifEntryV1, ContaminantPresetV1, ContaminantPresetsV1,
    ContaminantReferenceSpecV1, EffectiveContaminantSet,
};
pub use banks::{
    load_polyx_bank, load_polyx_presets, polyx_bank_path, polyx_presets_path, resolve_polyx_preset,
    EffectivePolyxSet, PolyxBankV1, PolyxEntryV1, PolyxPresetV1, PolyxPresetsV1,
};
pub use comparison_contract::{
    benchmark_comparison_artifact_ids, comparison_artifact_ids_for_stage,
    comparison_contract_for_stage, comparison_input_artifact_ids_for_stage,
    StageComparisonContract,
};
pub use domain_adapter::FastqDomain;
pub use execution_support::{
    admitted_tools_for_stage as admitted_execution_tools_for_stage, all_stage_execution_support,
    closed_stage_ids as execution_closed_stage_ids,
    declared_only_stage_ids as execution_declared_only_stage_ids,
    default_tool_for_stage as default_execution_tool_for_stage, execution_support_for_stage,
    ExecutionStatus, StageExecutionSupport,
};
pub use id_catalog::{
    FastqInvariantsPreset, FASTQ_METRICS_CATALOG, FASTQ_PARAMS_CATALOG, FASTQ_STAGE_ID_CATALOG,
};
pub use integration_matrix::{
    benchmark_scenarios, benchmark_scenarios_for_stage, is_reference_index_backend_compatible,
    governed_tool_ids_for_stage, planned_tool_ids_for_stage, reference_index_backends_for_tool,
    registered_tool_ids_for_stage, stage_tool_binding, stage_tool_bindings,
    stage_tool_bindings_for_stage, BenchmarkScenario, StageToolBinding, ToolIntegrationLevel,
};
pub use invariants::{
    evaluate_invariants, fastq_invariant_specs, thresholds_from_env, validate_edna_table,
    InvariantEvaluation, InvariantThresholds, EVALUATED_STAGES,
};
pub use metrics::{
    BrackenClassificationMetricsV1, BrackenRecordV1, ClassificationDbProvenanceV1,
    FastqQScoreSummaryV1, FastqQcSummaryMetricsV1, FastqScanMetricsV1,
    KrakenUniqClassificationMetricsV1, KrakenUniqRecordV1, SeqfuMetricsV1, TaxonomyRecordV1,
};
pub use observer::contracts::{
    is_observer_specialized_stage_tool, observer_semantic_surface_for_stage_tool,
    observer_specialization_contract_for_stage_tool, observer_specialization_contracts,
    observer_specialized_stage_tool_bindings, ObserverSpecializationContract,
};
pub use params::correct::FastqCorrectParams;
pub use params::defaults::{
    correct_defaults, overrepresented_profile_defaults, read_length_profile_defaults,
    stats_defaults, umi_defaults,
};
pub use params::stats::{
    FastqOverrepresentedProfileParams, FastqReadLengthProfileParams, FastqStatsParams,
};
pub use params::trim::{
    AlienTrimmerParamsV1, FastxClipperParamsV1, LeeHomTrimParamsV1, OverlapCollapseMode,
    ReadHandlingMode, SkewerTrimParamsV1, TrimAdapterMode, TrimQualityMode, TrimToolParamsV1,
};
pub use params::umi::FastqUmiParams;
pub use params::{
    parse_effective_params, stage_param_descriptor, EffectiveParams, PairedMode,
    StageParamDescriptor,
};
pub use pipeline_contract::{
    canonical_amplicon_stage_order, canonical_stage_order, default_amplicon_preprocess_stage_order,
    default_shotgun_preprocess_stage_order, forbidden_transitions, optional_branches,
    preprocess_pipeline_graph_for_stage_order, FastqPipelineMode, StageCriticality,
};
pub use qc_contract::{
    governed_qc_bench_contributor_stage_ids, governed_qc_default_tool_ids,
    governed_qc_output_ids_for_stage, governed_qc_producer_stage_ids,
};
pub use run::{assess_input_dir, discover_fastq_files};
pub use run::{bench_corpus, BenchCorpus, BenchCorpusId, BenchDataset};
pub use stage_tool_governance::{
    benchmark_readiness_for_stage_tool, filter_tools_for_input_layout, stage_benchmark_governance,
    stage_tool_capability_contract, stage_tool_governance_profile,
    stage_tool_governance_profiles_for_stage, stage_tool_maturity, tool_supports_input_layout,
    BenchmarkReadinessLevel, RuntimeNormalizationLevel, StageBenchmarkGovernance,
    StageToolBenchmarkContractMaturity, StageToolCapabilityContract, StageToolGovernanceProfile,
    StageToolMaturityLevel, StageToolNormalizationMaturity,
};
pub use stages::{
    assess_merge_suitability, contract_for_stage, ensure_umi_headers, inspect_headers,
    log_header_warnings, normalize_outputs, preflight_stage, stage_contract_hash,
    stage_contract_json, HeaderInspection, MergeSuitability, NormalizedOutputs,
};
pub use stages::{
    bench_dir_name, STAGES, STAGE_CLUSTER_OTUS, STAGE_CORRECT_ERRORS, STAGE_DEPLETE_RRNA,
    STAGE_DETECT_ADAPTERS, STAGE_EXTRACT_UMIS, STAGE_FILTER_LOW_COMPLEXITY, STAGE_FILTER_READS,
    STAGE_INFER_ASVS, STAGE_MERGE_PAIRS, STAGE_NORMALIZE_ABUNDANCE, STAGE_NORMALIZE_PRIMERS,
    STAGE_PREFIX, STAGE_PROFILE_READS, STAGE_REMOVE_CHIMERAS, STAGE_REMOVE_DUPLICATES,
    STAGE_REPORT_QC, STAGE_SCREEN_TAXONOMY, STAGE_TRIM_READS, STAGE_TRIM_TERMINAL_DAMAGE,
    STAGE_VALIDATE_READS,
};
pub use stages::{
    canonical_contract_for_stage, infer_input_kind, qc_class_for_stage, FastqStage,
    FastqStageContract, QcClass, StageContract, StageIO,
};
pub use stages::{
    fastq_stage_is_stable, stage_compatible_tool_ids, stage_criticality, stage_input_ids,
    stage_kind, stage_metric_classes, stage_metric_invariants, stage_output_ids,
    stage_parameter_ids, stage_semantics, BoundaryInvariant, FastqStageKind, StageDefinition,
    StageSemantics, STAGE_BOUNDARY_INVARIANTS,
};
pub use types::{
    AdapterContributionV1, AdapterTrimmingReportV1, FastqArtifact, FastqArtifactKind, FastqLayout,
    FastqPE, FastqPairedEnd, FastqSE, FastqSampleId, FastqSingleEnd, FastqStats, RetentionReportV1,
    ToolReferenceV1,
};

pub use bench_repository::{
    governed_stage_bench_query_context, BenchQueryContext, BenchResultsRepository,
};

pub mod stage_contract {
    pub use crate::stages::contract::{stage_contract_hash, stage_contract_json};
}

pub mod stage_semantics {
    pub use crate::stages::semantics::{
        canonical_stage_order, fastq_stage_is_stable, optional_branches, stage_criticality,
        stage_kind, stage_metric_classes, stage_metric_invariants, stage_semantics,
        BoundaryInvariant, FastqStageKind, StageDefinition, StageSemantics,
        STAGE_BOUNDARY_INVARIANTS,
    };
}

pub mod stage_specs {
    pub use crate::stages::specs::{
        canonical_contract_for_stage, infer_input_kind, qc_class_for_stage, FastqStage,
        FastqStageContract, QcClass, StageContract, StageIO,
    };
}
