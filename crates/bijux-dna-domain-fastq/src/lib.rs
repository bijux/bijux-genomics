//! FASTQ domain definitions and contracts.
//!
//! Owns: FASTQ stage semantics, invariants, and contracts.
//! Must NOT depend on: bijux-dna-engine or runtime/container execution logic.
// Reading order:
// 1. domain.rs
// 2. core types
// 3. stage semantics
// 4. metrics spec
// 5. domain adapter
// Structural layout of this crate is frozen as of FASTQ v1.
pub mod banks;
pub mod bench_repository;
mod comparison_contract;
mod domain_adapter;
pub mod execution_support;
mod filter_artifacts;
pub mod id_catalog;
mod index_reference_artifacts;
mod infer_asvs_artifacts;
mod integration_matrix;
pub mod invariants;
pub mod metrics;
mod merge_pairs_artifacts;
mod normalize_abundance_artifacts;
mod normalize_primers_artifacts;
mod observer_contract;
pub mod params;
pub mod pipeline_contract;
pub mod prelude;
mod profile_overrepresented_artifacts;
mod profile_reads_artifacts;
mod profile_read_lengths_artifacts;
mod qc_contract;
mod remove_chimeras_artifacts;
mod remove_duplicates_artifacts;
mod report_qc_artifacts;
pub mod run;
mod screen_taxonomy_artifacts;
pub mod stage_contract;
pub mod stage_semantics;
pub mod stage_specs;
mod stage_tool_governance;
mod trim_polyg_artifacts;
mod trim_artifacts;
mod terminal_damage_artifacts;
mod validation_artifacts;
pub mod stages;
pub mod types;

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
pub use bench_repository::{
    governed_stage_bench_query_context, BenchQueryContext, BenchResultsRepository,
};
pub use screen_taxonomy_artifacts::{
    ScreenTaxonomyReportV1, TaxonomyScreenSummaryEntryV1,
    SCREEN_TAXONOMY_REPORT_SCHEMA_VERSION,
};
pub use comparison_contract::{
    benchmark_comparison_artifact_ids, comparison_artifact_ids_for_stage,
    comparison_contract_for_stage, comparison_input_artifact_ids_for_stage,
    StageComparisonContract,
};
pub use profile_reads_artifacts::{
    ProfileReadsHistogramBinV1, ProfileReadsMateSummaryV1, ProfileReadsReportV1,
    PROFILE_READS_REPORT_SCHEMA_VERSION,
};
pub use merge_pairs_artifacts::{MergePairsReportV1, MERGE_PAIRS_REPORT_SCHEMA_VERSION};
pub use normalize_primers_artifacts::{
    NormalizePrimersReportV1, NORMALIZE_PRIMERS_REPORT_SCHEMA_VERSION,
};
pub use normalize_abundance_artifacts::{
    NormalizeAbundanceReportV1, NORMALIZE_ABUNDANCE_REPORT_SCHEMA_VERSION,
};
pub use profile_overrepresented_artifacts::{
    OverrepresentedSequenceRowV1, ProfileOverrepresentedReportV1,
    PROFILE_OVERREPRESENTED_REPORT_SCHEMA_VERSION,
};
pub use profile_read_lengths_artifacts::{
    ProfileReadLengthBinV1, ProfileReadLengthsReportV1,
    PROFILE_READ_LENGTHS_REPORT_SCHEMA_VERSION,
};
pub use report_qc_artifacts::{
    GovernedQcContributorV1, ReportQcReportV1, REPORT_QC_REPORT_SCHEMA_VERSION,
};
pub use remove_chimeras_artifacts::{
    RemoveChimerasReportV1, REMOVE_CHIMERAS_REPORT_SCHEMA_VERSION,
};
pub use remove_duplicates_artifacts::{
    DuplicateClassEntryV1, RemoveDuplicatesProvenanceV1, RemoveDuplicatesReportV1,
    REMOVE_DUPLICATES_PROVENANCE_SCHEMA_VERSION, REMOVE_DUPLICATES_REPORT_SCHEMA_VERSION,
};
pub use execution_support::{
    admitted_tools_for_stage as admitted_execution_tools_for_stage, all_stage_execution_support,
    closed_stage_ids as execution_closed_stage_ids,
    declared_only_stage_ids as execution_declared_only_stage_ids,
    default_tool_for_stage as default_execution_tool_for_stage, execution_support_for_stage,
    ExecutionStatus, StageExecutionSupport,
};
pub use filter_artifacts::{FilterReadsReportV1, FILTER_READS_REPORT_SCHEMA_VERSION};
pub use infer_asvs_artifacts::{InferAsvsReportV1, INFER_ASVS_REPORT_SCHEMA_VERSION};
pub use index_reference_artifacts::{
    IndexReferenceFileEntryV1, IndexReferenceReportV1, INDEX_REFERENCE_REPORT_SCHEMA_VERSION,
};
pub use id_catalog::{
    FastqInvariantsPreset, FASTQ_METRICS_CATALOG, FASTQ_PARAMS_CATALOG, FASTQ_STAGE_ID_CATALOG,
};
pub use integration_matrix::{
    benchmark_scenarios, benchmark_scenarios_for_stage, is_reference_index_backend_compatible,
    reference_index_backends_for_tool, stage_tool_binding, stage_tool_bindings,
    stage_tool_bindings_for_stage, BenchmarkScenario, StageToolBinding, ToolIntegrationLevel,
};
pub use invariants::{
    evaluate_invariants, fastq_invariant_specs, thresholds_from_env, validate_edna_table,
    InvariantEvaluation, InvariantThresholds,
};
pub use metrics::{
    BrackenClassificationMetricsV1, BrackenRecordV1, ClassificationDbProvenanceV1,
    FastqQScoreSummaryV1, FastqQcSummaryMetricsV1, FastqScanMetricsV1,
    KrakenUniqClassificationMetricsV1, KrakenUniqRecordV1, SeqfuMetricsV1, TaxonomyRecordV1,
};
pub use observer_contract::{
    is_observer_specialized_stage_tool, observer_semantic_surface_for_stage_tool,
    observer_specialization_contract_for_stage_tool, observer_specialization_contracts,
    observer_specialized_stage_tool_bindings, ObserverSpecializationContract,
};
pub use params::correct::FastqCorrectParams;
pub use params::defaults::{correct_defaults, stats_defaults, umi_defaults};
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
pub use terminal_damage_artifacts::{
    TerminalDamageReportV1, TERMINAL_DAMAGE_REPORT_SCHEMA_VERSION,
};
pub use trim_polyg_artifacts::{TrimPolygReportV1, TRIM_POLYG_REPORT_SCHEMA_VERSION};
pub use trim_artifacts::{TrimReadsReportV1, TRIM_READS_REPORT_SCHEMA_VERSION};
pub use types::{
    AdapterContributionV1, AdapterTrimmingReportV1, FastqArtifact, FastqArtifactKind, FastqLayout,
    FastqPE, FastqPairedEnd, FastqSE, FastqSampleId, FastqSingleEnd, FastqStats, RetentionReportV1,
    ToolReferenceV1,
};
pub use validation_artifacts::{
    ValidatedReadsManifestV1, ValidateFailureClass, ValidationReportV1,
    VALIDATED_READS_MANIFEST_SCHEMA_VERSION, VALIDATION_REPORT_SCHEMA_VERSION,
};
