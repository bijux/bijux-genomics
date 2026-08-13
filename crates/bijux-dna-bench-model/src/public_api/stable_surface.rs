//! Owner: bijux-dna-bench-model
//! Stable crate-root re-exports.

pub use crate::compare;
pub use crate::contract;
pub use crate::contract::{
    append_attempt_transition, benchmark_attempt_identity_sha256, benchmark_run_definition_sha256,
    benchmark_run_directory_name, migrate_benchmark_attempt_v1, parse_benchmark_attempt_document,
    parse_benchmark_cycle_document, validate_active_scope_fast_validation_report,
    validate_all_domain_active_stage_tool_matrix, validate_all_domain_parser_collector,
    validate_all_domain_retained_tools, validate_appraisal_report, validate_attempt_archive,
    validate_attempt_identity, validate_attempt_record_v1, validate_attempt_record_v2,
    validate_attempt_transition, validate_bam_benchmark_campaign_report,
    validate_bam_comparable_metrics_report, validate_benchmark_campaign_report_publication,
    validate_benchmark_case_definition, validate_benchmark_citation_record,
    validate_benchmark_corpus_dossier_summary, validate_benchmark_data_index,
    validate_benchmark_data_portfolio_audit, validate_benchmark_data_source_spec,
    validate_benchmark_dossier_index, validate_benchmark_essential_pipeline_readiness,
    validate_benchmark_evidence_book, validate_benchmark_local_all_domain_slurm_submit_manifest,
    validate_benchmark_local_all_domain_stage_inventory, validate_benchmark_local_corpus_run,
    validate_benchmark_local_corpus_skip_report,
    validate_benchmark_local_corpus_stage_compatibility_report,
    validate_benchmark_local_hpc_asset_staging_manifest,
    validate_benchmark_local_hpc_candidate_run_manifest,
    validate_benchmark_local_hpc_dependency_simulation,
    validate_benchmark_local_hpc_execution_resolver, validate_benchmark_local_hpc_job_graph,
    validate_benchmark_local_hpc_result_collection_simulation_report,
    validate_benchmark_local_hpc_resume_simulation, validate_benchmark_local_hpc_scratch_layout,
    validate_benchmark_local_pipeline_dag_validation,
    validate_benchmark_local_pipeline_dag_validation_set, validate_benchmark_local_runtime_metrics,
    validate_benchmark_local_slurm_dependency_check, validate_benchmark_local_slurm_dry_run,
    validate_benchmark_local_slurm_shell_syntax, validate_benchmark_local_slurm_submit_manifest,
    validate_benchmark_local_stage_commands, validate_benchmark_local_stage_inventory,
    validate_benchmark_local_stage_manifest_completion,
    validate_benchmark_local_stage_output_completion, validate_benchmark_local_summary,
    validate_benchmark_local_vcf_smoke_suite, validate_benchmark_matrix_report,
    validate_benchmark_protocol_citation, validate_benchmark_publication_findings,
    validate_benchmark_publication_receipt, validate_benchmark_publication_registry,
    validate_benchmark_publication_status, validate_benchmark_published_results_status,
    validate_benchmark_remediation_queue, validate_benchmark_replay_receipt,
    validate_benchmark_run_case_index, validate_benchmark_run_definition,
    validate_benchmark_run_definition_revision, validate_benchmark_run_index,
    validate_benchmark_run_publication_pointer, validate_benchmark_run_registry,
    validate_benchmark_stage_result_manifest, validate_benchmark_stage_result_validation_report,
    validate_benchmark_stage_tool_closure, validate_campaign_plan,
    validate_container_report_portfolio_index, validate_corpus_acquisition_plan,
    validate_corpus_centric_report, validate_cross_benchmark_campaign_report,
    validate_cycle_admission, validate_cycle_preparation, validate_cycle_status_v1,
    validate_cycle_status_v2, validate_data_fitness_portfolio_alignment,
    validate_data_fitness_rubric, validate_execution_spec,
    validate_fastq_benchmark_campaign_report, validate_fastq_comparable_metrics_report,
    validate_fastq_corpus_source_spec, validate_fastq_read_length_expectations,
    validate_governed_data_catalog, validate_governed_data_catalog_document,
    validate_governed_data_catalog_registry, validate_governed_data_catalog_registry_publication,
    validate_governed_data_catalog_snapshot_publication, validate_governed_preserved_data_catalog,
    validate_hardening_benchmark_campaign_report, validate_hardening_queue_report,
    validate_scheduler_observation, validate_scientific_readiness, validate_stage_centric_report,
    validate_stage_registry_extra_pairs, validate_stage_universe, validate_tool_centric_report,
    validate_utc_timestamp, validate_vcf_benchmark_campaign_report,
    validate_vcf_comparable_metrics_report, ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_SCHEMA_V1,
    ALL_DOMAIN_PARSER_COLLECTOR_SCHEMA_V1, ALL_DOMAIN_RETAINED_TOOLS_SCHEMA_V1,
    BAM_COMPARABLE_METRICS_SCHEMA_V1, BENCHMARK_ADMITTED_CASE_OBSERVATION_SCHEMA_V1,
    BENCHMARK_ADMITTED_CASE_OBSERVATION_SCHEMA_V2,
    BENCHMARK_ADMITTED_CASE_OBSERVATION_SET_SCHEMA_V1, BENCHMARK_ATTEMPT_ARCHIVE_SCHEMA_V1,
    BENCHMARK_ATTEMPT_MIGRATION_LEDGER_SCHEMA_V1, BENCHMARK_ATTEMPT_MIGRATION_PLAN_SCHEMA_V1,
    BENCHMARK_ATTEMPT_MIGRATION_VERIFICATION_SCHEMA_V1, BENCHMARK_ATTEMPT_SCHEMA_V1,
    BENCHMARK_ATTEMPT_SCHEMA_V2, BENCHMARK_CAMPAIGN_PLAN_SCHEMA_V1,
    BENCHMARK_CAMPAIGN_REPORT_PUBLICATION_SCHEMA_V1, BENCHMARK_CASE_AGGREGATION_SCHEMA_V1,
    BENCHMARK_CASE_AGGREGATION_SCHEMA_V2, BENCHMARK_CASE_AGGREGATION_SET_SCHEMA_V1,
    BENCHMARK_CASE_DEFINITION_SCHEMA_V1, BENCHMARK_CASE_MATRIX_SCHEMA_V1,
    BENCHMARK_CASE_MATRIX_SCHEMA_V2, BENCHMARK_CITATION_RECORD_SCHEMA_V1,
    BENCHMARK_CORPUS_MATERIALIZATION_PUBLICATION_SCHEMA_V1, BENCHMARK_CYCLE_ADMISSION_SCHEMA_V1,
    BENCHMARK_CYCLE_PREPARATION_SCHEMA_V1, BENCHMARK_CYCLE_STATUS_SCHEMA_V1,
    BENCHMARK_CYCLE_STATUS_SCHEMA_V2, BENCHMARK_DATA_FITNESS_SCHEMA_V1,
    BENCHMARK_DATA_INDEX_SCHEMA_V1, BENCHMARK_DATA_INDEX_SCHEMA_V2,
    BENCHMARK_DATA_PORTFOLIO_AUDIT_SCHEMA_V1, BENCHMARK_DATA_PORTFOLIO_AUDIT_SCHEMA_V2,
    BENCHMARK_DATA_SOURCE_SCHEMA_V1, BENCHMARK_EFFECTIVE_PARAMETERS_SCHEMA_V1,
    BENCHMARK_ESSENTIAL_PIPELINE_READINESS_SCHEMA_V1, BENCHMARK_EVIDENCE_BOOK_SCHEMA_V1,
    BENCHMARK_EXECUTION_ADMISSION_SCHEMA_V1, BENCHMARK_EXECUTION_SPEC_SCHEMA_V1,
    BENCHMARK_LOCAL_ALL_DOMAIN_SLURM_SUBMIT_MANIFEST_SCHEMA_V1,
    BENCHMARK_LOCAL_ALL_DOMAIN_STAGE_INVENTORY_SCHEMA_V1, BENCHMARK_LOCAL_CORPUS_RUN_SCHEMA_V1,
    BENCHMARK_LOCAL_CORPUS_SKIP_REPORT_SCHEMA_V1,
    BENCHMARK_LOCAL_CORPUS_STAGE_COMPATIBILITY_SCHEMA_V1,
    BENCHMARK_LOCAL_HPC_ASSET_STAGING_SCHEMA_V1,
    BENCHMARK_LOCAL_HPC_CANDIDATE_RUN_MANIFEST_SCHEMA_V1,
    BENCHMARK_LOCAL_HPC_DEPENDENCY_SIMULATION_SCHEMA_V1,
    BENCHMARK_LOCAL_HPC_EXECUTION_RESOLVER_SCHEMA_V1, BENCHMARK_LOCAL_HPC_JOB_GRAPH_SCHEMA_V1,
    BENCHMARK_LOCAL_HPC_RESULT_COLLECTION_SIMULATION_SCHEMA_V1,
    BENCHMARK_LOCAL_HPC_RESUME_SIMULATION_SCHEMA_V1, BENCHMARK_LOCAL_HPC_SCRATCH_LAYOUT_SCHEMA_V1,
    BENCHMARK_LOCAL_PIPELINE_DAG_VALIDATION_SCHEMA_V1,
    BENCHMARK_LOCAL_PIPELINE_DAG_VALIDATION_SET_SCHEMA_V1,
    BENCHMARK_LOCAL_RUNTIME_METRICS_SCHEMA_V1, BENCHMARK_LOCAL_SLURM_DEPENDENCY_CHECK_SCHEMA_V1,
    BENCHMARK_LOCAL_SLURM_DRY_RUN_SCHEMA_V1, BENCHMARK_LOCAL_SLURM_SHELL_SYNTAX_SCHEMA_V1,
    BENCHMARK_LOCAL_SLURM_SUBMIT_MANIFEST_SCHEMA_V1, BENCHMARK_LOCAL_STAGE_COMMANDS_SCHEMA_V3,
    BENCHMARK_LOCAL_STAGE_INVENTORY_SCHEMA_V1, BENCHMARK_LOCAL_STAGE_MANIFEST_COMPLETION_SCHEMA_V1,
    BENCHMARK_LOCAL_STAGE_OUTPUT_COMPLETION_SCHEMA_V1, BENCHMARK_LOCAL_SUMMARY_SCHEMA_V1,
    BENCHMARK_LOCAL_VCF_SMOKE_SUITE_SCHEMA_V1, BENCHMARK_PIPELINE_PARAMETERS_SCHEMA_V1,
    BENCHMARK_PORTFOLIO_SCHEMA_V1, BENCHMARK_PORTFOLIO_SCHEMA_V2,
    BENCHMARK_PROTOCOL_CITATION_SCHEMA_V1, BENCHMARK_PUBLICATION_RECEIPT_SCHEMA_V1,
    BENCHMARK_PUBLICATION_REGISTRY_SCHEMA_V1, BENCHMARK_REPLAY_RECEIPT_SCHEMA_V1,
    BENCHMARK_REPORT_ADMISSION_SCHEMA_V1, BENCHMARK_RUN_CASE_INDEX_SCHEMA_V1,
    BENCHMARK_RUN_DEFINITION_SCHEMA_V1, BENCHMARK_RUN_INDEX_SCHEMA_V1,
    BENCHMARK_RUN_PUBLICATION_POINTER_SCHEMA_V1, BENCHMARK_RUN_REGISTRY_SCHEMA_V1,
    BENCHMARK_SCHEDULER_OBSERVATION_SCHEMA_V1, BENCHMARK_SCIENTIFIC_READINESS_SCHEMA_V1,
    BENCHMARK_SCIENTIFIC_READINESS_SCHEMA_V2, BENCHMARK_STAGE_CAMPAIGNS_SCHEMA_V1,
    BENCHMARK_STAGE_COVERAGE_SCHEMA_V1, BENCHMARK_STAGE_INTERFACES_SCHEMA_V1,
    BENCHMARK_STAGE_REPORT_SCHEMA_V1, BENCHMARK_STAGE_RESULT_SCHEMA_V2,
    BENCHMARK_STAGE_RESULT_VALIDATION_SCHEMA_V1, BENCHMARK_STAGE_UNIVERSE_SCHEMA_V1,
    BENCHMARK_SUITE_RUN_MANIFEST_SCHEMA_V2, BUNDLE_SCHEMA_V1, CORPUS_ACQUISITION_SCHEMA_V1,
    CORPUS_ACQUISITION_SCHEMA_V2, CORPUS_CENTRIC_REPORT_SCHEMA_V1, CORPUS_SCHEMA_V1,
    DECISION_SCHEMA_V1, FASTQ_COMPARABLE_METRICS_SCHEMA_V1, FASTQ_CORPUS_SOURCE_SCHEMA_V1,
    FASTQ_PROFILE_READS_EXPECTATIONS_SCHEMA_V1, FASTQ_READ_LENGTH_EXPECTATIONS_SCHEMA_V1,
    FASTQ_READ_LENGTH_EXPECTATIONS_SCHEMA_V2, GOVERNED_DATA_CATALOG_REGISTRY_PUBLICATION_SCHEMA_V1,
    GOVERNED_DATA_CATALOG_REGISTRY_SCHEMA_V1, GOVERNED_DATA_CATALOG_SCHEMA_V1,
    GOVERNED_DATA_CATALOG_SCHEMA_V2, GOVERNED_DATA_CATALOG_SNAPSHOT_PUBLICATION_SCHEMA_V1,
    OBSERVATION_SCHEMA_V1, STAGE_CENTRIC_REPORT_SCHEMA_V1, STAGE_REGISTRY_EXTRA_PAIRS_SCHEMA_V1,
    SUITE_SCHEMA_V1, SUMMARY_SCHEMA_V1, TOOL_CENTRIC_REPORT_SCHEMA_V1,
    VCF_COMPARABLE_METRICS_SCHEMA_V1,
};
pub use crate::diagnostics::BenchError;
pub use crate::model::active_scope::{
    active_scope_fast_validation_report_schema, ActiveScopeFastValidationCheck,
    ActiveScopeFastValidationReport, ACTIVE_SCOPE_FAST_VALIDATION_SCHEMA_V1,
};
pub use crate::model::admission::{
    benchmark_execution_admission_schema, BenchmarkExecutionAdmission,
    BenchmarkExecutionAdmissionPublication, BenchmarkMetricAssessment, BenchmarkOutputFileEvidence,
};
pub use crate::model::all_domain_parser_collector::{
    all_domain_parser_collector_schema, AllDomainParserCollectorReport,
    AllDomainParserCollectorRow, AllDomainParserCollectorSourceKind,
    ALL_DOMAIN_PARSER_COLLECTOR_FAKE_RUN_ROOT, ALL_DOMAIN_PARSER_COLLECTOR_FIXTURE_ROOT,
    ALL_DOMAIN_PARSER_COLLECTOR_OUTPUT_PATH,
};
pub use crate::model::appraisal::{
    appraisal_report_schema, hardening_queue_report_schema, AppraisalFinding, AppraisalReport,
    AppraisalSummary, HardeningQueueEntry, HardeningQueueReport, APPRAISAL_REPORT_SCHEMA_V1,
    HARDENING_QUEUE_REPORT_SCHEMA_V1,
};
pub use crate::model::benchmark_matrix::{
    benchmark_matrix_report_schema, BenchmarkMatrixReport, BenchmarkMatrixRow,
    BenchmarkMatrixSummary, BenchmarkReadiness, BenchmarkSurfaceMatch,
    BENCHMARK_MATRIX_REPORT_SCHEMA_V1,
};
pub use crate::model::campaign_engine::{
    benchmark_admitted_case_observation_schema, benchmark_admitted_case_observation_set_schema,
    benchmark_admitted_case_observation_v2_schema, benchmark_case_aggregation_schema,
    benchmark_case_aggregation_set_schema, benchmark_case_aggregation_v2_schema,
    benchmark_case_matrix_schema, benchmark_case_matrix_v1_schema,
    BenchmarkAdmittedCaseObservationSetV1, BenchmarkAdmittedCaseObservationV1,
    BenchmarkAdmittedCaseObservationV2, BenchmarkCaseAggregationSetV1, BenchmarkCaseAggregationV1,
    BenchmarkCaseAggregationV2, BenchmarkCaseDefinitionV1, BenchmarkCaseExecutionEvidenceV1,
    BenchmarkCaseMatrixV1, BenchmarkCaseMatrixV2, BenchmarkCaseMeasurementPlanV1,
    BenchmarkCaseRepetitionObservationV1, BenchmarkCaseSciencePlanV1, BenchmarkCorpusSnapshotV1,
    BenchmarkDataSnapshotRoleV1, BenchmarkDataSnapshotV1,
};
pub use crate::model::container_report_portfolio::{
    container_report_portfolio_index_schema, ContainerIndexedReport, ContainerIndexedReportFile,
    ContainerReportFreshness, ContainerReportPortfolioIndex, ContainerReportPortfolioSummary,
    CONTAINER_REPORT_PORTFOLIO_INDEX_SCHEMA_V1,
};
pub use crate::model::cross_campaign::{
    cross_benchmark_campaign_report_schema, CrossBenchmarkCampaignReport,
    CrossBenchmarkCampaignSummary, CrossBenchmarkGoalEntry, CrossMatrixRowRef, CrossScenario,
    CROSS_BENCHMARK_CAMPAIGN_SCHEMA_V1,
};
pub use crate::model::data_catalog_registry::{
    governed_data_catalog_registry_schema, GovernedDataCatalogRegistry,
    GovernedDataCatalogRegistryRecord, GovernedDataCatalogSnapshot,
};
pub use crate::model::data_fitness::{
    benchmark_data_fitness_rubric_schema, BenchmarkDataAccessRequirement,
    BenchmarkDataFitnessDefaults, BenchmarkDataFitnessExceptionPolicy, BenchmarkDataFitnessProfile,
    BenchmarkDataFitnessRubric, BenchmarkFitnessException, BenchmarkFitnessExceptionDimension,
    BenchmarkStratificationRequirement, BenchmarkTruthEvidenceRequirement,
};
pub use crate::model::data_portfolio_audit::{
    benchmark_data_portfolio_audit_schema, BenchmarkCorpusDataAudit, BenchmarkDataPortfolioAudit,
    BenchmarkDataPortfolioAuditSummary, BenchmarkDataSourceSpecIdentity,
    BenchmarkGovernedDataBinding, BenchmarkUnboundDataAsset,
};
pub use crate::model::domain_campaign::{
    bam_benchmark_campaign_report_schema, fastq_benchmark_campaign_report_schema,
    hardening_benchmark_campaign_report_schema, vcf_benchmark_campaign_report_schema,
    BamBenchmarkCampaignReport, BamBenchmarkCampaignSummary, BamBenchmarkGoalEntry,
    BamMatrixRowRef, BamScenario, DomainBenchmarkCampaignSummary, DomainBenchmarkGoalEntry,
    DomainBenchmarkMatrixRowRef, DomainBenchmarkScenario, FastqBenchmarkCampaignReport,
    FastqBenchmarkCampaignSummary, FastqBenchmarkGoalEntry, FastqMatrixRowRef, FastqScenario,
    HardeningBenchmarkCampaignReport, HardeningBenchmarkCampaignSummary,
    HardeningBenchmarkGoalEntry, HardeningMatrixRowRef, HardeningScenario,
    VcfBenchmarkCampaignReport, VcfBenchmarkCampaignSummary, VcfBenchmarkGoalEntry,
    VcfMatrixRowRef, VcfScenario, BAM_BENCHMARK_CAMPAIGN_SCHEMA_V1,
    FASTQ_BENCHMARK_CAMPAIGN_SCHEMA_V1, HARDENING_BENCHMARK_CAMPAIGN_SCHEMA_V1,
    VCF_BENCHMARK_CAMPAIGN_SCHEMA_V1,
};
pub use crate::model::dossier_index::{
    benchmark_dossier_index_schema, BenchmarkDossierIndex, BenchmarkDossierStageEntry,
    BENCHMARK_DOSSIER_INDEX_SCHEMA_V1,
};
pub use crate::model::essential_pipeline_readiness::{
    benchmark_essential_pipeline_readiness_schema, BenchmarkEssentialPipelineReadinessGoalCheck,
    BenchmarkEssentialPipelineReadinessGoalContract, BenchmarkEssentialPipelineReadinessReport,
    BENCHMARK_ESSENTIAL_PIPELINE_READINESS_GOALS,
    BENCHMARK_ESSENTIAL_PIPELINE_READINESS_OUTPUT_PATH,
};
pub use crate::model::execution::{
    benchmark_effective_parameters_schema, benchmark_execution_spec_schema,
    BenchmarkAcceptanceExpectedValue, BenchmarkAcceptanceMetric, BenchmarkAcceptanceOperator,
    BenchmarkEffectiveParameterValue, BenchmarkEffectiveParameters, BenchmarkExecutionAcceptance,
    BenchmarkExecutionCachePolicy, BenchmarkExecutionCommands, BenchmarkExecutionDeterminism,
    BenchmarkExecutionDownstreamProbe, BenchmarkExecutionImage, BenchmarkExecutionInput,
    BenchmarkExecutionInputKind, BenchmarkExecutionInterface, BenchmarkExecutionMeasurement,
    BenchmarkExecutionOutputs, BenchmarkExecutionParameterEvidence, BenchmarkExecutionResources,
    BenchmarkExecutionRetryPolicy, BenchmarkExecutionRuntimePolicy, BenchmarkExecutionSourceLock,
    BenchmarkExecutionSpec,
};
pub use crate::model::graph::{
    benchmark_local_hpc_job_graph_schema, BenchmarkGraphNode, BenchmarkGraphNodeKind,
    BenchmarkLocalHpcJobGraph, BenchmarkLocalHpcJobGraphNode, BenchmarkStageEdge,
};
pub use crate::model::interfaces::{
    benchmark_pipeline_parameter_config_schema, benchmark_stage_interface_catalog_schema,
    BenchmarkParameterValue, BenchmarkPipelineParameterConfig,
    BenchmarkReportComparisonPresentation, BenchmarkReportDeterminismPresentation,
    BenchmarkReportMetricFormat, BenchmarkReportPerformanceColumn, BenchmarkReportPresentation,
    BenchmarkReportTitleClass, BenchmarkStageDownstreamProbe, BenchmarkStageInterfaceCatalog,
    BenchmarkStageInterfaceSpec, BenchmarkStageParameterConfig,
};
pub use crate::model::lifecycle::{
    benchmark_attempt_archive_schema, benchmark_attempt_identity_schema,
    benchmark_attempt_migration_ledger_schema, benchmark_attempt_migration_plan_schema,
    benchmark_attempt_migration_verification_schema, benchmark_attempt_schema,
    benchmark_attempt_v2_schema, benchmark_campaign_plan_schema, benchmark_cycle_admission_schema,
    benchmark_cycle_preparation_schema, benchmark_cycle_status_schema,
    benchmark_cycle_status_v2_schema, benchmark_scheduler_observation_schema,
    BenchmarkArchiveFileRecord, BenchmarkArchiveSourcePaths, BenchmarkAttemptArchive,
    BenchmarkAttemptDisposition, BenchmarkAttemptDocument, BenchmarkAttemptIdentity,
    BenchmarkAttemptMigrationEntry, BenchmarkAttemptMigrationExclusion,
    BenchmarkAttemptMigrationLedger, BenchmarkAttemptMigrationLedgerEntry,
    BenchmarkAttemptMigrationPlan, BenchmarkAttemptMigrationState,
    BenchmarkAttemptMigrationVerification, BenchmarkAttemptMigrationVerificationEntry,
    BenchmarkAttemptRecord, BenchmarkAttemptRecordV2, BenchmarkAttemptTransition,
    BenchmarkAttemptV1MigrationContext, BenchmarkCampaignPlan, BenchmarkCycleAdmission,
    BenchmarkCycleAdmissionState, BenchmarkCycleAdmissionUnit, BenchmarkCycleDocument,
    BenchmarkCyclePreparation, BenchmarkCyclePreparationState, BenchmarkCyclePreparationUnit,
    BenchmarkCycleStatus, BenchmarkCycleStatusV2, BenchmarkEvidenceReference,
    BenchmarkJobReference, BenchmarkLifecycleStatus, BenchmarkRunIdentity,
    BenchmarkSchedulerAccounting, BenchmarkSchedulerEvidenceSource, BenchmarkSchedulerObservation,
    BenchmarkSchedulerObservationDraft, BenchmarkSchedulerObservedState,
    BenchmarkSchedulerTerminalState,
};
pub use crate::model::local_benchmark_summary::{
    benchmark_local_summary_schema, derive_benchmark_local_readiness_status,
    BenchmarkLocalReadinessStatus, BenchmarkLocalSummaryReport, BenchmarkLocalSummaryStage,
};
pub use crate::model::local_corpus_run::{
    benchmark_local_corpus_run_schema, BenchmarkCorpusRunManifest, BenchmarkCorpusSampleRun,
};
pub use crate::model::local_corpus_skip::{
    benchmark_local_corpus_skip_report_schema, BenchmarkLocalAssetBackedStage,
    BenchmarkLocalCorpusFixture, BenchmarkLocalCorpusSkip, BenchmarkLocalCorpusSkipReport,
    BenchmarkLocalPlannerOnlyStage,
};
pub use crate::model::local_corpus_stage_compatibility::{
    benchmark_local_corpus_stage_compatibility_schema, BenchmarkLocalCorpusStageCompatibilityEntry,
    BenchmarkLocalCorpusStageCompatibilityReport, BenchmarkLocalCorpusStageValidatedFixture,
};
pub use crate::model::local_hpc_asset_staging::{
    benchmark_local_hpc_asset_staging_schema, BenchmarkLocalHpcAssetStagingInput,
    BenchmarkLocalHpcAssetStagingJob, BenchmarkLocalHpcAssetStagingManifest,
};
pub use crate::model::local_hpc_candidate_run::{
    benchmark_local_hpc_candidate_run_manifest_schema, BenchmarkLocalHpcCandidateBehavior,
    BenchmarkLocalHpcCandidateJob, BenchmarkLocalHpcCandidateResourceCeiling,
    BenchmarkLocalHpcCandidateResourceHint, BenchmarkLocalHpcCandidateRunManifest,
    BenchmarkLocalHpcCandidateStopCondition, BENCHMARK_LOCAL_HPC_CANDIDATE_CPUS_PER_TASK_MAX,
    BENCHMARK_LOCAL_HPC_CANDIDATE_MEMORY_MB_MAX, BENCHMARK_LOCAL_HPC_CANDIDATE_PROFILE_ID,
    BENCHMARK_LOCAL_HPC_CANDIDATE_SCRATCH_GB_MAX,
    BENCHMARK_LOCAL_HPC_CANDIDATE_STAGED_INPUT_BYTES_MAX,
    BENCHMARK_LOCAL_HPC_CANDIDATE_WALLTIME_MINUTES_MAX,
};
pub use crate::model::local_hpc_dependency_simulation::{
    benchmark_local_hpc_dependency_simulation_schema, BenchmarkLocalHpcDependencySimulationCase,
    BenchmarkLocalHpcDependencySimulationJob, BenchmarkLocalHpcDependencySimulationReport,
    BENCHMARK_LOCAL_HPC_REFERENCE_BLOCKED_JOB_ID, BENCHMARK_LOCAL_HPC_REFERENCE_CONTINUED_JOB_ID,
    BENCHMARK_LOCAL_HPC_REFERENCE_FAILURE_JOB_ID, BENCHMARK_LOCAL_HPC_RELATEDNESS_BLOCKED_JOB_ID,
    BENCHMARK_LOCAL_HPC_RELATEDNESS_CONTINUED_JOB_ID,
    BENCHMARK_LOCAL_HPC_RELATEDNESS_FAILURE_JOB_ID,
};
pub use crate::model::local_hpc_execution_resolver::{
    benchmark_local_hpc_execution_resolver_schema, BenchmarkLocalHpcExecutionResolverReport,
    BenchmarkLocalHpcExecutionResolverRow,
};
pub use crate::model::local_hpc_result_collection_simulation::{
    benchmark_local_hpc_result_collection_simulation_schema,
    BenchmarkLocalHpcResultCollectionSimulationBehavior,
    BenchmarkLocalHpcResultCollectionSimulationReport,
    BenchmarkLocalHpcResultCollectionSimulationRow,
};
pub use crate::model::local_hpc_resume_simulation::{
    benchmark_local_hpc_resume_simulation_schema, BenchmarkLocalHpcResumeSimulationBehavior,
    BenchmarkLocalHpcResumeSimulationJob, BenchmarkLocalHpcResumeSimulationReport,
    BENCHMARK_LOCAL_HPC_RESUME_DEPENDENCY_JOB_ID, BENCHMARK_LOCAL_HPC_RESUME_FAILED_JOB_ID,
    BENCHMARK_LOCAL_HPC_RESUME_MISSING_MANIFEST_JOB_ID,
    BENCHMARK_LOCAL_HPC_RESUME_STALE_OUTPUT_JOB_ID, BENCHMARK_LOCAL_HPC_RESUME_UNRELATED_JOB_ID,
    BENCHMARK_LOCAL_HPC_RESUME_VALID_JOB_ID,
};
pub use crate::model::local_hpc_scratch_layout::{
    benchmark_local_hpc_scratch_layout_schema, BenchmarkLocalHpcScratchCleanupPolicy,
    BenchmarkLocalHpcScratchInputLink, BenchmarkLocalHpcScratchJobResources,
    BenchmarkLocalHpcScratchLayout, BenchmarkLocalHpcScratchLayoutJob,
};
pub use crate::model::local_pipeline_dag::{
    benchmark_local_pipeline_dag_validation_schema,
    benchmark_local_pipeline_dag_validation_set_schema, BenchmarkLocalPipelineAncientDnaPolicy,
    BenchmarkLocalPipelineDagValidationNode, BenchmarkLocalPipelineDagValidationProfile,
    BenchmarkLocalPipelineDagValidationReport, BenchmarkLocalPipelineDagValidationSetReport,
    BenchmarkLocalPipelineExecutionContext, BenchmarkLocalPipelineProjectSource,
    BenchmarkLocalPipelineReferenceAssets, BenchmarkLocalPipelineReferenceContext,
    BenchmarkLocalPipelineSnpEffAssets, BenchmarkLocalPipelineVariantAssets,
};
pub use crate::model::local_runtime_metrics::{
    benchmark_local_runtime_metrics_schema, BenchmarkLocalStageReadinessKind,
    BenchmarkLocalStageRuntimeMetric, BenchmarkLocalStageRuntimeMetricsReport,
};
pub use crate::model::local_slurm_dependency::{
    benchmark_local_slurm_dependency_check_schema, benchmark_local_slurm_dry_run_schema,
    benchmark_local_slurm_shell_syntax_schema, BenchmarkLocalSlurmDependencyCheckJob,
    BenchmarkLocalSlurmDependencyCheckReport, BenchmarkLocalSlurmDryRunReport,
    BenchmarkLocalSlurmScriptEntry, BenchmarkLocalSlurmShellSyntaxEntry,
    BenchmarkLocalSlurmShellSyntaxReport,
};
pub use crate::model::local_slurm_submit::{
    benchmark_local_all_domain_slurm_submit_manifest_schema,
    benchmark_local_slurm_submit_manifest_schema, BenchmarkLocalAllDomainSlurmSubmitJob,
    BenchmarkLocalAllDomainSlurmSubmitManifest, BenchmarkLocalSlurmSubmitJob,
    BenchmarkLocalSlurmSubmitLogs, BenchmarkLocalSlurmSubmitManifest,
    BenchmarkLocalSlurmSubmitResources,
};
pub use crate::model::local_stage_commands::{
    benchmark_local_stage_commands_schema, BenchmarkLocalStageArtifactEntry,
    BenchmarkLocalStageCommandArgvEntry, BenchmarkLocalStageCommandEntry,
    BenchmarkLocalStageCommandManifest,
};
pub use crate::model::local_stage_completion::{
    benchmark_local_stage_manifest_completion_schema,
    benchmark_local_stage_output_completion_schema, BenchmarkLocalStageManifestCompletionEntry,
    BenchmarkLocalStageManifestCompletionReport, BenchmarkLocalStageMissingOutputEntry,
    BenchmarkLocalStageOutputCompletionEntry, BenchmarkLocalStageOutputCompletionReport,
};
pub use crate::model::local_stage_inventory::{
    benchmark_local_all_domain_stage_inventory_schema, benchmark_local_stage_inventory_schema,
    BenchmarkLocalAllDomainStageInventory, BenchmarkLocalStageInventory,
    BenchmarkLocalStageInventoryEntry,
};
pub use crate::model::local_vcf_smoke::{
    benchmark_local_vcf_smoke_suite_schema, BenchmarkLocalVcfSmokeGoalCheck,
    BenchmarkLocalVcfSmokeGoalContract, BenchmarkLocalVcfSmokeSuiteReport,
    BENCHMARK_LOCAL_VCF_SMOKE_GOALS, BENCHMARK_LOCAL_VCF_SMOKE_OUTPUT_PATH,
};
pub use crate::model::materialization::{
    corpus_acquisition_plan_schema, corpus_materialization_publication_schema,
    fastq_corpus_source_spec_schema, governed_data_catalog_schema,
    CorpusMaterializationPublication, GovernedDataCatalogDocument, GovernedPreservedDataCatalog,
};
pub use crate::model::normalized_result;
pub use crate::model::observation::benchmark_observation_schema;
pub use crate::model::observation::MetricsEnvelope;
pub use crate::model::planning::{
    benchmark_portfolio_schema, benchmark_run_registry_schema,
    benchmark_scientific_readiness_schema, benchmark_stage_campaign_registry_schema,
    benchmark_stage_coverage_schema, benchmark_stage_universe_schema, BenchmarkCachePolicy,
    BenchmarkComparisonMetricTolerance, BenchmarkCorpusClass, BenchmarkCorpusContract,
    BenchmarkCorpusStatus, BenchmarkDisposition, BenchmarkEvidenceScope, BenchmarkEvidenceTier,
    BenchmarkExecutionRuntime, BenchmarkExecutionScheduler, BenchmarkMeasurementContract,
    BenchmarkPortfolio, BenchmarkProfileAssignment, BenchmarkRunNamespacePolicy,
    BenchmarkRunRegistry, BenchmarkRunRegistryRow, BenchmarkRunRegistryState,
    BenchmarkRunRegistrySummary, BenchmarkScientificProfile, BenchmarkScientificReadinessCorpus,
    BenchmarkScientificReadinessDomain, BenchmarkScientificReadinessReport,
    BenchmarkScientificReadinessStage, BenchmarkScientificReadinessStatus,
    BenchmarkScientificReadinessSummary, BenchmarkStageCampaignRegistry,
    BenchmarkStageCampaignSpec, BenchmarkStageCoverage, BenchmarkStageCoverageRow,
    BenchmarkStageCoverageSummary, BenchmarkStageDefinitionStatus, BenchmarkStageExecutionStatus,
    BenchmarkStageInterfaceStatus, BenchmarkStageUniverse, BenchmarkStageUniverseRow,
    BenchmarkStageUniverseSummary,
};
pub use crate::model::protocol::{
    benchmark_case_definition_schema, benchmark_run_definition_schema, BenchmarkCaseDefinition,
    BenchmarkCaseInputLock, BenchmarkCaseQualificationReceipt, BenchmarkContractLock,
    BenchmarkCorpusRole, BenchmarkRunDefinition, BenchmarkRunState,
};
pub use crate::model::publication::{
    benchmark_citation_record_schema, benchmark_evidence_book_schema,
    benchmark_publication_receipt_schema, BenchmarkCitationAssetIdentity, BenchmarkCitationRecord,
    BenchmarkCitationToolIdentity, BenchmarkEvidenceBook, BenchmarkEvidenceBookEntry,
    BenchmarkEvidenceBookStatus, BenchmarkPublicationReceipt,
};
pub use crate::model::publication_findings::{
    benchmark_publication_findings_schema, BenchmarkPublicationFindings,
    BENCHMARK_PUBLICATION_FINDINGS_SCHEMA_V1,
};
pub use crate::model::publication_outcome::{
    benchmark_campaign_report_publication_schema,
    governed_data_catalog_registry_publication_schema,
    governed_data_catalog_snapshot_publication_schema, BenchmarkCampaignReportPublication,
    GovernedDataCatalogRegistryPublication, GovernedDataCatalogSnapshotPublication,
};
pub use crate::model::publication_registry::{
    benchmark_publication_registry_schema, BenchmarkCurrentPublication,
    BenchmarkPublicationRegistry, BenchmarkPublicationRegistrySummary,
    BenchmarkQuarantinedPublicationSurface, BenchmarkStagePublication,
    BenchmarkStagePublicationDisposition,
};
pub use crate::model::publication_status::{
    benchmark_publication_status_schema, BenchmarkPublicationAuditIssue,
    BenchmarkPublicationExcludedStage, BenchmarkPublicationStageStatus,
    BenchmarkPublicationStatusReport, BENCHMARK_PUBLICATION_STATUS_SCHEMA_V1,
};
pub use crate::model::published_results_status::{
    benchmark_published_results_status_schema, PublishedResultsStageReport,
    PublishedResultsStatusReport, StageResultIssue, BENCHMARK_PUBLISHED_RESULTS_STATUS_SCHEMA_V1,
};
pub use crate::model::remediation_queue::{
    benchmark_remediation_queue_schema, BenchmarkRemediationIssue, BenchmarkRemediationIssueGroup,
    BenchmarkRemediationQueue, BenchmarkRemediationStageEntry,
    BENCHMARK_REMEDIATION_QUEUE_SCHEMA_V1,
};
pub use crate::model::replay::{
    benchmark_replay_receipt_schema, BenchmarkReplayOutcome, BenchmarkReplayReceipt,
    BenchmarkReplaySemanticIdentity,
};
pub use crate::model::report::{
    admitted_benchmark_stage_report_schema, benchmark_stage_report_schema,
    AdmittedBenchmarkStageReport, BenchmarkComparisonMetric, BenchmarkComparisonStatus,
    BenchmarkCrossToolComparison, BenchmarkMetricDistribution, BenchmarkReportComparability,
    BenchmarkReportDeterminism, BenchmarkReportDownstreamProbe, BenchmarkReportImage,
    BenchmarkReportInputLock, BenchmarkReportSlurmJob, BenchmarkReportStageInterface,
    BenchmarkReportVerdict, BenchmarkScientificStratum, BenchmarkScientificValue,
    BenchmarkStageReport, BenchmarkStageReportDesign, BenchmarkToolReport,
};
pub use crate::model::run_index::{
    benchmark_protocol_citation_schema, benchmark_run_case_index_schema,
    benchmark_run_index_schema, benchmark_run_publication_pointer_schema,
    BenchmarkProtocolCitation, BenchmarkProtocolCitationStatus, BenchmarkRunCaseIndex,
    BenchmarkRunCaseIndexEntry, BenchmarkRunIndex, BenchmarkRunIndexEntry,
    BenchmarkRunIndexFileReference, BenchmarkRunIndexPublication, BenchmarkRunIndexSummary,
    BenchmarkRunPublicationPointer,
};
pub use crate::model::source_authority::{
    benchmark_data_source_spec_schema, BenchmarkDataSourceCollection, BenchmarkDataSourceSpec,
};
pub use crate::model::stage_result::{
    benchmark_stage_result_schema, benchmark_stage_result_validation_schema,
    BenchmarkStageResultCommand, BenchmarkStageResultManifest, BenchmarkStageResultOutput,
    BenchmarkStageResultResourceMetricSource, BenchmarkStageResultResourceMetrics,
    BenchmarkStageResultRuntime, BenchmarkStageResultStatus, BenchmarkStageResultTool,
    BenchmarkStageResultValidationReport,
};
pub use crate::model::stage_tool_closure::{
    benchmark_stage_tool_closure_schema, BenchmarkStageToolClosureDisposition,
    BenchmarkStageToolClosureReport, BenchmarkStageToolClosureRow, BenchmarkStageToolClosureSource,
    BenchmarkStageToolClosureSummary, BenchmarkStageToolProtocolDisposition,
    BENCHMARK_STAGE_TOOL_CLOSURE_SCHEMA_V1,
};
pub use crate::model::suite::{
    benchmark_suite_run_manifest_schema, benchmark_suite_spec_schema, AnalysisRequirements,
    BenchmarkParamBinding, BenchmarkStageSpec, ColdWarmContract, DatasetSpec, DecisionTraceRow,
    DiversityRequirements, EnvironmentSnapshot, FairnessContract, MetricsArtifactRow,
    PostconditionRow, ReplicatePolicy, RunRecordRow, ScientificDefaultsStatus,
    StratificationRequirement, SuiteRunManifest, ToolInvocationRow,
};
pub use crate::model::summary::{
    benchmark_summary_schema, MetricSummary, SummaryRow, SummaryStratum,
};
pub use crate::model::{
    all_domain_active_stage_tool_matrix_schema, all_domain_retained_tools_schema,
    bam_comparable_metrics_schema, benchmark_bundle_manifest_schema,
    benchmark_corpus_dossier_summary_schema, benchmark_corpus_manifest_schema,
    benchmark_data_index_schema, corpus_centric_report_schema, fastq_comparable_metrics_schema,
    fastq_profile_reads_expectations_schema, stage_centric_report_schema,
    stage_registry_extra_pairs_schema, tool_centric_report_schema, vcf_comparable_metrics_schema,
    AdmittedAlignmentTruthSample, AdmittedAlignmentTruthSet, AdmittedBamCohort,
    AdmittedBamCohortSample, AdmittedBamCorpus, AdmittedBamCorpusFile, AdmittedFastqCorpus,
    AdmittedFastqCorpusFile, AdmittedFastqCorpusSample, AdmittedReferenceBundle,
    AllDomainActiveStageToolMatrixReport, AllDomainActiveStageToolMatrixRow,
    AllDomainRetainedToolRow, AllDomainRetainedToolsReport, BackendComparisonSpec,
    BamComparableMetricsReport, BamComparableMetricsRow, BenchmarkBundleManifest,
    BenchmarkCorpusDossierHeadline, BenchmarkCorpusDossierSampleResultRow,
    BenchmarkCorpusDossierSummary, BenchmarkCorpusDossierToolSummary, BenchmarkCorpusManifest,
    BenchmarkDataIndex, BenchmarkDataIndexFile, BenchmarkDataIndexRecord,
    BenchmarkDataIndexSummary, BenchmarkEligibility, BenchmarkFastqLayout,
    BenchmarkTemporalProvenance, ComparableMetricContractStatus, ComparableMetricDirection,
    CorpusAcquisitionFile, CorpusAcquisitionLayout, CorpusAcquisitionPlan, CorpusAcquisitionSample,
    CorpusCentricCorpusReport, CorpusCentricReport, CorpusCentricStageRow, CorpusDatasetSpec,
    CorpusDomain, CorpusMaterializationDisposition, CorpusScale, CorpusSourcePolicy,
    DataIntegrityStatus, DriftScenarioSpec, FastqComparableMetricsReport,
    FastqComparableMetricsRow, FastqCorpusSourceSample, FastqCorpusSourceSpec,
    FastqProfileReadsExpectationsProvenance, FastqReadLengthExpectationSample,
    FastqReadLengthExpectations, GovernedDataCatalog, GovernedDataCatalogFile,
    GovernedDataCatalogSample, ReadinessComparisonContractStatus, StageCentricReport,
    StageCentricStageReport, StageCentricToolRow, StageRegistryExtraPairRow,
    StageRegistryExtraPairsReport, ToolCentricReport, ToolCentricStageRow, ToolCentricToolReport,
    TruthSetHook, TruthSetStatus, VcfComparableMetricsReport, VcfComparableMetricsRow,
    BENCHMARK_CORPUS_DOSSIER_SUMMARY_SCHEMA_V1,
};
pub use crate::model::{BenchmarkObservation, BenchmarkSuiteSpec, BenchmarkSummary};
pub use crate::policy::{
    gate_decision_schema, GateDecision, GatePolicy, GatePolicyOverrides, GateViolation,
};
pub use crate::stats::robust_estimators::robust_stats;
