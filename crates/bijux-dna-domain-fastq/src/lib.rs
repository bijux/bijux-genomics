//! FASTQ domain definitions and contracts.
//!
//! Owns: FASTQ stage semantics, invariants, and contracts.
//! Must NOT depend on: bijux-dna-engine or runtime/container execution logic.
//! Reading order: `stages`, `params`, `metrics`, `invariants`, `banks`, then `observer`.
//! Structural layout is documented in `docs/ARCHITECTURE.md`.
#![allow(dead_code)]
#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::comparison_chain,
    clippy::expect_used,
    clippy::float_cmp,
    clippy::fn_params_excessive_bools,
    clippy::format_collect,
    clippy::format_push_string,
    clippy::if_not_else,
    clippy::large_enum_variant,
    clippy::map_unwrap_or,
    clippy::match_same_arms,
    clippy::match_single_binding,
    clippy::missing_panics_doc,
    clippy::needless_pass_by_value,
    clippy::redundant_closure_for_method_calls,
    clippy::similar_names,
    clippy::single_match_else,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    clippy::unnecessary_wraps,
    clippy::unnecessary_sort_by,
    clippy::iter_kv_map,
    clippy::cloned_instead_of_copied,
    clippy::unwrap_used
)]
mod artifacts;
pub mod banks;
mod bench;
pub mod bench_repository;
mod chunking;
pub mod contracts;
mod comparison_contract;
mod domain_adapter;
pub mod execution_support;
mod filter_policy_matrix;
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

pub use artifacts::{
    build_fastq_scientific_drift_report, FastqScientificDriftReportV1,
    ScientificDriftArtifactDeltaV1, ScientificDriftChangeKind, ScientificDriftMetricDeltaV1,
    ScientificDriftSnapshotV1, FASTQ_SCIENTIFIC_DRIFT_REPORT_SCHEMA_VERSION,
};
pub use artifacts::{
    contaminant_depletion_artifact_paths, corrected_fastq_artifact_paths,
    host_depletion_artifact_paths, merge_fastq_artifact_paths, qc_bundle_artifact_paths,
    rejected_fastq_artifact_paths, rrna_depletion_artifact_paths, singleton_fastq_artifact_path,
    trim_artifact_paths, umi_artifact_paths, validation_artifact_paths,
    ContaminantDepletionArtifactPaths, FastqTransformArtifactPaths, HostDepletionArtifactPaths,
    QcBundleArtifactPaths, RrnaDepletionArtifactPaths, ValidationArtifactPaths,
};
pub use artifacts::{
    derived_governed_qc_lineage_hash, governed_qc_contributors_from_inputs,
    governed_qc_inputs_manifest_from_inputs, GovernedQcInputsManifestV1,
    GovernedQcManifestContributorV1, GOVERNED_QC_INPUTS_MANIFEST_SCHEMA_VERSION,
};
pub use artifacts::{
    AssetVerificationEntryV1, AssetVerificationStatusV1, VerifyAssetsReportV1,
    VERIFY_ASSETS_REPORT_SCHEMA_VERSION,
};
pub use artifacts::{
    BuildContaminantDbReportV1, BuildContaminantDbSourceEntryV1,
    BUILD_CONTAMINANT_DB_REPORT_SCHEMA_VERSION,
};
pub use artifacts::{
    BuildRrnaDbReportV1, BuildRrnaDbSourceEntryV1, BUILD_RRNA_DB_REPORT_SCHEMA_VERSION,
};
pub use artifacts::{
    BuildTaxonomyDbReportV1, BuildTaxonomyDbSourceEntryV1, BUILD_TAXONOMY_DB_REPORT_SCHEMA_VERSION,
};
pub use artifacts::{
    CaptureProvenanceSnapshotReportV1, ProvenanceFileEntryV1, ProvenanceStageEntryV1,
    CAPTURE_PROVENANCE_SNAPSHOT_REPORT_SCHEMA_VERSION,
};
pub use artifacts::{
    ClassifyLayoutReportV1, FastqLayoutClassV1, CLASSIFY_LAYOUT_REPORT_SCHEMA_VERSION,
};
pub use artifacts::{ClusterOtusReportV1, CLUSTER_OTUS_REPORT_SCHEMA_VERSION};
pub use artifacts::{
    ConcatenateLaneSummaryV1, ConcatenateLanesReportV1, CONCATENATE_LANES_REPORT_SCHEMA_VERSION,
};
pub use artifacts::{CorrectErrorsReportV1, CORRECT_ERRORS_REPORT_SCHEMA_VERSION};
pub use artifacts::{DeinterleaveReadsReportV1, DEINTERLEAVE_READS_REPORT_SCHEMA_VERSION};
pub use artifacts::{
    DemultiplexReadsReportV1, DemultiplexSampleSummaryV1, DEMULTIPLEX_READS_REPORT_SCHEMA_VERSION,
};
pub use artifacts::{DepleteHostReportV1, DEPLETE_HOST_REPORT_SCHEMA_VERSION};
pub use artifacts::{
    DepleteReferenceContaminantsReportV1, DEPLETE_REFERENCE_CONTAMINANTS_REPORT_SCHEMA_VERSION,
};
pub use artifacts::{DepleteRrnaReportV1, DEPLETE_RRNA_REPORT_SCHEMA_VERSION};
pub use artifacts::{DetectAdaptersReportV1, DETECT_ADAPTERS_REPORT_SCHEMA_VERSION};
pub use artifacts::{
    DetectDuplicatesPremergeReportV1, DETECT_DUPLICATES_PREMERGE_REPORT_SCHEMA_VERSION,
};
pub use artifacts::{
    DetectInstrumentArtifactsReportV1, DETECT_INSTRUMENT_ARTIFACTS_REPORT_SCHEMA_VERSION,
};
pub use artifacts::{
    DuplicateClassEntryV1, RemoveDuplicatesProvenanceV1, RemoveDuplicatesReportV1,
    REMOVE_DUPLICATES_PROVENANCE_SCHEMA_VERSION, REMOVE_DUPLICATES_REPORT_SCHEMA_VERSION,
};
pub use artifacts::{
    EstimateLibraryComplexityPrealignReportV1,
    ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN_REPORT_SCHEMA_VERSION,
};
pub use artifacts::{ExtractUmisReportV1, EXTRACT_UMIS_REPORT_SCHEMA_VERSION};
pub use artifacts::{FilterLowComplexityReportV1, FILTER_LOW_COMPLEXITY_REPORT_SCHEMA_VERSION};
pub use artifacts::{FilterReadsReportV1, FILTER_READS_REPORT_SCHEMA_VERSION};
pub use artifacts::{GovernedQcContributorV1, ReportQcReportV1, REPORT_QC_REPORT_SCHEMA_VERSION};
pub use artifacts::{
    HostReferenceBundleFileV1, PrepareHostReferenceBundleReportV1,
    PREPARE_HOST_REFERENCE_BUNDLE_REPORT_SCHEMA_VERSION,
};
pub use artifacts::{
    IndexReferenceFileEntryV1, IndexReferenceReportV1, INDEX_REFERENCE_REPORT_SCHEMA_VERSION,
};
pub use artifacts::{InferAsvsReportV1, INFER_ASVS_REPORT_SCHEMA_VERSION};
pub use artifacts::{InterleaveReadsReportV1, INTERLEAVE_READS_REPORT_SCHEMA_VERSION};
pub use artifacts::{
    MaterializeQcManifestReportV1, QcManifestEntryV1, MATERIALIZE_QC_MANIFEST_REPORT_SCHEMA_VERSION,
};
pub use artifacts::{MergePairsReportV1, MERGE_PAIRS_REPORT_SCHEMA_VERSION};
pub use artifacts::{NormalizeAbundanceReportV1, NORMALIZE_ABUNDANCE_REPORT_SCHEMA_VERSION};
pub use artifacts::{NormalizePrimersReportV1, NORMALIZE_PRIMERS_REPORT_SCHEMA_VERSION};
pub use artifacts::{NormalizeReadNamesReportV1, NORMALIZE_READ_NAMES_REPORT_SCHEMA_VERSION};
pub use artifacts::{
    OverrepresentedSequenceRowV1, ProfileOverrepresentedReportV1,
    PROFILE_OVERREPRESENTED_REPORT_SCHEMA_VERSION,
};
pub use artifacts::{PrepareAdapterBankReportV1, PREPARE_ADAPTER_BANK_REPORT_SCHEMA_VERSION};
pub use artifacts::{PreparePrimerBankReportV1, PREPARE_PRIMER_BANK_REPORT_SCHEMA_VERSION};
pub use artifacts::{
    ProfileReadLengthBinV1, ProfileReadLengthsReportV1, PROFILE_READ_LENGTHS_REPORT_SCHEMA_VERSION,
};
pub use artifacts::{
    ProfileReadsHistogramBinV1, ProfileReadsMateSummaryV1, ProfileReadsReportV1,
    PROFILE_READS_REPORT_SCHEMA_VERSION,
};
pub use artifacts::{RemoveChimerasReportV1, REMOVE_CHIMERAS_REPORT_SCHEMA_VERSION};
pub use artifacts::{RepairPairsReportV1, REPAIR_PAIRS_REPORT_SCHEMA_VERSION};
pub use artifacts::{
    ScreenTaxonomyReportV1, TaxonomyScreenSummaryEntryV1, SCREEN_TAXONOMY_REPORT_SCHEMA_VERSION,
};
pub use artifacts::{SubsampleReadsReportV1, SUBSAMPLE_READS_REPORT_SCHEMA_VERSION};
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
    amplicon_governance_path, load_primer_bank, primer_checksums_path, primer_evidence_path,
    PrimerBankV1, PrimerSetV1,
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
pub use chunking::{
    aggregate_chunked_preprocess, chunked_and_unchunked_are_equivalent,
    ChunkedPreprocessAggregateV1, ChunkedPreprocessChunkV1, ChunkedPreprocessContractV1,
};
pub use comparison_contract::{
    benchmark_comparison_artifact_ids, comparison_artifact_ids_for_stage,
    comparison_contract_for_stage, comparison_input_artifact_ids_for_stage,
    trim_backend_comparison_contract, StageComparisonContract, TrimBackendComparisonContract,
    TrimComparisonToolProfile,
};
pub use contracts::{
    fastq_parser_fixture_bindings, fastq_parser_fixture_cases, find_fastq_parser_fixture_binding,
    find_fastq_parser_fixture_case, FastqParserFixtureBinding, FastqParserFixtureCase,
};
pub use domain_adapter::FastqDomain;
pub use execution_support::{
    admitted_tools_for_stage as admitted_execution_tools_for_stage, all_stage_execution_support,
    closed_stage_ids as execution_closed_stage_ids, comparable_benchmark_stage_ids,
    declared_only_stage_ids as execution_declared_only_stage_ids,
    default_tool_for_stage as default_execution_tool_for_stage, execution_support_for_stage,
    ExecutionStatus, StageExecutionSupport,
};
pub use filter_policy_matrix::{
    governed_filter_policy_matrix, FilterPolicyEntryV1, FilterScientificBoundary,
};
pub use id_catalog::{
    FastqInvariantsPreset, FASTQ_LOCAL_BENCH_STAGE_ID_CATALOG, FASTQ_METRICS_CATALOG,
    FASTQ_PARAMS_CATALOG, FASTQ_STAGE_ID_CATALOG,
};
pub use integration_matrix::{
    benchmark_scenarios, benchmark_scenarios_for_stage, governed_tool_ids_for_stage,
    is_reference_index_backend_compatible, planned_tool_ids_for_stage,
    reference_index_backends_for_tool, registered_tool_ids_for_stage,
    stage_sanity_metrics_for_stage, stage_tool_binding, stage_tool_bindings,
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
pub use observer::{
    evaluate_fastq_raw_parser_failure_contracts, FastqRawParserFailureClass,
    FastqRawParserFailureContractRow,
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
    default_shotgun_preprocess_stage_order, forbidden_transitions,
    non_general_genomics_branch_contract_for_stage, non_general_genomics_branch_contracts,
    optional_branches, preprocess_pipeline_graph_for_stage_order, FastqPipelineMode,
    NonGeneralGenomicsBranchContractV1, NonGeneralGenomicsBranchFamily, StageCriticality,
};
pub use qc_contract::{
    governed_qc_bench_contributor_stage_ids, governed_qc_default_tool_ids,
    governed_qc_output_ids_for_stage, governed_qc_producer_stage_ids,
};
pub use run::{assess_input_dir, discover_fastq_files};
pub use run::{
    bench_corpus, bench_corpus_manifest, required_bench_corpus_scenarios, BenchCorpus,
    BenchCorpusDatasetManifestEntryV1, BenchCorpusId, BenchCorpusManifestV1, BenchDataset,
    BenchDatasetScenario, BENCH_CORPUS_MANIFEST_SCHEMA_VERSION,
};
pub use stage_tool_governance::{
    benchmark_corpus_assignment_for_stage_tool, benchmark_readiness_for_stage_tool,
    declared_input_layouts_for_stage, filter_tools_for_input_layout, stage_accepts_input_layout,
    stage_benchmark_governance, stage_tool_capability_contract, stage_tool_governance_profile,
    stage_tool_governance_profiles_for_stage, stage_tool_maturity, tool_supports_input_layout,
    BenchmarkCorpusAssignment, BenchmarkCorpusFamily, BenchmarkReadinessLevel,
    FastqStageLayoutPolicy, RuntimeNormalizationLevel, StageBenchmarkGovernance,
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
    STAGE_DETECT_ADAPTERS, STAGE_DETECT_DUPLICATES_PREMERGE,
    STAGE_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN, STAGE_EXTRACT_UMIS, STAGE_FILTER_LOW_COMPLEXITY,
    STAGE_FILTER_READS, STAGE_INFER_ASVS, STAGE_MERGE_PAIRS, STAGE_NORMALIZE_ABUNDANCE,
    STAGE_NORMALIZE_PRIMERS, STAGE_PREFIX, STAGE_PROFILE_READS, STAGE_REMOVE_CHIMERAS,
    STAGE_REMOVE_DUPLICATES, STAGE_REPORT_QC, STAGE_SCREEN_TAXONOMY, STAGE_TRIM_READS,
    STAGE_TRIM_TERMINAL_DAMAGE, STAGE_VALIDATE_READS,
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
    FastqPE, FastqPairedEnd, FastqReadLayout, FastqSE, FastqSampleId, FastqSingleEnd, FastqStats,
    RetentionReportV1, ToolReferenceV1, FASTQ_DECLARED_READ_LAYOUTS,
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
