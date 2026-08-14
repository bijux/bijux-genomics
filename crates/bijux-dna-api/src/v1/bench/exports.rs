//! Benchmarking helpers for v1.
//!
//! Stability: v1 (stable).

pub use bijux_dna_bench::{
    all_domain_output_declarations_schema, stage_tool_assets_config_schema,
    stage_tool_assets_report_schema, stage_tool_containers_config_schema,
    stage_tool_containers_report_schema, stage_tool_resources_config_schema,
    stage_tool_resources_report_schema, validate_all_domain_output_declaration_rows,
    validate_all_domain_output_declarations_report, validate_stage_tool_assets_config,
    validate_stage_tool_assets_report, validate_stage_tool_containers_config,
    validate_stage_tool_containers_report, validate_stage_tool_resources_config,
    validate_stage_tool_resources_report, AllDomainOutputDeclarationRow,
    AllDomainOutputDeclarationStatus, AllDomainOutputDeclarationsReport, StageToolAssetRow,
    StageToolAssetsConfig, StageToolAssetsReport, StageToolContainerRow, StageToolContainersConfig,
    StageToolContainersReport, StageToolResourceRow, StageToolResourcesConfig,
    StageToolResourcesReport, ALL_DOMAIN_OUTPUT_DECLARATIONS_SCHEMA_V1,
    BAM_STAGE_TOOL_RESOURCE_ORIGIN, FASTQ_STAGE_TOOL_RESOURCE_ORIGIN,
    LOCAL_STAGE_TOOL_ASSETS_SCHEMA_V1, LOCAL_STAGE_TOOL_CONTAINERS_SCHEMA_V1,
    LOCAL_STAGE_TOOL_RESOURCES_SCHEMA_V1, STAGE_TOOL_ASSETS_DECLARATION_ORIGIN,
    STAGE_TOOL_ASSETS_REPORT_SCHEMA_V1, STAGE_TOOL_ASSETS_SCOPE,
    STAGE_TOOL_CONTAINERS_DECLARATION_ORIGIN, STAGE_TOOL_CONTAINERS_REPORT_SCHEMA_V1,
    STAGE_TOOL_CONTAINERS_SCOPE, STAGE_TOOL_RESOURCES_REPORT_SCHEMA_V1, STAGE_TOOL_RESOURCES_SCOPE,
    VCF_STAGE_TOOL_RESOURCE_ORIGIN,
};

use std::path::Path;

use anyhow::{Context, Result};

pub use bijux_dna_analyze::compare::compare_runs_with_baseline;
pub use bijux_dna_analyze::{build_rankings, compare_runs, print_bench_schema, RankInput};
pub use bijux_dna_bench::{
    acquire_fastq_corpus, admit_benchmark_cases, admit_checked_in_benchmark_campaign,
    aggregate_benchmark_cases, all_domain_failure_classification_schema,
    all_domain_parser_collector_schema, appraise_matrix_report, benchmark_dossier_index_schema,
    benchmark_essential_pipeline_readiness_schema,
    benchmark_local_all_domain_slurm_submit_manifest_schema,
    benchmark_local_all_domain_stage_inventory_schema, benchmark_local_corpus_run_schema,
    benchmark_local_corpus_skip_report_schema, benchmark_local_corpus_stage_compatibility_schema,
    benchmark_local_essential_pipeline_fake_run_node_metrics_schema,
    benchmark_local_essential_pipeline_fake_run_pipeline_schema,
    benchmark_local_essential_pipeline_fake_runs_schema, benchmark_local_hpc_asset_staging_schema,
    benchmark_local_hpc_candidate_run_manifest_schema,
    benchmark_local_hpc_dependency_simulation_schema, benchmark_local_hpc_dry_run_readiness_schema,
    benchmark_local_hpc_execution_resolver_schema, benchmark_local_hpc_job_graph_schema,
    benchmark_local_hpc_pipeline_node_array_schema,
    benchmark_local_hpc_result_collection_simulation_schema,
    benchmark_local_hpc_resume_simulation_schema, benchmark_local_hpc_scratch_layout_schema,
    benchmark_local_hpc_stage_benchmark_array_schema,
    benchmark_local_hpc_submission_readiness_schema,
    benchmark_local_pipeline_dag_validation_schema,
    benchmark_local_pipeline_dag_validation_set_schema, benchmark_local_runtime_metrics_schema,
    benchmark_local_slurm_dependency_check_schema, benchmark_local_slurm_dry_run_schema,
    benchmark_local_slurm_shell_syntax_schema, benchmark_local_slurm_submit_manifest_schema,
    benchmark_local_stage_commands_schema, benchmark_local_stage_fake_failure_manifest_schema,
    benchmark_local_stage_fake_failure_record_schema,
    benchmark_local_stage_fake_run_manifest_schema, benchmark_local_stage_inventory_schema,
    benchmark_local_stage_manifest_completion_schema,
    benchmark_local_stage_output_completion_schema, benchmark_local_summary_schema,
    benchmark_local_vcf_smoke_suite_schema, benchmark_publication_findings_schema,
    benchmark_publication_status_schema, benchmark_published_results_status_schema,
    benchmark_remediation_queue_schema, benchmark_stage_result_validation_schema,
    choose_canonical_tool_id, current_benchmark_matrix_timestamp, derive_bam_benchmark_campaign,
    derive_benchmark_local_readiness_status, derive_benchmark_matrix,
    derive_cross_benchmark_campaign, derive_fastq_benchmark_campaign,
    derive_hardening_benchmark_campaign, derive_vcf_benchmark_campaign,
    ensure_stage_campaign_attempts, hardening_queue_from_appraisal,
    legacy_benchmark_stage_alias_target, legacy_benchmark_stage_aliases, load_appraisal_report,
    load_benchmark_matrix_report, load_benchmark_stage_result_manifest,
    load_checked_in_benchmark_campaign_plan, materialize_checked_in_reference_bundle,
    materialize_fastq_corpus, normalize_tool_id, plan_benchmark_cases,
    plan_checked_in_benchmark_campaign, prepare_checked_in_benchmark_campaign,
    publish_appraisal_report, publish_bam_benchmark_campaign_report,
    publish_benchmark_campaign_report_with_presentation_authority, publish_benchmark_data_index,
    publish_benchmark_matrix, publish_checked_in_benchmark_evidence_book,
    publish_cross_benchmark_campaign_report, publish_fastq_benchmark_campaign_report,
    publish_governed_data_catalog_registry, publish_hardening_benchmark_campaign_report,
    publish_hardening_queue_report, publish_vcf_benchmark_campaign_report,
    replay_checked_in_benchmark_publication, required_migration_stage_ids,
    retry_checked_in_benchmark_campaign, stage_set_contains_canonical_or_migration_alias,
    status_checked_in_benchmark_campaign, supersede_checked_in_benchmark_campaign,
    validate_benchmark_stage_result_path, LegacyBenchmarkStageAlias,
};
use bijux_dna_bench::{
    admitted_benchmark_stage_report_schema, benchmark_attempt_archive_schema,
    benchmark_attempt_schema, benchmark_cycle_status_schema,
    benchmark_pipeline_parameter_config_schema, benchmark_stage_interface_catalog_schema,
    benchmark_stage_report_schema,
};
pub use bijux_dna_bench::{
    all_domain_active_stage_tool_matrix_schema, all_domain_expected_benchmark_results_schema,
    all_domain_retained_tools_schema, bam_comparable_metrics_schema,
    benchmark_corpus_dossier_summary_schema, build_full_benchmark_comparable_metric_rows,
    build_full_benchmark_corpus_rows, build_full_benchmark_failures_section,
    build_full_benchmark_memory_rows, build_full_benchmark_missing_result_rows,
    build_full_benchmark_pipeline_rows, build_full_benchmark_runtime_rows,
    build_full_benchmark_stage_rows, build_full_benchmark_tool_rows,
    build_full_benchmark_unsupported_pair_rows, corpus_centric_report_schema,
    fastq_comparable_metrics_schema, full_benchmark_report_schema,
    full_benchmark_result_collector_schema, load_fastq_corpus_source_spec,
    render_full_benchmark_report_markdown, scientific_acceptance_thresholds_report_schema,
    scientific_acceptance_thresholds_schema, stage_centric_report_schema,
    stage_registry_extra_pairs_schema, tool_centric_report_schema,
    validate_active_scope_fast_validation_report, validate_all_domain_active_stage_tool_matrix,
    validate_all_domain_expected_benchmark_results, validate_all_domain_failure_classification,
    validate_all_domain_parser_collector, validate_all_domain_retained_tools,
    validate_bam_comparable_metrics_report, validate_benchmark_corpus_dossier_summary,
    validate_benchmark_dossier_index, validate_benchmark_essential_pipeline_readiness,
    validate_benchmark_local_all_domain_slurm_submit_manifest,
    validate_benchmark_local_all_domain_stage_inventory, validate_benchmark_local_corpus_run,
    validate_benchmark_local_corpus_skip_report,
    validate_benchmark_local_corpus_stage_compatibility_report,
    validate_benchmark_local_essential_pipeline_fake_run_node_metrics,
    validate_benchmark_local_essential_pipeline_fake_run_node_report,
    validate_benchmark_local_essential_pipeline_fake_run_pipeline_report,
    validate_benchmark_local_essential_pipeline_fake_runs_report,
    validate_benchmark_local_hpc_asset_staging_manifest,
    validate_benchmark_local_hpc_candidate_run_manifest,
    validate_benchmark_local_hpc_dependency_simulation,
    validate_benchmark_local_hpc_dry_run_readiness,
    validate_benchmark_local_hpc_execution_resolver, validate_benchmark_local_hpc_job_graph,
    validate_benchmark_local_hpc_pipeline_node_array,
    validate_benchmark_local_hpc_result_collection_simulation_report,
    validate_benchmark_local_hpc_resume_simulation, validate_benchmark_local_hpc_scratch_layout,
    validate_benchmark_local_hpc_stage_benchmark_array,
    validate_benchmark_local_hpc_submission_readiness,
    validate_benchmark_local_pipeline_dag_validation,
    validate_benchmark_local_pipeline_dag_validation_set, validate_benchmark_local_runtime_metrics,
    validate_benchmark_local_slurm_dependency_check, validate_benchmark_local_slurm_dry_run,
    validate_benchmark_local_slurm_shell_syntax, validate_benchmark_local_slurm_submit_manifest,
    validate_benchmark_local_stage_commands, validate_benchmark_local_stage_fake_failure_manifest,
    validate_benchmark_local_stage_fake_failure_record,
    validate_benchmark_local_stage_fake_run_manifest,
    validate_benchmark_local_stage_fake_run_result, validate_benchmark_local_stage_inventory,
    validate_benchmark_local_stage_manifest_completion,
    validate_benchmark_local_stage_output_completion, validate_benchmark_local_summary,
    validate_benchmark_local_vcf_smoke_suite, validate_benchmark_publication_findings,
    validate_benchmark_publication_status, validate_benchmark_published_results_status,
    validate_benchmark_remediation_queue, validate_benchmark_stage_result_manifest,
    validate_benchmark_stage_result_validation_report, validate_corpus_centric_report,
    validate_fastq_comparable_metrics_report, validate_fastq_corpus_source_spec,
    validate_full_benchmark_report, validate_full_benchmark_result_collector,
    validate_scientific_acceptance_thresholds_config,
    validate_scientific_acceptance_thresholds_report, validate_stage_centric_report,
    validate_stage_registry_extra_pairs, validate_tool_centric_report,
    validate_vcf_comparable_metrics_report, vcf_comparable_metrics_schema,
    ActiveScopeFastValidationCheck, ActiveScopeFastValidationReport, AdmittedBenchmarkStageReport,
    AllDomainActiveStageToolMatrixReport, AllDomainActiveStageToolMatrixRow,
    AllDomainExpectedBenchmarkResultRow, AllDomainExpectedBenchmarkResultsReport,
    AllDomainFailureClassificationReport, AllDomainFailureClassificationRow,
    AllDomainParserCollectorReport, AllDomainParserCollectorRow,
    AllDomainParserCollectorSourceKind, AllDomainRetainedToolRow, AllDomainRetainedToolsReport,
    AppraisalFinding, AppraisalReport, AppraisalSummary, BamBenchmarkCampaignReport,
    BamBenchmarkCampaignSummary, BamBenchmarkGoalEntry, BamComparableMetricsReport,
    BamComparableMetricsRow, BamMatrixRowRef, BamScenario, BenchmarkAttemptArchive,
    BenchmarkAttemptRecord, BenchmarkCaseAdmissionRequest, BenchmarkCaseAggregationRequest,
    BenchmarkCasePlanningRequest, BenchmarkCorpusDossierHeadline,
    BenchmarkCorpusDossierSampleResultRow, BenchmarkCorpusDossierSummary,
    BenchmarkCorpusDossierToolSummary, BenchmarkCorpusRunManifest, BenchmarkCorpusSampleRun,
    BenchmarkCycleAdmission, BenchmarkCycleAdmissionRequest, BenchmarkCycleAdmissionState,
    BenchmarkCycleAdmissionUnit, BenchmarkCyclePlanRequest, BenchmarkCyclePreparation,
    BenchmarkCyclePreparationRequest, BenchmarkCyclePreparationState,
    BenchmarkCyclePreparationUnit, BenchmarkCycleRetryRequest, BenchmarkCycleStatus,
    BenchmarkCycleStatusRequest, BenchmarkCycleSupersedeRequest,
    BenchmarkDataIndexPublicationRequest, BenchmarkDataIndexRequest, BenchmarkDossierIndex,
    BenchmarkDossierStageEntry, BenchmarkEssentialPipelineReadinessGoalCheck,
    BenchmarkEssentialPipelineReadinessGoalContract, BenchmarkEssentialPipelineReadinessReport,
    BenchmarkEvidenceBookRequest, BenchmarkJobReference, BenchmarkLifecycleStatus,
    BenchmarkLocalAllDomainSlurmSubmitJob, BenchmarkLocalAllDomainSlurmSubmitManifest,
    BenchmarkLocalAllDomainStageInventory, BenchmarkLocalAssetBackedStage,
    BenchmarkLocalCorpusFixture, BenchmarkLocalCorpusSkip, BenchmarkLocalCorpusSkipReport,
    BenchmarkLocalCorpusStageCompatibilityEntry, BenchmarkLocalCorpusStageCompatibilityReport,
    BenchmarkLocalCorpusStageValidatedFixture, BenchmarkLocalEssentialPipelineFakeRunInputBinding,
    BenchmarkLocalEssentialPipelineFakeRunNodeMetrics,
    BenchmarkLocalEssentialPipelineFakeRunNodeReport,
    BenchmarkLocalEssentialPipelineFakeRunOutputEntry,
    BenchmarkLocalEssentialPipelineFakeRunPipelineReport,
    BenchmarkLocalEssentialPipelineFakeRunsReport, BenchmarkLocalHpcArrayResources,
    BenchmarkLocalHpcAssetStagingInput, BenchmarkLocalHpcAssetStagingJob,
    BenchmarkLocalHpcAssetStagingManifest, BenchmarkLocalHpcCandidateBehavior,
    BenchmarkLocalHpcCandidateJob, BenchmarkLocalHpcCandidateResourceCeiling,
    BenchmarkLocalHpcCandidateResourceHint, BenchmarkLocalHpcCandidateRunManifest,
    BenchmarkLocalHpcCandidateStopCondition, BenchmarkLocalHpcDependencySimulationCase,
    BenchmarkLocalHpcDependencySimulationJob, BenchmarkLocalHpcDependencySimulationReport,
    BenchmarkLocalHpcDryRunReadinessBehavior, BenchmarkLocalHpcDryRunReadinessGoalCheck,
    BenchmarkLocalHpcDryRunReadinessReport, BenchmarkLocalHpcDryRunReadinessSummary,
    BenchmarkLocalHpcExecutionResolverReport, BenchmarkLocalHpcExecutionResolverRow,
    BenchmarkLocalHpcJobGraph, BenchmarkLocalHpcJobGraphNode,
    BenchmarkLocalHpcPipelineNodeArrayDependency, BenchmarkLocalHpcPipelineNodeArrayExpectedOutput,
    BenchmarkLocalHpcPipelineNodeArrayReport, BenchmarkLocalHpcPipelineNodeArrayRow,
    BenchmarkLocalHpcResultCollectionSimulationBehavior,
    BenchmarkLocalHpcResultCollectionSimulationReport,
    BenchmarkLocalHpcResultCollectionSimulationRow, BenchmarkLocalHpcResumeSimulationBehavior,
    BenchmarkLocalHpcResumeSimulationJob, BenchmarkLocalHpcResumeSimulationReport,
    BenchmarkLocalHpcScratchCleanupPolicy, BenchmarkLocalHpcScratchInputLink,
    BenchmarkLocalHpcScratchJobResources, BenchmarkLocalHpcScratchLayout,
    BenchmarkLocalHpcScratchLayoutJob, BenchmarkLocalHpcStageBenchmarkArrayReport,
    BenchmarkLocalHpcStageBenchmarkArrayRow, BenchmarkLocalHpcSubmissionReadinessGoalCheck,
    BenchmarkLocalHpcSubmissionReadinessReport, BenchmarkLocalPipelineAncientDnaPolicy,
    BenchmarkLocalPipelineDagValidationNode, BenchmarkLocalPipelineDagValidationProfile,
    BenchmarkLocalPipelineDagValidationReport, BenchmarkLocalPipelineDagValidationSetReport,
    BenchmarkLocalPipelineExecutionContext, BenchmarkLocalPipelineProjectSource,
    BenchmarkLocalPipelineReferenceAssets, BenchmarkLocalPipelineReferenceContext,
    BenchmarkLocalPipelineSnpEffAssets, BenchmarkLocalPipelineVariantAssets,
    BenchmarkLocalPlannerOnlyStage, BenchmarkLocalReadinessStatus,
    BenchmarkLocalSlurmDependencyCheckJob, BenchmarkLocalSlurmDependencyCheckReport,
    BenchmarkLocalSlurmDryRunReport, BenchmarkLocalSlurmScriptEntry,
    BenchmarkLocalSlurmShellSyntaxEntry, BenchmarkLocalSlurmShellSyntaxReport,
    BenchmarkLocalSlurmSubmitJob, BenchmarkLocalSlurmSubmitLogs, BenchmarkLocalSlurmSubmitManifest,
    BenchmarkLocalSlurmSubmitResources, BenchmarkLocalStageArtifactEntry,
    BenchmarkLocalStageCommandArgvEntry, BenchmarkLocalStageCommandEntry,
    BenchmarkLocalStageCommandManifest, BenchmarkLocalStageFakeFailureManifest,
    BenchmarkLocalStageFakeFailureOutputEntry, BenchmarkLocalStageFakeFailureRecord,
    BenchmarkLocalStageFakeRunManifest, BenchmarkLocalStageFakeRunOutputEntry,
    BenchmarkLocalStageFakeRunResult, BenchmarkLocalStageInventory,
    BenchmarkLocalStageInventoryEntry, BenchmarkLocalStageManifestCompletionEntry,
    BenchmarkLocalStageManifestCompletionReport, BenchmarkLocalStageMissingOutputEntry,
    BenchmarkLocalStageOutputCompletionEntry, BenchmarkLocalStageOutputCompletionReport,
    BenchmarkLocalStageReadinessKind, BenchmarkLocalStageRuntimeMetric,
    BenchmarkLocalStageRuntimeMetricsReport, BenchmarkLocalSummaryReport,
    BenchmarkLocalSummaryStage, BenchmarkLocalVcfSmokeGoalCheck,
    BenchmarkLocalVcfSmokeGoalContract, BenchmarkLocalVcfSmokeSuiteReport, BenchmarkMatrixReport,
    BenchmarkMatrixRequest, BenchmarkMatrixRow, BenchmarkMatrixSummary,
    BenchmarkPipelineParameterConfig, BenchmarkPublicationAuditIssue,
    BenchmarkPublicationExcludedStage, BenchmarkPublicationFindings,
    BenchmarkPublicationStageStatus, BenchmarkPublicationStatusReport, BenchmarkReadiness,
    BenchmarkRemediationIssue, BenchmarkRemediationIssueGroup, BenchmarkRemediationQueue,
    BenchmarkRemediationStageEntry, BenchmarkReplayRequest, BenchmarkStageFamily,
    BenchmarkStageInterfaceCatalog, BenchmarkStageReport, BenchmarkStageResultCommand,
    BenchmarkStageResultManifest, BenchmarkStageResultOutput,
    BenchmarkStageResultResourceMetricSource, BenchmarkStageResultResourceMetrics,
    BenchmarkStageResultRuntime, BenchmarkStageResultStatus, BenchmarkStageResultTool,
    BenchmarkStageResultValidationReport, BenchmarkSurfaceMatch, ColdWarmContract,
    ComparableMetricContractStatus, ComparableMetricDirection, CorpusAcquisitionLayout,
    CorpusAcquisitionRequest, CorpusCentricCorpusReport, CorpusCentricReport,
    CorpusCentricStageRow, CorpusMaterializationPublication, CorpusMaterializationRequest,
    CrossBenchmarkCampaignReport, CrossBenchmarkCampaignSummary, CrossBenchmarkGoalEntry,
    CrossMatrixRowRef, CrossScenario, DecisionTraceRow, EnvironmentSnapshot, FairnessContract,
    FastqBenchmarkCampaignReport, FastqBenchmarkCampaignSummary, FastqBenchmarkGoalEntry,
    FastqComparableMetricsReport, FastqComparableMetricsRow, FastqCorpusSourceSample,
    FastqCorpusSourceSpec, FastqMatrixRowRef, FastqScenario, FullBenchmarkComparableMetricRow,
    FullBenchmarkCorpusSectionRow, FullBenchmarkFailureRow, FullBenchmarkFailuresSection,
    FullBenchmarkMemoryRow, FullBenchmarkMissingResultRow, FullBenchmarkPipelineSection,
    FullBenchmarkPipelineSectionRow, FullBenchmarkReport, FullBenchmarkReportAdmissionContext,
    FullBenchmarkReportRow, FullBenchmarkReportRowStatus, FullBenchmarkResultBinding,
    FullBenchmarkResultCollectorAdmissionContext, FullBenchmarkResultCollectorReport,
    FullBenchmarkResultCollectorRow, FullBenchmarkResultStatus, FullBenchmarkResultSurfaceKind,
    FullBenchmarkRuntimeRow, FullBenchmarkStageSectionRow, FullBenchmarkToolSectionRow,
    FullBenchmarkUnsupportedPairRow, GovernedDataCatalogRegistryPublication,
    GovernedDataCatalogRegistryPublicationRequest, HardeningBenchmarkCampaignReport,
    HardeningBenchmarkCampaignSummary, HardeningBenchmarkGoalEntry, HardeningMatrixRowRef,
    HardeningQueueEntry, HardeningQueueReport, HardeningScenario, MetricsArtifactRow,
    PostconditionRow, PublishedResultsStageReport, PublishedResultsStatusReport,
    ReadinessComparisonContractStatus, ReferenceBundleMaterializationRequest, RunRecordRow,
    ScientificAcceptanceDirection, ScientificAcceptanceInsufficiencyBehavior,
    ScientificAcceptancePassRule, ScientificAcceptanceThresholdRow,
    ScientificAcceptanceThresholdsConfig, ScientificAcceptanceThresholdsReport,
    ScientificAcceptanceToleranceKind, ScientificDefaultsStatus, StageCentricReport,
    StageCentricStageReport, StageCentricToolRow, StageRegistryExtraPairRow,
    StageRegistryExtraPairsReport, StageResultIssue, SuiteRunManifest, ToolCentricReport,
    ToolCentricStageRow, ToolCentricToolReport, ToolInvocationRow, VcfBenchmarkCampaignReport,
    VcfBenchmarkCampaignSummary, VcfBenchmarkGoalEntry, VcfComparableMetricsReport,
    VcfComparableMetricsRow, VcfMatrixRowRef, VcfScenario, ACTIVE_SCOPE_FAST_VALIDATION_SCHEMA_V1,
    ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_SCHEMA_V1, ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_SCHEMA_V1,
    ALL_DOMAIN_FAILURE_CLASSIFICATION_SCHEMA_V1, ALL_DOMAIN_PARSER_COLLECTOR_FAKE_RUN_ROOT,
    ALL_DOMAIN_PARSER_COLLECTOR_FIXTURE_ROOT, ALL_DOMAIN_PARSER_COLLECTOR_OUTPUT_PATH,
    ALL_DOMAIN_PARSER_COLLECTOR_SCHEMA_V1, ALL_DOMAIN_RETAINED_TOOLS_SCHEMA_V1,
    BAM_COMPARABLE_METRICS_SCHEMA_V1, BAM_STAGE_FAMILIES, BENCHMARK_ATTEMPT_ARCHIVE_SCHEMA_V1,
    BENCHMARK_ATTEMPT_SCHEMA_V1, BENCHMARK_CORPUS_DOSSIER_SUMMARY_SCHEMA_V1,
    BENCHMARK_CYCLE_STATUS_SCHEMA_V1, BENCHMARK_DOSSIER_INDEX_SCHEMA_V1,
    BENCHMARK_ESSENTIAL_PIPELINE_READINESS_GOALS,
    BENCHMARK_ESSENTIAL_PIPELINE_READINESS_OUTPUT_PATH,
    BENCHMARK_ESSENTIAL_PIPELINE_READINESS_SCHEMA_V1,
    BENCHMARK_LOCAL_ALL_DOMAIN_SLURM_SUBMIT_MANIFEST_SCHEMA_V1,
    BENCHMARK_LOCAL_ALL_DOMAIN_STAGE_INVENTORY_SCHEMA_V1, BENCHMARK_LOCAL_CORPUS_RUN_SCHEMA_V1,
    BENCHMARK_LOCAL_CORPUS_SKIP_REPORT_SCHEMA_V1,
    BENCHMARK_LOCAL_CORPUS_STAGE_COMPATIBILITY_SCHEMA_V1,
    BENCHMARK_LOCAL_ESSENTIAL_PIPELINE_FAKE_RUNS_SCHEMA_V1,
    BENCHMARK_LOCAL_ESSENTIAL_PIPELINE_FAKE_RUN_NODE_METRICS_SCHEMA_V1,
    BENCHMARK_LOCAL_ESSENTIAL_PIPELINE_FAKE_RUN_PIPELINE_SCHEMA_V1,
    BENCHMARK_LOCAL_HPC_ASSET_STAGING_SCHEMA_V1, BENCHMARK_LOCAL_HPC_CANDIDATE_CPUS_PER_TASK_MAX,
    BENCHMARK_LOCAL_HPC_CANDIDATE_MEMORY_MB_MAX, BENCHMARK_LOCAL_HPC_CANDIDATE_PROFILE_ID,
    BENCHMARK_LOCAL_HPC_CANDIDATE_RUN_MANIFEST_SCHEMA_V1,
    BENCHMARK_LOCAL_HPC_CANDIDATE_SCRATCH_GB_MAX,
    BENCHMARK_LOCAL_HPC_CANDIDATE_STAGED_INPUT_BYTES_MAX,
    BENCHMARK_LOCAL_HPC_CANDIDATE_WALLTIME_MINUTES_MAX,
    BENCHMARK_LOCAL_HPC_DEPENDENCY_SIMULATION_SCHEMA_V1,
    BENCHMARK_LOCAL_HPC_DRY_RUN_READINESS_SCHEMA_V1,
    BENCHMARK_LOCAL_HPC_EXECUTION_RESOLVER_SCHEMA_V1, BENCHMARK_LOCAL_HPC_JOB_GRAPH_SCHEMA_V1,
    BENCHMARK_LOCAL_HPC_PIPELINE_NODE_ARRAY_SCHEMA_V1,
    BENCHMARK_LOCAL_HPC_REFERENCE_BLOCKED_JOB_ID, BENCHMARK_LOCAL_HPC_REFERENCE_CONTINUED_JOB_ID,
    BENCHMARK_LOCAL_HPC_REFERENCE_FAILURE_JOB_ID, BENCHMARK_LOCAL_HPC_RELATEDNESS_BLOCKED_JOB_ID,
    BENCHMARK_LOCAL_HPC_RELATEDNESS_CONTINUED_JOB_ID,
    BENCHMARK_LOCAL_HPC_RELATEDNESS_FAILURE_JOB_ID,
    BENCHMARK_LOCAL_HPC_RESULT_COLLECTION_SIMULATION_SCHEMA_V1,
    BENCHMARK_LOCAL_HPC_RESUME_DEPENDENCY_JOB_ID, BENCHMARK_LOCAL_HPC_RESUME_FAILED_JOB_ID,
    BENCHMARK_LOCAL_HPC_RESUME_MISSING_MANIFEST_JOB_ID,
    BENCHMARK_LOCAL_HPC_RESUME_SIMULATION_SCHEMA_V1,
    BENCHMARK_LOCAL_HPC_RESUME_STALE_OUTPUT_JOB_ID, BENCHMARK_LOCAL_HPC_RESUME_UNRELATED_JOB_ID,
    BENCHMARK_LOCAL_HPC_RESUME_VALID_JOB_ID, BENCHMARK_LOCAL_HPC_SCRATCH_LAYOUT_SCHEMA_V1,
    BENCHMARK_LOCAL_HPC_STAGE_BENCHMARK_ARRAY_SCHEMA_V1,
    BENCHMARK_LOCAL_HPC_SUBMISSION_READINESS_SCHEMA_V1,
    BENCHMARK_LOCAL_PIPELINE_DAG_VALIDATION_SCHEMA_V1,
    BENCHMARK_LOCAL_PIPELINE_DAG_VALIDATION_SET_SCHEMA_V1,
    BENCHMARK_LOCAL_RUNTIME_METRICS_SCHEMA_V1, BENCHMARK_LOCAL_SLURM_DEPENDENCY_CHECK_SCHEMA_V1,
    BENCHMARK_LOCAL_SLURM_DRY_RUN_SCHEMA_V1, BENCHMARK_LOCAL_SLURM_SHELL_SYNTAX_SCHEMA_V1,
    BENCHMARK_LOCAL_SLURM_SUBMIT_MANIFEST_SCHEMA_V1, BENCHMARK_LOCAL_STAGE_COMMANDS_SCHEMA_V3,
    BENCHMARK_LOCAL_STAGE_FAKE_FAILURE_MANIFEST_SCHEMA_V1,
    BENCHMARK_LOCAL_STAGE_FAKE_FAILURE_RECORD_SCHEMA_V1,
    BENCHMARK_LOCAL_STAGE_FAKE_RUN_MANIFEST_SCHEMA_V1, BENCHMARK_LOCAL_STAGE_INVENTORY_SCHEMA_V1,
    BENCHMARK_LOCAL_STAGE_MANIFEST_COMPLETION_SCHEMA_V1,
    BENCHMARK_LOCAL_STAGE_OUTPUT_COMPLETION_SCHEMA_V1, BENCHMARK_LOCAL_SUMMARY_SCHEMA_V1,
    BENCHMARK_LOCAL_VCF_SMOKE_GOALS, BENCHMARK_LOCAL_VCF_SMOKE_OUTPUT_PATH,
    BENCHMARK_LOCAL_VCF_SMOKE_SUITE_SCHEMA_V1, BENCHMARK_MATRIX_REPORT_SCHEMA_V1,
    BENCHMARK_PIPELINE_PARAMETERS_SCHEMA_V1, BENCHMARK_PUBLICATION_FINDINGS_SCHEMA_V1,
    BENCHMARK_PUBLICATION_STATUS_SCHEMA_V1, BENCHMARK_PUBLISHED_RESULTS_STATUS_SCHEMA_V1,
    BENCHMARK_REMEDIATION_QUEUE_SCHEMA_V1, BENCHMARK_REPORT_ADMISSION_SCHEMA_V1,
    BENCHMARK_STAGE_INTERFACES_SCHEMA_V1, BENCHMARK_STAGE_REPORT_SCHEMA_V1,
    BENCHMARK_STAGE_RESULT_SCHEMA_V2, BENCHMARK_STAGE_RESULT_VALIDATION_SCHEMA_V1,
    BENCHMARK_SUITE_RUN_MANIFEST_SCHEMA_V2, CORPUS_CENTRIC_REPORT_SCHEMA_V1,
    FASTQ_COMPARABLE_METRICS_SCHEMA_V1, FASTQ_STAGE_FAMILIES, FULL_BENCHMARK_REPORT_SCHEMA_V1,
    FULL_BENCHMARK_RESULT_COLLECTOR_SCHEMA_V1, REQUIRED_ALL_DOMAIN_FAILURE_CLASSES,
    SCIENTIFIC_ACCEPTANCE_THRESHOLDS_REPORT_SCHEMA_V1, SCIENTIFIC_ACCEPTANCE_THRESHOLDS_SCHEMA_V1,
    STAGE_CENTRIC_REPORT_SCHEMA_V1, STAGE_REGISTRY_EXTRA_PAIRS_SCHEMA_V1,
    TOOL_CENTRIC_REPORT_SCHEMA_V1, VCF_COMPARABLE_METRICS_SCHEMA_V1, VCF_STAGE_FAMILIES,
};
pub use bijux_dna_bench::{
    derive_supported_domain_stage_tool_pairs, publish_checked_in_benchmark_run_registry,
    BenchmarkRunRegistryWriteMode, BenchmarkStageToolClosureRow,
};
pub use bijux_dna_runner::api::{
    cancel_slurm_jobs, collect_benchmark_campaign_cycle, collect_corpus_materialization_slurm_job,
    load_corpus_materialization_slurm_job_handle, load_corpus_materialization_slurm_spec,
    load_prepared_corpus_materialization_slurm_job, observe_corpus_materialization_slurm_job,
    prepare_corpus_materialization_slurm_job, publish_corpus_materialization_evidence_correction,
    submit_benchmark_campaign_frontier, submit_prepared_corpus_materialization_job,
    BenchmarkCampaignSubmission, BenchmarkCampaignSubmissionState, BenchmarkCampaignSubmissionUnit,
    CorpusMaterializationCollection, CorpusMaterializationEvidence,
    CorpusMaterializationEvidenceCorrection, CorpusMaterializationEvidenceCorrectionRequest,
    CorpusMaterializationEvidenceStatus, CorpusMaterializationSlurmResources,
    CorpusMaterializationSlurmSpec, PreparedCorpusMaterializationSlurmJob, SlurmCancelReport,
    SlurmCancelRequest, SlurmClientPaths, SlurmJobHandle, SlurmJobObservation,
    CORPUS_MATERIALIZATION_EVIDENCE_CORRECTION_SCHEMA_V1,
    CORPUS_MATERIALIZATION_EVIDENCE_SCHEMA_V1, CORPUS_MATERIALIZATION_PREPARED_JOB_SCHEMA_V1,
    CORPUS_MATERIALIZATION_SLURM_SPEC_SCHEMA_V1,
};

pub use crate::internal::public_surface::handlers::bam::BamBenchOutcome;
pub use crate::internal::public_surface::handlers::bam::{bench_bam_pipeline, bench_bam_stage};
pub use crate::surface::request_contracts::{BamRunArgs, BenchBamPipelineArgs, BenchBamStageArgs};
pub use bijux_dna_core::contract::objective_spec;
pub use bijux_dna_core::contract::{Objective, ObjectiveSpec, ObjectiveWeights};
pub use bijux_dna_planner_bam::stage_api::{bam_stage_completeness, BamStage};
pub use bijux_dna_planner_fastq::stage_api as fastq_banks;
pub use bijux_dna_planner_fastq::stage_api::args as fastq_args;
pub use bijux_dna_planner_fastq::stage_api::banks as fastq_bank_ops;
pub use bijux_dna_planner_fastq::stage_api::*;

/// Stability: v1 (stable).
/// Alias for `BenchOutcome<M: StageMetricSchema>` from the fastq handlers.
pub type BenchOutcome<M> = crate::internal::public_surface::handlers::fastq::BenchOutcome<M>;
pub use crate::internal::public_surface::handlers::fastq::*;

fn render_owned_schema(schema: schemars::Schema) -> serde_json::Value {
    match serde_json::to_value(schema) {
        Ok(value) => value,
        Err(error) => panic!("serialize JSON schema: {error}"),
    }
}

/// Render the governed benchmark attempt schema.
#[must_use]
pub fn render_benchmark_attempt_schema() -> serde_json::Value {
    render_owned_schema(benchmark_attempt_schema())
}

/// Write the governed benchmark attempt schema to disk.
///
/// # Errors
/// Returns an error if the schema cannot be written to `path`.
pub fn write_benchmark_attempt_schema(path: &Path) -> Result<()> {
    let schema = render_benchmark_attempt_schema();
    bijux_dna_infra::atomic_write_json(path, &schema)
        .with_context(|| format!("write {}", path.display()))
}

/// Render the governed benchmark attempt-archive schema.
#[must_use]
pub fn render_benchmark_attempt_archive_schema() -> serde_json::Value {
    render_owned_schema(benchmark_attempt_archive_schema())
}

/// Write the governed benchmark attempt-archive schema to disk.
///
/// # Errors
/// Returns an error if the schema cannot be written to `path`.
pub fn write_benchmark_attempt_archive_schema(path: &Path) -> Result<()> {
    let schema = render_benchmark_attempt_archive_schema();
    bijux_dna_infra::atomic_write_json(path, &schema)
        .with_context(|| format!("write {}", path.display()))
}

/// Render the governed benchmark cycle status schema.
#[must_use]
pub fn render_benchmark_cycle_status_schema() -> serde_json::Value {
    render_owned_schema(benchmark_cycle_status_schema())
}

/// Write the governed benchmark cycle status schema to disk.
///
/// # Errors
/// Returns an error if the schema cannot be written to `path`.
pub fn write_benchmark_cycle_status_schema(path: &Path) -> Result<()> {
    let schema = render_benchmark_cycle_status_schema();
    bijux_dna_infra::atomic_write_json(path, &schema)
        .with_context(|| format!("write {}", path.display()))
}

/// Render the governed benchmark stage-interface catalog schema.
#[must_use]
pub fn render_benchmark_stage_interfaces_schema() -> serde_json::Value {
    render_owned_schema(benchmark_stage_interface_catalog_schema())
}

/// Write the governed benchmark stage-interface catalog schema to disk.
///
/// # Errors
/// Returns an error if the schema cannot be written to `path`.
pub fn write_benchmark_stage_interfaces_schema(path: &Path) -> Result<()> {
    let schema = render_benchmark_stage_interfaces_schema();
    bijux_dna_infra::atomic_write_json(path, &schema)
        .with_context(|| format!("write {}", path.display()))
}

/// Render the governed benchmark pipeline-parameter schema.
#[must_use]
pub fn render_benchmark_pipeline_parameters_schema() -> serde_json::Value {
    render_owned_schema(benchmark_pipeline_parameter_config_schema())
}

/// Write the governed benchmark pipeline-parameter schema to disk.
///
/// # Errors
/// Returns an error if the schema cannot be written to `path`.
pub fn write_benchmark_pipeline_parameters_schema(path: &Path) -> Result<()> {
    let schema = render_benchmark_pipeline_parameters_schema();
    bijux_dna_infra::atomic_write_json(path, &schema)
        .with_context(|| format!("write {}", path.display()))
}

/// Render the governed benchmark stage-report schema.
#[must_use]
pub fn render_benchmark_stage_report_schema() -> serde_json::Value {
    render_owned_schema(benchmark_stage_report_schema())
}

/// Write the governed benchmark stage-report schema to disk.
///
/// # Errors
/// Returns an error if the schema cannot be written to `path`.
pub fn write_benchmark_stage_report_schema(path: &Path) -> Result<()> {
    let schema = render_benchmark_stage_report_schema();
    bijux_dna_infra::atomic_write_json(path, &schema)
        .with_context(|| format!("write {}", path.display()))
}

/// Render the governed benchmark report-admission schema.
#[must_use]
pub fn render_benchmark_report_admission_schema() -> serde_json::Value {
    render_owned_schema(admitted_benchmark_stage_report_schema())
}

/// Write the governed benchmark report-admission schema to disk.
///
/// # Errors
/// Returns an error if the schema cannot be written to `path`.
pub fn write_benchmark_report_admission_schema(path: &Path) -> Result<()> {
    let schema = render_benchmark_report_admission_schema();
    bijux_dna_infra::atomic_write_json(path, &schema)
        .with_context(|| format!("write {}", path.display()))
}

/// Render the governed FASTQ normalized metrics schema.
#[must_use]
pub fn render_fastq_normalized_metrics_schema() -> serde_json::Value {
    crate::internal::fastq::stages::preprocess::render_governed_fastq_normalized_metrics_schema()
}

/// Write the governed FASTQ normalized metrics schema to disk.
///
/// # Errors
/// Returns an error if the schema cannot be written to `path`.
pub fn write_fastq_normalized_metrics_schema(path: &Path) -> Result<()> {
    let schema = render_fastq_normalized_metrics_schema();
    bijux_dna_infra::atomic_write_json(path, &schema)
        .with_context(|| format!("write {}", path.display()))
}

/// Render the governed BAM normalized metrics schema.
#[must_use]
pub fn render_bam_normalized_metrics_schema() -> serde_json::Value {
    bijux_dna_bench_model::normalized_result::bam_envelope::render_bam_normalized_metrics_schema()
}

/// Write the governed BAM normalized metrics schema to disk.
///
/// # Errors
/// Returns an error if the schema cannot be written to `path`.
pub fn write_bam_normalized_metrics_schema(path: &Path) -> Result<()> {
    let schema = render_bam_normalized_metrics_schema();
    bijux_dna_infra::atomic_write_json(path, &schema)
        .with_context(|| format!("write {}", path.display()))
}

/// Stability: v1 (stable).
/// Descriptor for one governed stage-specific VCF normalized metrics schema.
pub use bijux_dna_bench_model::normalized_result::vcf_envelope::VcfNormalizedMetricsStageSchemaDescriptor;

/// Render the governed VCF normalized metrics schema.
///
/// # Errors
/// Returns an error if the governed schema cannot be rendered.
pub fn render_vcf_normalized_metrics_schema() -> Result<serde_json::Value> {
    bijux_dna_bench_model::normalized_result::vcf_envelope::render_vcf_normalized_metrics_schema()
}

/// Render the governed stage-specific VCF normalized metrics schema.
///
/// # Errors
/// Returns an error if the stage-specific schema cannot be rendered.
pub fn render_vcf_normalized_metrics_stage_schema(stage_id: &str) -> Result<serde_json::Value> {
    bijux_dna_bench_model::normalized_result::vcf_envelope::render_vcf_normalized_metrics_stage_schema(stage_id)
}

/// Validate a normalized VCF metrics payload against the governed stage schema contract.
///
/// # Errors
/// Returns an error if the payload does not satisfy the governed schema contract.
pub fn validate_vcf_normalized_metrics(
    metrics: &serde_json::Value,
) -> Result<VcfNormalizedMetricsStageSchemaDescriptor> {
    bijux_dna_bench_model::normalized_result::vcf_envelope::validate_vcf_normalized_metrics(metrics)
}

/// Describe the governed stage-specific VCF normalized metrics schema set.
///
/// # Errors
/// Returns an error if the governed stage schema descriptors cannot be enumerated.
pub fn vcf_normalized_metrics_stage_schema_descriptors(
) -> Result<Vec<VcfNormalizedMetricsStageSchemaDescriptor>> {
    bijux_dna_bench_model::normalized_result::vcf_envelope::vcf_normalized_metrics_stage_descriptors(
    )
}

/// Write the governed VCF normalized metrics schema to disk.
///
/// # Errors
/// Returns an error if the schema cannot be written to `path`.
pub fn write_vcf_normalized_metrics_schema(path: &Path) -> Result<()> {
    let schema = render_vcf_normalized_metrics_schema()?;
    bijux_dna_infra::atomic_write_json(path, &schema)
        .with_context(|| format!("write {}", path.display()))
}

/// Write the governed stage-specific VCF normalized metrics schema set to disk.
///
/// # Errors
/// Returns an error if the directory cannot be prepared or any schema cannot be written.
pub fn write_vcf_normalized_metrics_stage_schemas(
    stage_dir: &Path,
) -> Result<Vec<VcfNormalizedMetricsStageSchemaDescriptor>> {
    std::fs::create_dir_all(stage_dir)
        .with_context(|| format!("create {}", stage_dir.display()))?;
    let descriptors = vcf_normalized_metrics_stage_schema_descriptors()?;
    for descriptor in &descriptors {
        let schema = render_vcf_normalized_metrics_stage_schema(&descriptor.stage_id)?;
        let path = stage_dir.join(&descriptor.file_name);
        bijux_dna_infra::atomic_write_json(&path, &schema)
            .with_context(|| format!("write {}", path.display()))?;
    }
    Ok(descriptors)
}
