//! Owner: bijux-dna-bench-model
//! Typed models for bench (suite, observation, summary, decision).
//! Must not perform IO or depend on repo/policy/compare logic.

pub mod active_scope;
pub mod admission;
pub mod all_domain_failure_classification;
pub mod all_domain_parser_collector;
pub mod appraisal;
pub mod benchmark_config;
pub mod benchmark_matrix;
pub mod campaign_engine;
pub mod comparable_metrics;
pub mod container_report_portfolio;
pub mod corpus;
pub mod corpus_dossier;
pub mod cross_campaign;
pub mod data_catalog_registry;
pub mod data_fitness;
pub mod data_portfolio_audit;
pub mod derived_catalog;
pub mod domain_campaign;
pub mod dossier_index;
pub mod essential_pipeline_readiness;
pub mod execution;
pub mod expected_benchmark_results;
pub mod full_benchmark_report;
pub mod full_benchmark_result_collector;
pub mod graph;
pub mod interfaces;
pub mod lifecycle;
pub mod local_benchmark_summary;
pub mod local_corpus_run;
pub mod local_corpus_skip;
pub mod local_corpus_stage_compatibility;
pub mod local_essential_pipeline_fake_run;
pub mod local_hpc_array;
pub mod local_hpc_asset_staging;
pub mod local_hpc_candidate_run;
pub mod local_hpc_dependency_simulation;
pub mod local_hpc_dry_run_readiness;
pub mod local_hpc_execution_resolver;
pub mod local_hpc_pipeline_node_array;
pub mod local_hpc_result_collection_simulation;
pub mod local_hpc_resume_simulation;
pub mod local_hpc_scratch_layout;
pub mod local_hpc_stage_benchmark_array;
pub mod local_hpc_submission_readiness;
pub mod local_pipeline_dag;
pub mod local_runtime_metrics;
pub mod local_slurm_dependency;
pub mod local_slurm_scripts;
pub mod local_slurm_submit;
pub mod local_stage_commands;
pub mod local_stage_completion;
pub mod local_stage_fake_run;
pub mod local_stage_inventory;
pub mod local_vcf_call_gl_smoke;
pub mod local_vcf_call_smoke;
pub mod local_vcf_ibd_smoke;
pub mod local_vcf_imputation_metrics_smoke;
pub mod local_vcf_smoke;
pub mod local_vcf_stage_catalog;
pub mod local_vcf_stage_matrix;
pub mod materialization;
pub mod normalized_result;
pub mod observation;
pub mod output_declarations;
pub mod persisted_schema;
pub mod planning;
pub mod progress_ledger;
pub mod protocol;
pub mod publication;
pub mod publication_findings;
pub mod publication_outcome;
pub mod publication_registry;
pub mod publication_status;
pub mod published_results_status;
pub mod readiness_views;
pub mod remediation_queue;
pub mod replay;
pub mod report;
pub mod run_index;
pub mod scientific_acceptance;
pub mod scope_views;
pub mod slurm_submission;
pub mod source_authority;
pub mod stage_result;
pub mod stage_scoring;
pub mod stage_tool_assets;
pub mod stage_tool_benchmark_ready;
pub mod stage_tool_closure;
pub mod stage_tool_containers;
pub mod stage_tool_resources;
pub mod suite;
pub mod summary;
pub mod workspace_layout;

#[allow(unused_imports)]
pub use active_scope::{
    active_scope_fast_validation_report_schema, ActiveScopeFastValidationCheck,
    ActiveScopeFastValidationReport, ACTIVE_SCOPE_FAST_VALIDATION_SCHEMA_V1,
};
#[allow(unused_imports)]
pub use admission::{
    benchmark_execution_admission_schema, BenchmarkExecutionAdmission,
    BenchmarkExecutionAdmissionPublication, BenchmarkMetricAssessment, BenchmarkOutputFileEvidence,
};
#[allow(unused_imports)]
pub use all_domain_failure_classification::{
    all_domain_failure_classification_schema, AllDomainFailureClassificationReport,
    AllDomainFailureClassificationRow,
};
#[allow(unused_imports)]
pub use all_domain_parser_collector::{
    all_domain_parser_collector_schema, AllDomainParserCollectorReport,
    AllDomainParserCollectorRow, AllDomainParserCollectorSourceKind,
    ALL_DOMAIN_PARSER_COLLECTOR_FAKE_RUN_ROOT, ALL_DOMAIN_PARSER_COLLECTOR_FIXTURE_ROOT,
    ALL_DOMAIN_PARSER_COLLECTOR_OUTPUT_PATH,
};
#[allow(unused_imports)]
pub use appraisal::{
    appraisal_report_schema, hardening_queue_report_schema, AppraisalFinding, AppraisalReport,
    AppraisalSummary, HardeningQueueEntry, HardeningQueueReport, APPRAISAL_REPORT_SCHEMA_V1,
    HARDENING_QUEUE_REPORT_SCHEMA_V1,
};
#[allow(unused_imports)]
pub use benchmark_config::{
    benchmark_config_schema, BenchmarkConfig, BenchmarkCorpusConfig,
    BenchmarkCorpusPublicationConfig, BenchmarkDepleteRrnaInputConfig, BenchmarkPublicationConfig,
    BenchmarkReferenceInputConfig, BenchmarkScreenTaxonomyInputConfig, BenchmarkStageInputConfig,
    BenchmarkWorkspaceArtifact, BenchmarkWorkspaceConfig, BenchmarkWorkspaceLayout,
    BenchmarkWorkspaceLocal, BenchmarkWorkspaceRemote, BenchmarkWorkspaceStageRuns,
    BenchmarkWorkspaceSync, BenchmarkWorkspaceSyncDefaults, CorpusBenchmarkContract,
    CorpusBenchmarkExclusion,
};
#[allow(unused_imports)]
pub use benchmark_matrix::{
    benchmark_matrix_report_schema, BenchmarkMatrixReport, BenchmarkMatrixRow,
    BenchmarkMatrixSummary, BenchmarkReadiness, BenchmarkSurfaceMatch,
    BENCHMARK_MATRIX_REPORT_SCHEMA_V1,
};
#[allow(unused_imports)]
pub use campaign_engine::{
    benchmark_admitted_case_observation_schema, benchmark_admitted_case_observation_set_schema,
    benchmark_admitted_case_observation_v2_schema, benchmark_case_aggregation_schema,
    benchmark_case_aggregation_set_schema, benchmark_case_aggregation_v2_schema,
    benchmark_case_matrix_schema, benchmark_case_matrix_v1_schema,
    benchmark_stage_normalization_schema, BenchmarkAdmittedCaseObservationSetV1,
    BenchmarkAdmittedCaseObservationV1, BenchmarkAdmittedCaseObservationV2,
    BenchmarkCaseAggregationSetV1, BenchmarkCaseAggregationV1, BenchmarkCaseAggregationV2,
    BenchmarkCaseDefinitionV1, BenchmarkCaseExecutionEvidenceV1, BenchmarkCaseMatrixV1,
    BenchmarkCaseMatrixV2, BenchmarkCaseMeasurementPlanV1, BenchmarkCaseRepetitionObservationV1,
    BenchmarkCaseSciencePlanV1, BenchmarkCorpusSnapshotV1, BenchmarkDataArtifactSnapshotV1,
    BenchmarkDataSnapshotRoleV1, BenchmarkDataSnapshotV1, BenchmarkStageNormalizationCheckV1,
    BenchmarkStageNormalizationUnitV1, BenchmarkStageNormalizationV1,
    BENCHMARK_STAGE_NORMALIZATION_ARTIFACT,
};
#[allow(unused_imports)]
pub use comparable_metrics::{
    bam_comparable_metrics_schema, fastq_comparable_metrics_schema, vcf_comparable_metrics_schema,
    BamComparableMetricsReport, BamComparableMetricsRow, ComparableMetricContractStatus,
    ComparableMetricDirection, FastqComparableMetricsReport, FastqComparableMetricsRow,
    VcfComparableMetricsReport, VcfComparableMetricsRow,
};
#[allow(unused_imports)]
pub use container_report_portfolio::{
    container_report_portfolio_index_schema, ContainerIndexedReport, ContainerIndexedReportFile,
    ContainerReportFreshness, ContainerReportPortfolioIndex, ContainerReportPortfolioSummary,
    CONTAINER_REPORT_PORTFOLIO_INDEX_SCHEMA_V1,
};
#[allow(unused_imports)]
pub use corpus::{
    benchmark_bundle_manifest_schema, benchmark_corpus_manifest_schema,
    benchmark_data_index_schema, fastq_profile_reads_expectations_schema,
    AdmittedAlignmentTruthSample, AdmittedAlignmentTruthSet, AdmittedBamCohort,
    AdmittedBamCohortSample, AdmittedBamCorpus, AdmittedBamCorpusFile, AdmittedFastqCorpus,
    AdmittedFastqCorpusFile, AdmittedFastqCorpusSample, AdmittedIndexedReference,
    AdmittedReferenceBundle, BackendComparisonSpec, BenchmarkBundleManifest,
    BenchmarkCorpusManifest, BenchmarkDataIndex, BenchmarkDataIndexFile, BenchmarkDataIndexRecord,
    BenchmarkDataIndexSummary, BenchmarkEligibility, BenchmarkFastqLayout,
    BenchmarkTemporalProvenance, CorpusDatasetSpec, CorpusDomain, CorpusScale, DataIntegrityStatus,
    DriftScenarioSpec, FastqProfileReadsExpectationsProvenance, FastqReadLengthExpectationSample,
    FastqReadLengthExpectations, TruthSetHook, TruthSetStatus,
};
#[allow(unused_imports)]
pub use corpus_dossier::{
    benchmark_corpus_dossier_summary_schema, BenchmarkCorpusDossierHeadline,
    BenchmarkCorpusDossierSampleResultRow, BenchmarkCorpusDossierSummary,
    BenchmarkCorpusDossierToolSummary, BENCHMARK_CORPUS_DOSSIER_SUMMARY_SCHEMA_V1,
};
#[allow(unused_imports)]
pub use cross_campaign::{
    cross_benchmark_campaign_report_schema, CrossBenchmarkCampaignReport,
    CrossBenchmarkCampaignSummary, CrossBenchmarkGoalEntry, CrossMatrixRowRef, CrossScenario,
    CROSS_BENCHMARK_CAMPAIGN_SCHEMA_V1,
};
#[allow(unused_imports)]
pub use data_catalog_registry::{
    governed_data_catalog_registry_schema, GovernedDataCatalogRegistry,
    GovernedDataCatalogRegistryRecord, GovernedDataCatalogSnapshot,
};
#[allow(unused_imports)]
pub use data_fitness::{
    benchmark_data_fitness_rubric_schema, BenchmarkDataAccessRequirement,
    BenchmarkDataFitnessDefaults, BenchmarkDataFitnessExceptionPolicy, BenchmarkDataFitnessProfile,
    BenchmarkDataFitnessRubric, BenchmarkFitnessException, BenchmarkFitnessExceptionDimension,
    BenchmarkStratificationRequirement, BenchmarkTruthEvidenceRequirement,
};
#[allow(unused_imports)]
pub use data_portfolio_audit::{
    benchmark_data_portfolio_audit_schema, BenchmarkCorpusDataAudit, BenchmarkDataPortfolioAudit,
    BenchmarkDataPortfolioAuditSummary, BenchmarkDataSourceSpecIdentity,
    BenchmarkGovernedDataBinding, BenchmarkUnboundDataAsset,
};
#[allow(unused_imports)]
pub use derived_catalog::{
    governed_derived_data_catalog_schema, GovernedDataDerivation, GovernedDerivedDataCatalog,
    GovernedDerivedDataCatalogFile, GovernedDerivedDataCatalogSample, GovernedDerivedDataSource,
};
#[allow(unused_imports)]
pub use domain_campaign::{
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
#[allow(unused_imports)]
pub use dossier_index::{
    benchmark_dossier_index_schema, BenchmarkDossierIndex, BenchmarkDossierStageEntry,
    BENCHMARK_DOSSIER_INDEX_SCHEMA_V1,
};
#[allow(unused_imports)]
pub use essential_pipeline_readiness::{
    benchmark_essential_pipeline_readiness_schema, BenchmarkEssentialPipelineReadinessGoalCheck,
    BenchmarkEssentialPipelineReadinessGoalContract, BenchmarkEssentialPipelineReadinessReport,
    BENCHMARK_ESSENTIAL_PIPELINE_READINESS_GOALS,
    BENCHMARK_ESSENTIAL_PIPELINE_READINESS_OUTPUT_PATH,
};
#[allow(unused_imports)]
pub use execution::{
    benchmark_effective_parameters_schema, benchmark_execution_spec_schema,
    benchmark_stage_plan_manifest_schema, BenchmarkAcceptanceExpectedValue,
    BenchmarkAcceptanceMetric, BenchmarkAcceptanceOperator, BenchmarkEffectiveParameterValue,
    BenchmarkEffectiveParameters, BenchmarkExecutionAcceptance, BenchmarkExecutionCachePolicy,
    BenchmarkExecutionCommands, BenchmarkExecutionDeterminism, BenchmarkExecutionDownstreamProbe,
    BenchmarkExecutionImage, BenchmarkExecutionInput, BenchmarkExecutionInputKind,
    BenchmarkExecutionInterface, BenchmarkExecutionMeasurement, BenchmarkExecutionOutputs,
    BenchmarkExecutionParameterEvidence, BenchmarkExecutionResources,
    BenchmarkExecutionRetryPolicy, BenchmarkExecutionRuntimePolicy, BenchmarkExecutionSourceLock,
    BenchmarkExecutionSpec, BenchmarkStagePlanManifest, BenchmarkStagePlanReference,
};
#[allow(unused_imports)]
pub use expected_benchmark_results::{
    all_domain_expected_benchmark_results_schema, AllDomainExpectedBenchmarkResultRow,
    AllDomainExpectedBenchmarkResultsReport,
};
#[allow(unused_imports)]
pub use full_benchmark_report::{
    full_benchmark_report_schema, FullBenchmarkComparableMetricRow, FullBenchmarkCorpusSectionRow,
    FullBenchmarkFailureRow, FullBenchmarkFailuresSection, FullBenchmarkMemoryRow,
    FullBenchmarkMissingResultRow, FullBenchmarkPipelineSection, FullBenchmarkPipelineSectionRow,
    FullBenchmarkReport, FullBenchmarkReportRow, FullBenchmarkReportRowStatus,
    FullBenchmarkRuntimeRow, FullBenchmarkStageSectionRow, FullBenchmarkToolSectionRow,
    FullBenchmarkUnsupportedPairRow,
};
#[allow(unused_imports)]
pub use full_benchmark_result_collector::{
    full_benchmark_result_collector_schema, FullBenchmarkResultCollectorReport,
    FullBenchmarkResultCollectorRow, FullBenchmarkResultStatus, FullBenchmarkResultSurfaceKind,
};
#[allow(unused_imports)]
pub use graph::{
    benchmark_local_hpc_job_graph_schema, BenchmarkGraphNode, BenchmarkGraphNodeKind,
    BenchmarkLocalHpcJobGraph, BenchmarkLocalHpcJobGraphNode, BenchmarkStageEdge,
};
#[allow(unused_imports)]
pub use interfaces::{
    BenchmarkParameterValue, BenchmarkPipelineParameterConfig,
    BenchmarkReportComparisonPresentation, BenchmarkReportDeterminismPresentation,
    BenchmarkReportMetricFormat, BenchmarkReportPerformanceColumn, BenchmarkReportPresentation,
    BenchmarkReportTitleClass, BenchmarkStageDownstreamProbe, BenchmarkStageInterfaceCatalog,
    BenchmarkStageInterfaceSpec, BenchmarkStageParameterConfig,
};
#[allow(unused_imports)]
pub use lifecycle::{
    benchmark_attempt_identity_schema, benchmark_attempt_migration_verification_schema,
    benchmark_campaign_plan_schema, benchmark_cycle_admission_schema,
    benchmark_cycle_preparation_schema, benchmark_scheduler_observation_schema,
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
#[allow(unused_imports)]
pub use local_benchmark_summary::{
    benchmark_local_summary_schema, derive_benchmark_local_readiness_status,
    BenchmarkLocalReadinessStatus, BenchmarkLocalSummaryReport, BenchmarkLocalSummaryStage,
};
#[allow(unused_imports)]
pub use local_corpus_run::{
    benchmark_local_corpus_run_schema, BenchmarkCorpusRunManifest, BenchmarkCorpusSampleRun,
};
#[allow(unused_imports)]
pub use local_corpus_skip::{
    benchmark_local_corpus_skip_report_schema, BenchmarkLocalAssetBackedStage,
    BenchmarkLocalCorpusFixture, BenchmarkLocalCorpusSkip, BenchmarkLocalCorpusSkipReport,
    BenchmarkLocalPlannerOnlyStage,
};
#[allow(unused_imports)]
pub use local_corpus_stage_compatibility::{
    benchmark_local_corpus_stage_compatibility_schema, BenchmarkLocalCorpusStageCompatibilityEntry,
    BenchmarkLocalCorpusStageCompatibilityReport, BenchmarkLocalCorpusStageValidatedFixture,
};
#[allow(unused_imports)]
pub use local_essential_pipeline_fake_run::{
    benchmark_local_essential_pipeline_fake_run_node_metrics_schema,
    benchmark_local_essential_pipeline_fake_run_pipeline_schema,
    benchmark_local_essential_pipeline_fake_runs_schema,
    BenchmarkLocalEssentialPipelineFakeRunInputBinding,
    BenchmarkLocalEssentialPipelineFakeRunNodeMetrics,
    BenchmarkLocalEssentialPipelineFakeRunNodeReport,
    BenchmarkLocalEssentialPipelineFakeRunOutputEntry,
    BenchmarkLocalEssentialPipelineFakeRunPipelineReport,
    BenchmarkLocalEssentialPipelineFakeRunsReport,
};
#[allow(unused_imports)]
pub use local_hpc_array::BenchmarkLocalHpcArrayResources;
#[allow(unused_imports)]
pub use local_hpc_asset_staging::{
    benchmark_local_hpc_asset_staging_schema, BenchmarkLocalHpcAssetStagingInput,
    BenchmarkLocalHpcAssetStagingJob, BenchmarkLocalHpcAssetStagingManifest,
};
#[allow(unused_imports)]
pub use local_hpc_candidate_run::{
    benchmark_local_hpc_candidate_run_manifest_schema, BenchmarkLocalHpcCandidateBehavior,
    BenchmarkLocalHpcCandidateJob, BenchmarkLocalHpcCandidateResourceCeiling,
    BenchmarkLocalHpcCandidateResourceHint, BenchmarkLocalHpcCandidateRunManifest,
    BenchmarkLocalHpcCandidateStopCondition, BENCHMARK_LOCAL_HPC_CANDIDATE_CPUS_PER_TASK_MAX,
    BENCHMARK_LOCAL_HPC_CANDIDATE_MEMORY_MB_MAX, BENCHMARK_LOCAL_HPC_CANDIDATE_PROFILE_ID,
    BENCHMARK_LOCAL_HPC_CANDIDATE_SCRATCH_GB_MAX,
    BENCHMARK_LOCAL_HPC_CANDIDATE_STAGED_INPUT_BYTES_MAX,
    BENCHMARK_LOCAL_HPC_CANDIDATE_WALLTIME_MINUTES_MAX,
};
#[allow(unused_imports)]
pub use local_hpc_dependency_simulation::{
    benchmark_local_hpc_dependency_simulation_schema, BenchmarkLocalHpcDependencySimulationCase,
    BenchmarkLocalHpcDependencySimulationJob, BenchmarkLocalHpcDependencySimulationReport,
    BENCHMARK_LOCAL_HPC_REFERENCE_BLOCKED_JOB_ID, BENCHMARK_LOCAL_HPC_REFERENCE_CONTINUED_JOB_ID,
    BENCHMARK_LOCAL_HPC_REFERENCE_FAILURE_JOB_ID, BENCHMARK_LOCAL_HPC_RELATEDNESS_BLOCKED_JOB_ID,
    BENCHMARK_LOCAL_HPC_RELATEDNESS_CONTINUED_JOB_ID,
    BENCHMARK_LOCAL_HPC_RELATEDNESS_FAILURE_JOB_ID,
};
#[allow(unused_imports)]
pub use local_hpc_dry_run_readiness::{
    benchmark_local_hpc_dry_run_readiness_schema, BenchmarkLocalHpcDryRunReadinessBehavior,
    BenchmarkLocalHpcDryRunReadinessGoalCheck, BenchmarkLocalHpcDryRunReadinessReport,
    BenchmarkLocalHpcDryRunReadinessSummary,
};
#[allow(unused_imports)]
pub use local_hpc_execution_resolver::{
    benchmark_local_hpc_execution_resolver_schema, BenchmarkLocalHpcExecutionResolverReport,
    BenchmarkLocalHpcExecutionResolverRow,
};
#[allow(unused_imports)]
pub use local_hpc_pipeline_node_array::{
    benchmark_local_hpc_pipeline_node_array_schema, BenchmarkLocalHpcPipelineNodeArrayDependency,
    BenchmarkLocalHpcPipelineNodeArrayExpectedOutput, BenchmarkLocalHpcPipelineNodeArrayReport,
    BenchmarkLocalHpcPipelineNodeArrayRow,
};
#[allow(unused_imports)]
pub use local_hpc_result_collection_simulation::{
    benchmark_local_hpc_result_collection_simulation_schema,
    BenchmarkLocalHpcResultCollectionSimulationBehavior,
    BenchmarkLocalHpcResultCollectionSimulationReport,
    BenchmarkLocalHpcResultCollectionSimulationRow,
};
#[allow(unused_imports)]
pub use local_hpc_resume_simulation::{
    benchmark_local_hpc_resume_simulation_schema, BenchmarkLocalHpcResumeSimulationBehavior,
    BenchmarkLocalHpcResumeSimulationJob, BenchmarkLocalHpcResumeSimulationReport,
    BENCHMARK_LOCAL_HPC_RESUME_DEPENDENCY_JOB_ID, BENCHMARK_LOCAL_HPC_RESUME_FAILED_JOB_ID,
    BENCHMARK_LOCAL_HPC_RESUME_MISSING_MANIFEST_JOB_ID,
    BENCHMARK_LOCAL_HPC_RESUME_STALE_OUTPUT_JOB_ID, BENCHMARK_LOCAL_HPC_RESUME_UNRELATED_JOB_ID,
    BENCHMARK_LOCAL_HPC_RESUME_VALID_JOB_ID,
};
#[allow(unused_imports)]
pub use local_hpc_scratch_layout::{
    benchmark_local_hpc_scratch_layout_schema, BenchmarkLocalHpcScratchCleanupPolicy,
    BenchmarkLocalHpcScratchInputLink, BenchmarkLocalHpcScratchJobResources,
    BenchmarkLocalHpcScratchLayout, BenchmarkLocalHpcScratchLayoutJob,
};
#[allow(unused_imports)]
pub use local_hpc_stage_benchmark_array::{
    benchmark_local_hpc_stage_benchmark_array_schema, BenchmarkLocalHpcStageBenchmarkArrayReport,
    BenchmarkLocalHpcStageBenchmarkArrayRow,
};
#[allow(unused_imports)]
pub use local_hpc_submission_readiness::{
    benchmark_local_hpc_submission_readiness_schema, BenchmarkLocalHpcSubmissionReadinessGoalCheck,
    BenchmarkLocalHpcSubmissionReadinessReport,
};
#[allow(unused_imports)]
pub use local_pipeline_dag::{
    benchmark_local_pipeline_dag_config_schema,
    benchmark_local_pipeline_dag_metadata_config_schema,
    benchmark_local_pipeline_dag_stages_config_schema,
    benchmark_local_pipeline_dag_validation_schema,
    benchmark_local_pipeline_dag_validation_set_schema, BenchmarkLocalPipelineAncientDnaPolicy,
    BenchmarkLocalPipelineDagConfig, BenchmarkLocalPipelineDagDomain,
    BenchmarkLocalPipelineDagMetadataConfig, BenchmarkLocalPipelineDagNode,
    BenchmarkLocalPipelineDagStagesConfig, BenchmarkLocalPipelineDagValidationNode,
    BenchmarkLocalPipelineDagValidationProfile, BenchmarkLocalPipelineDagValidationReport,
    BenchmarkLocalPipelineDagValidationSetReport, BenchmarkLocalPipelineExecutionContext,
    BenchmarkLocalPipelineProjectSource, BenchmarkLocalPipelineProjectSourceConfig,
    BenchmarkLocalPipelineReferenceAssets, BenchmarkLocalPipelineReferenceContext,
    BenchmarkLocalPipelineSnpEffAssets, BenchmarkLocalPipelineVariantAssets,
};
#[allow(unused_imports)]
pub use local_runtime_metrics::{
    benchmark_local_runtime_metrics_schema, BenchmarkLocalStageReadinessKind,
    BenchmarkLocalStageRuntimeMetric, BenchmarkLocalStageRuntimeMetricsReport,
};
#[allow(unused_imports)]
pub use local_slurm_dependency::{
    benchmark_local_slurm_dependency_check_schema, benchmark_local_slurm_dry_run_schema,
    benchmark_local_slurm_shell_syntax_schema, BenchmarkLocalSlurmDependencyCheckJob,
    BenchmarkLocalSlurmDependencyCheckReport, BenchmarkLocalSlurmDryRunReport,
    BenchmarkLocalSlurmScriptEntry, BenchmarkLocalSlurmShellSyntaxEntry,
    BenchmarkLocalSlurmShellSyntaxReport,
};
#[allow(unused_imports)]
pub use local_slurm_submit::{
    benchmark_local_all_domain_slurm_submit_manifest_schema,
    benchmark_local_slurm_submit_manifest_schema, BenchmarkLocalAllDomainSlurmSubmitJob,
    BenchmarkLocalAllDomainSlurmSubmitManifest, BenchmarkLocalSlurmSubmitJob,
    BenchmarkLocalSlurmSubmitLogs, BenchmarkLocalSlurmSubmitManifest,
    BenchmarkLocalSlurmSubmitResources,
};
#[allow(unused_imports)]
pub use local_stage_commands::{
    benchmark_local_stage_commands_schema, BenchmarkLocalStageArtifactEntry,
    BenchmarkLocalStageCommandArgvEntry, BenchmarkLocalStageCommandEntry,
    BenchmarkLocalStageCommandManifest,
};
#[allow(unused_imports)]
pub use local_stage_completion::{
    benchmark_local_stage_manifest_completion_schema,
    benchmark_local_stage_output_completion_schema, BenchmarkLocalStageManifestCompletionEntry,
    BenchmarkLocalStageManifestCompletionReport, BenchmarkLocalStageMissingOutputEntry,
    BenchmarkLocalStageOutputCompletionEntry, BenchmarkLocalStageOutputCompletionReport,
};
#[allow(unused_imports)]
pub use local_stage_fake_run::{
    benchmark_local_stage_fake_failure_manifest_schema,
    benchmark_local_stage_fake_failure_record_schema,
    benchmark_local_stage_fake_run_manifest_schema, BenchmarkLocalStageFakeFailureManifest,
    BenchmarkLocalStageFakeFailureOutputEntry, BenchmarkLocalStageFakeFailureRecord,
    BenchmarkLocalStageFakeRunManifest, BenchmarkLocalStageFakeRunOutputEntry,
    BenchmarkLocalStageFakeRunResult,
};
#[allow(unused_imports)]
pub use local_stage_inventory::{
    benchmark_local_all_domain_stage_inventory_schema, benchmark_local_stage_inventory_schema,
    benchmark_local_stage_matrix_config_schema, BenchmarkLocalAllDomainStageInventory,
    BenchmarkLocalStageInventory, BenchmarkLocalStageInventoryEntry,
    BenchmarkLocalStageMatrixConfig, BENCHMARK_LOCAL_BAM_STAGE_MATRIX_SCHEMA_V1,
    BENCHMARK_LOCAL_FASTQ_STAGE_MATRIX_SCHEMA_V1,
};
#[allow(unused_imports)]
pub use local_vcf_call_gl_smoke::{
    benchmark_local_vcf_call_gl_smoke_metrics_schema, benchmark_local_vcf_call_gl_smoke_schema,
    BenchmarkLocalVcfCallGlSmokeMetrics, BenchmarkLocalVcfCallGlSmokeReport,
    BENCHMARK_LOCAL_VCF_CALL_GL_SMOKE_METRICS_SCHEMA_V1,
    BENCHMARK_LOCAL_VCF_CALL_GL_SMOKE_SCHEMA_V1,
};
#[allow(unused_imports)]
pub use local_vcf_call_smoke::{
    benchmark_local_vcf_call_smoke_metrics_schema, benchmark_local_vcf_call_smoke_schema,
    BenchmarkLocalVcfCallSmokeMetrics, BenchmarkLocalVcfCallSmokeReport,
    BENCHMARK_LOCAL_VCF_CALL_SMOKE_METRICS_SCHEMA_V1, BENCHMARK_LOCAL_VCF_CALL_SMOKE_SCHEMA_V1,
};
#[allow(unused_imports)]
pub use local_vcf_ibd_smoke::{
    benchmark_local_vcf_ibd_smoke_schema, BenchmarkLocalVcfIbdInsufficientOverlapProbe,
    BenchmarkLocalVcfIbdSmokeReport, BenchmarkLocalVcfIbdSmokeRow,
    BENCHMARK_LOCAL_VCF_IBD_SMOKE_SCHEMA_V1,
};
#[allow(unused_imports)]
pub use local_vcf_imputation_metrics_smoke::{
    benchmark_local_vcf_imputation_metrics_smoke_schema,
    BenchmarkImputationQualityFieldAvailability, BenchmarkLocalVcfImputationMetricsSmokeReport,
    BENCHMARK_LOCAL_VCF_IMPUTATION_METRICS_SMOKE_SCHEMA_V1,
};
#[allow(unused_imports)]
pub use local_vcf_smoke::{
    benchmark_local_vcf_smoke_suite_schema, BenchmarkLocalVcfSmokeGoalCheck,
    BenchmarkLocalVcfSmokeGoalContract, BenchmarkLocalVcfSmokeSuiteReport,
    BENCHMARK_LOCAL_VCF_SMOKE_GOALS, BENCHMARK_LOCAL_VCF_SMOKE_OUTPUT_PATH,
};
#[allow(unused_imports)]
pub use local_vcf_stage_catalog::{
    local_vcf_stage_catalog_config_schema, local_vcf_stage_catalog_report_schema,
    LocalVcfStageCatalogConfig, LocalVcfStageCatalogReport, VcfStageCatalogRow,
};
#[allow(unused_imports)]
pub use local_vcf_stage_matrix::{
    local_vcf_stage_matrix_config_schema, local_vcf_stage_matrix_report_schema,
    validate_vcf_stage_matrix_report_schema, LocalVcfStageMatrixConfig, LocalVcfStageMatrixReport,
    ValidateVcfStageMatrixReport, VcfStageMatrixRow,
};
#[allow(unused_imports)]
pub use materialization::{
    corpus_acquisition_plan_schema, corpus_materialization_publication_schema,
    fastq_corpus_source_spec_schema, governed_data_catalog_schema, CorpusAcquisitionFile,
    CorpusAcquisitionLayout, CorpusAcquisitionPlan, CorpusAcquisitionSample,
    CorpusEvaluationDesign, CorpusMaterializationDisposition, CorpusMaterializationPublication,
    CorpusSourcePolicy, FastqCorpusSourceSample, FastqCorpusSourceSpec, GovernedDataCatalog,
    GovernedDataCatalogDocument, GovernedDataCatalogFile, GovernedDataCatalogSample,
    GovernedPreservedDataCatalog,
};
pub use observation::BenchmarkObservation;
#[allow(unused_imports)]
pub use output_declarations::{
    all_domain_output_declarations_schema, AllDomainOutputDeclarationRow,
    AllDomainOutputDeclarationStatus, AllDomainOutputDeclarationsReport,
};
#[allow(unused_imports)]
pub use planning::{
    benchmark_run_registry_schema, benchmark_stage_coverage_schema,
    benchmark_stage_universe_schema, BenchmarkCachePolicy, BenchmarkCampaignMaterialDriver,
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
#[allow(unused_imports)]
pub use progress_ledger::{
    benchmark_progress_ledger_schema, BenchmarkProgressAttempt, BenchmarkProgressAttemptStatus,
    BenchmarkProgressCheck, BenchmarkProgressCheckStatus, BenchmarkProgressCommit,
    BenchmarkProgressExecutionUnit, BenchmarkProgressExecutionUnitStatus,
    BenchmarkProgressExternalWait, BenchmarkProgressFrontier, BenchmarkProgressLedger,
    BenchmarkProgressLedgerSource, BenchmarkProgressOwner, BenchmarkProgressTask,
    BenchmarkProgressTaskStatus,
};
#[allow(unused_imports)]
pub use protocol::{
    benchmark_case_definition_schema, benchmark_run_definition_schema, BenchmarkCaseDefinition,
    BenchmarkCaseInputLock, BenchmarkCaseQualificationReceipt, BenchmarkContractLock,
    BenchmarkCorpusRole, BenchmarkRunDefinition, BenchmarkRunState,
};
#[allow(unused_imports)]
pub use publication::{
    benchmark_citation_record_schema, benchmark_evidence_book_schema,
    benchmark_publication_receipt_schema, BenchmarkCitationAssetIdentity, BenchmarkCitationRecord,
    BenchmarkCitationToolIdentity, BenchmarkEvidenceBook, BenchmarkEvidenceBookEntry,
    BenchmarkEvidenceBookStatus, BenchmarkPublicationReceipt,
};
#[allow(unused_imports)]
pub use publication_findings::{
    benchmark_publication_findings_schema, BenchmarkPublicationFindings,
    BENCHMARK_PUBLICATION_FINDINGS_SCHEMA_V1,
};
#[allow(unused_imports)]
pub use publication_outcome::{
    benchmark_campaign_report_publication_schema,
    governed_data_catalog_registry_publication_schema,
    governed_data_catalog_snapshot_publication_schema, BenchmarkCampaignReportPublication,
    GovernedDataCatalogRegistryPublication, GovernedDataCatalogSnapshotPublication,
};
#[allow(unused_imports)]
pub use publication_registry::{
    benchmark_publication_registry_schema, BenchmarkCurrentPublication,
    BenchmarkPublicationRegistry, BenchmarkPublicationRegistrySummary,
    BenchmarkQuarantinedPublicationSurface, BenchmarkRunReportDisposition,
    BenchmarkRunReportIndexEntry, BenchmarkStagePublication, BenchmarkStagePublicationDisposition,
};
#[allow(unused_imports)]
pub use publication_status::{
    benchmark_publication_status_schema, BenchmarkPublicationAuditIssue,
    BenchmarkPublicationExcludedStage, BenchmarkPublicationStageStatus,
    BenchmarkPublicationStatusReport, BENCHMARK_PUBLICATION_STATUS_SCHEMA_V1,
};
#[allow(unused_imports)]
pub use published_results_status::{
    benchmark_published_results_status_schema, PublishedResultsStageReport,
    PublishedResultsStatusReport, StageResultIssue, BENCHMARK_PUBLISHED_RESULTS_STATUS_SCHEMA_V1,
};
#[allow(unused_imports)]
pub use readiness_views::{
    corpus_centric_report_schema, stage_centric_report_schema, tool_centric_report_schema,
    CorpusCentricCorpusReport, CorpusCentricReport, CorpusCentricStageRow,
    ReadinessComparisonContractStatus, StageCentricReport, StageCentricStageReport,
    StageCentricToolRow, ToolCentricReport, ToolCentricStageRow, ToolCentricToolReport,
};
#[allow(unused_imports)]
pub use remediation_queue::{
    benchmark_remediation_queue_schema, BenchmarkRemediationIssue, BenchmarkRemediationIssueGroup,
    BenchmarkRemediationQueue, BenchmarkRemediationStageEntry,
    BENCHMARK_REMEDIATION_QUEUE_SCHEMA_V1,
};
#[allow(unused_imports)]
pub use replay::{
    benchmark_replay_receipt_schema, BenchmarkReplayOutcome, BenchmarkReplayReceipt,
    BenchmarkReplaySemanticIdentity,
};
#[allow(unused_imports)]
pub use report::{
    AdmittedBenchmarkStageReport, BenchmarkComparisonMetric, BenchmarkComparisonStatus,
    BenchmarkCrossToolComparison, BenchmarkMetricDistribution, BenchmarkReportComparability,
    BenchmarkReportDeterminism, BenchmarkReportDownstreamProbe, BenchmarkReportImage,
    BenchmarkReportInputLock, BenchmarkReportSlurmJob, BenchmarkReportStageInterface,
    BenchmarkReportVerdict, BenchmarkScientificStratum, BenchmarkScientificValue,
    BenchmarkStageReport, BenchmarkStageReportDesign, BenchmarkToolReport,
};
#[allow(unused_imports)]
pub use run_index::{
    benchmark_protocol_citation_schema, benchmark_run_case_index_schema,
    benchmark_run_index_schema, benchmark_run_publication_pointer_schema,
    BenchmarkProtocolCitation, BenchmarkProtocolCitationStatus, BenchmarkRunCaseIndex,
    BenchmarkRunCaseIndexEntry, BenchmarkRunIndex, BenchmarkRunIndexEntry,
    BenchmarkRunIndexFileReference, BenchmarkRunIndexPublication, BenchmarkRunIndexSummary,
    BenchmarkRunPublicationPointer,
};
#[allow(unused_imports)]
pub use scientific_acceptance::{
    scientific_acceptance_thresholds_report_schema, scientific_acceptance_thresholds_schema,
    ScientificAcceptanceDirection, ScientificAcceptanceInsufficiencyBehavior,
    ScientificAcceptancePassRule, ScientificAcceptanceThresholdRow,
    ScientificAcceptanceThresholdsConfig, ScientificAcceptanceThresholdsReport,
    ScientificAcceptanceToleranceKind,
};
#[allow(unused_imports)]
pub use scope_views::{
    all_domain_active_stage_tool_matrix_schema, all_domain_retained_tools_schema,
    stage_registry_extra_pairs_schema, AllDomainActiveStageToolMatrixReport,
    AllDomainActiveStageToolMatrixRow, AllDomainRetainedToolRow, AllDomainRetainedToolsReport,
    StageRegistryExtraPairRow, StageRegistryExtraPairsReport,
};
#[allow(unused_imports)]
pub use slurm_submission::{
    benchmark_slurm_submission_intent_schema, BenchmarkSlurmArraySpec,
    BenchmarkSlurmDependencyCondition, BenchmarkSlurmSubmissionIntent,
    BenchmarkSlurmSubmissionMaterial,
};
#[allow(unused_imports)]
pub use source_authority::{
    benchmark_data_source_spec_schema, BenchmarkDataSourceCollection, BenchmarkDataSourceSpec,
};
#[allow(unused_imports)]
pub use stage_result::{
    benchmark_stage_result_schema, benchmark_stage_result_validation_schema,
    BenchmarkStageResultCommand, BenchmarkStageResultManifest, BenchmarkStageResultOutput,
    BenchmarkStageResultResourceMetricSource, BenchmarkStageResultResourceMetrics,
    BenchmarkStageResultRuntime, BenchmarkStageResultStatus, BenchmarkStageResultTool,
    BenchmarkStageResultValidationReport,
};
#[allow(unused_imports)]
pub use stage_tool_assets::{
    stage_tool_assets_config_schema, stage_tool_assets_report_schema, StageToolAssetRow,
    StageToolAssetsConfig, StageToolAssetsReport, STAGE_TOOL_ASSETS_DECLARATION_ORIGIN,
    STAGE_TOOL_ASSETS_SCOPE,
};
#[allow(unused_imports)]
pub use stage_tool_benchmark_ready::{
    stage_tool_benchmark_ready_schema, StageToolBenchmarkReadyExcludedPair,
    StageToolBenchmarkReadyFailingPair, StageToolBenchmarkReadyReport,
    StageToolBenchmarkReadySurfaceStatus, StageToolBenchmarkReadySurfaceSummary,
};
#[allow(unused_imports)]
pub use stage_tool_closure::{
    benchmark_stage_tool_closure_schema, BenchmarkStageToolClosureDisposition,
    BenchmarkStageToolClosureReport, BenchmarkStageToolClosureRow, BenchmarkStageToolClosureSource,
    BenchmarkStageToolClosureSummary, BenchmarkStageToolProtocolDisposition,
    BENCHMARK_STAGE_TOOL_CLOSURE_SCHEMA_V1,
};
#[allow(unused_imports)]
pub use stage_tool_containers::{
    stage_tool_containers_config_schema, stage_tool_containers_report_schema,
    StageToolContainerRow, StageToolContainersConfig, StageToolContainersReport,
    STAGE_TOOL_CONTAINERS_DECLARATION_ORIGIN, STAGE_TOOL_CONTAINERS_SCOPE,
};
#[allow(unused_imports)]
pub use stage_tool_resources::{
    stage_tool_resources_config_schema, stage_tool_resources_report_schema, StageToolResourceRow,
    StageToolResourcesConfig, StageToolResourcesReport, BAM_STAGE_TOOL_RESOURCE_ORIGIN,
    FASTQ_STAGE_TOOL_RESOURCE_ORIGIN, STAGE_TOOL_RESOURCES_SCOPE, VCF_STAGE_TOOL_RESOURCE_ORIGIN,
};
#[allow(unused_imports)]
pub use suite::{
    benchmark_suite_run_manifest_schema, benchmark_suite_spec_schema, BenchmarkParamBinding,
    BenchmarkStageSpec, BenchmarkSuiteSpec, ColdWarmContract, DecisionTraceRow,
    EnvironmentSnapshot, FairnessContract, MetricsArtifactRow, PostconditionRow, RunRecordRow,
    ScientificDefaultsStatus, SuiteRunManifest, ToolInvocationRow,
};
#[allow(unused_imports)]
pub use summary::{BenchmarkSummary, MetricSummary, SummaryRow, SummaryStratum};
#[allow(unused_imports)]
pub use workspace_layout::{
    workspace_convergence_plan_schema, workspace_layout_report_schema,
    workspace_normalization_report_schema, WorkspaceAuthoritativeRoots, WorkspaceConvergenceAction,
    WorkspaceConvergencePlan, WorkspaceEntrySummary, WorkspaceLayoutIssue, WorkspaceLayoutReport,
    WorkspaceLocalStageLayout, WorkspaceNormalizationReport, WorkspaceRootPairSummary,
    WorkspaceStageNormalizationReport,
};

pub use local_slurm_scripts::{
    BenchmarkLocalAllDomainSlurmScriptEntry, BenchmarkLocalAllDomainSlurmScriptsReport,
    BENCHMARK_LOCAL_ALL_DOMAIN_SLURM_SCRIPTS_SCHEMA_V1,
};
