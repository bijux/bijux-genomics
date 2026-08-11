//! Owner: bijux-dna-bench-model
//! Typed models for bench (suite, observation, summary, decision).
//! Must not perform IO or depend on repo/policy/compare logic.

pub mod active_scope;
pub mod admission;
pub mod appraisal;
pub mod benchmark_matrix;
pub mod campaign_engine;
pub mod container_report_portfolio;
pub mod corpus;
pub mod cross_campaign;
pub mod data_catalog_registry;
pub mod data_fitness;
pub mod data_portfolio_audit;
pub mod domain_campaign;
pub mod execution;
pub mod graph;
pub mod interfaces;
pub mod lifecycle;
pub mod materialization;
pub mod normalized_result;
pub mod observation;
pub mod planning;
pub mod protocol;
pub mod publication;
pub mod publication_outcome;
pub mod publication_registry;
pub mod replay;
pub mod report;
pub mod run_index;
pub mod source_authority;
pub mod stage_result;
pub mod suite;
pub mod summary;

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
pub use appraisal::{
    appraisal_report_schema, hardening_queue_report_schema, AppraisalFinding, AppraisalReport,
    AppraisalSummary, HardeningQueueEntry, HardeningQueueReport, APPRAISAL_REPORT_SCHEMA_V1,
    HARDENING_QUEUE_REPORT_SCHEMA_V1,
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
    BenchmarkAdmittedCaseObservationSetV1, BenchmarkAdmittedCaseObservationV1,
    BenchmarkAdmittedCaseObservationV2, BenchmarkCaseAggregationSetV1, BenchmarkCaseAggregationV1,
    BenchmarkCaseAggregationV2, BenchmarkCaseDefinitionV1, BenchmarkCaseExecutionEvidenceV1,
    BenchmarkCaseMatrixV1, BenchmarkCaseMatrixV2, BenchmarkCaseMeasurementPlanV1,
    BenchmarkCaseRepetitionObservationV1, BenchmarkCaseSciencePlanV1, BenchmarkCorpusSnapshotV1,
    BenchmarkDataSnapshotRoleV1, BenchmarkDataSnapshotV1,
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
    benchmark_data_index_schema, fastq_profile_reads_expectations_schema, AdmittedBamCorpus,
    AdmittedBamCorpusFile, AdmittedFastqCorpus, AdmittedFastqCorpusFile, AdmittedFastqCorpusSample,
    BackendComparisonSpec, BenchmarkBundleManifest, BenchmarkCorpusManifest, BenchmarkDataIndex,
    BenchmarkDataIndexFile, BenchmarkDataIndexRecord, BenchmarkDataIndexSummary,
    BenchmarkEligibility, BenchmarkFastqLayout, BenchmarkTemporalProvenance, CorpusDatasetSpec,
    CorpusDomain, CorpusScale, DataIntegrityStatus, DriftScenarioSpec,
    FastqProfileReadsExpectationsProvenance, FastqReadLengthExpectationSample,
    FastqReadLengthExpectations, TruthSetHook, TruthSetStatus,
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
pub use execution::{
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
#[allow(unused_imports)]
pub use graph::{BenchmarkGraphNode, BenchmarkGraphNodeKind, BenchmarkStageEdge};
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
pub use materialization::{
    corpus_acquisition_plan_schema, corpus_materialization_publication_schema,
    fastq_corpus_source_spec_schema, governed_data_catalog_schema, CorpusAcquisitionFile,
    CorpusAcquisitionLayout, CorpusAcquisitionPlan, CorpusAcquisitionSample,
    CorpusMaterializationDisposition, CorpusMaterializationPublication, CorpusSourcePolicy,
    FastqCorpusSourceSample, FastqCorpusSourceSpec, GovernedDataCatalog,
    GovernedDataCatalogDocument, GovernedDataCatalogFile, GovernedDataCatalogSample,
    GovernedPreservedDataCatalog,
};
pub use observation::BenchmarkObservation;
#[allow(unused_imports)]
pub use planning::{
    benchmark_run_registry_schema, benchmark_stage_coverage_schema,
    benchmark_stage_universe_schema, BenchmarkCachePolicy, BenchmarkComparisonMetricTolerance,
    BenchmarkCorpusClass, BenchmarkCorpusContract, BenchmarkCorpusStatus, BenchmarkDisposition,
    BenchmarkEvidenceScope, BenchmarkEvidenceTier, BenchmarkExecutionRuntime,
    BenchmarkExecutionScheduler, BenchmarkMeasurementContract, BenchmarkPortfolio,
    BenchmarkProfileAssignment, BenchmarkRunNamespacePolicy, BenchmarkRunRegistry,
    BenchmarkRunRegistryRow, BenchmarkRunRegistryState, BenchmarkRunRegistrySummary,
    BenchmarkScientificProfile, BenchmarkScientificReadinessCorpus,
    BenchmarkScientificReadinessDomain, BenchmarkScientificReadinessReport,
    BenchmarkScientificReadinessStage, BenchmarkScientificReadinessStatus,
    BenchmarkScientificReadinessSummary, BenchmarkStageCampaignRegistry,
    BenchmarkStageCampaignSpec, BenchmarkStageCoverage, BenchmarkStageCoverageRow,
    BenchmarkStageCoverageSummary, BenchmarkStageDefinitionStatus, BenchmarkStageExecutionStatus,
    BenchmarkStageInterfaceStatus, BenchmarkStageUniverse, BenchmarkStageUniverseRow,
    BenchmarkStageUniverseSummary,
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
    BenchmarkQuarantinedPublicationSurface, BenchmarkStagePublication,
    BenchmarkStagePublicationDisposition,
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
    BenchmarkReportVerdict, BenchmarkScientificValue, BenchmarkStageReport,
    BenchmarkStageReportDesign, BenchmarkToolReport,
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
pub use source_authority::{
    benchmark_data_source_spec_schema, BenchmarkDataSourceCollection, BenchmarkDataSourceSpec,
};
#[allow(unused_imports)]
pub use stage_result::{
    benchmark_stage_result_schema, BenchmarkStageResultCommand, BenchmarkStageResultManifest,
    BenchmarkStageResultOutput, BenchmarkStageResultResourceMetricSource,
    BenchmarkStageResultResourceMetrics, BenchmarkStageResultRuntime, BenchmarkStageResultStatus,
    BenchmarkStageResultTool,
};
#[allow(unused_imports)]
pub use suite::{
    benchmark_suite_spec_schema, BenchmarkParamBinding, BenchmarkStageSpec, BenchmarkSuiteSpec,
};
#[allow(unused_imports)]
pub use summary::{BenchmarkSummary, MetricSummary, SummaryRow, SummaryStratum};
