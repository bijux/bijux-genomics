//! Owner: bijux-dna-bench-model
//! Stable crate-root re-exports.

pub use crate::compare;
pub use crate::contract;
pub use crate::contract::{
    append_attempt_transition, benchmark_attempt_identity_sha256, benchmark_run_definition_sha256,
    benchmark_run_directory_name, migrate_benchmark_attempt_v1, parse_benchmark_attempt_document,
    parse_benchmark_cycle_document, validate_attempt_archive, validate_attempt_identity,
    validate_attempt_record_v1, validate_attempt_record_v2, validate_attempt_transition,
    validate_benchmark_case_definition, validate_benchmark_citation_record,
    validate_benchmark_data_index, validate_benchmark_data_portfolio_audit,
    validate_benchmark_data_source_spec, validate_benchmark_evidence_book,
    validate_benchmark_protocol_citation, validate_benchmark_publication_receipt,
    validate_benchmark_publication_registry, validate_benchmark_replay_receipt,
    validate_benchmark_run_case_index, validate_benchmark_run_definition,
    validate_benchmark_run_definition_revision, validate_benchmark_run_index,
    validate_benchmark_run_publication_pointer, validate_benchmark_run_registry,
    validate_benchmark_stage_result_manifest, validate_campaign_plan,
    validate_container_report_portfolio_index, validate_corpus_acquisition_plan,
    validate_cycle_admission, validate_cycle_preparation, validate_cycle_status_v1,
    validate_cycle_status_v2, validate_data_fitness_portfolio_alignment,
    validate_data_fitness_rubric, validate_execution_spec, validate_fastq_corpus_source_spec,
    validate_fastq_read_length_expectations, validate_governed_data_catalog,
    validate_governed_data_catalog_document, validate_governed_data_catalog_registry,
    validate_governed_preserved_data_catalog, validate_scheduler_observation,
    validate_scientific_readiness, validate_stage_universe, validate_utc_timestamp,
    BENCHMARK_ADMITTED_CASE_OBSERVATION_SCHEMA_V1, BENCHMARK_ADMITTED_CASE_OBSERVATION_SCHEMA_V2,
    BENCHMARK_ADMITTED_CASE_OBSERVATION_SET_SCHEMA_V1, BENCHMARK_ATTEMPT_ARCHIVE_SCHEMA_V1,
    BENCHMARK_ATTEMPT_MIGRATION_LEDGER_SCHEMA_V1, BENCHMARK_ATTEMPT_MIGRATION_PLAN_SCHEMA_V1,
    BENCHMARK_ATTEMPT_MIGRATION_VERIFICATION_SCHEMA_V1, BENCHMARK_ATTEMPT_SCHEMA_V1,
    BENCHMARK_ATTEMPT_SCHEMA_V2, BENCHMARK_CAMPAIGN_PLAN_SCHEMA_V1,
    BENCHMARK_CASE_AGGREGATION_SCHEMA_V1, BENCHMARK_CASE_AGGREGATION_SCHEMA_V2,
    BENCHMARK_CASE_AGGREGATION_SET_SCHEMA_V1, BENCHMARK_CASE_DEFINITION_SCHEMA_V1,
    BENCHMARK_CASE_MATRIX_SCHEMA_V1, BENCHMARK_CASE_MATRIX_SCHEMA_V2,
    BENCHMARK_CITATION_RECORD_SCHEMA_V1, BENCHMARK_CORPUS_MATERIALIZATION_PUBLICATION_SCHEMA_V1,
    BENCHMARK_CYCLE_ADMISSION_SCHEMA_V1, BENCHMARK_CYCLE_PREPARATION_SCHEMA_V1,
    BENCHMARK_CYCLE_STATUS_SCHEMA_V1, BENCHMARK_CYCLE_STATUS_SCHEMA_V2,
    BENCHMARK_DATA_FITNESS_SCHEMA_V1, BENCHMARK_DATA_INDEX_SCHEMA_V1,
    BENCHMARK_DATA_INDEX_SCHEMA_V2, BENCHMARK_DATA_PORTFOLIO_AUDIT_SCHEMA_V1,
    BENCHMARK_DATA_PORTFOLIO_AUDIT_SCHEMA_V2, BENCHMARK_DATA_SOURCE_SCHEMA_V1,
    BENCHMARK_EFFECTIVE_PARAMETERS_SCHEMA_V1, BENCHMARK_EVIDENCE_BOOK_SCHEMA_V1,
    BENCHMARK_EXECUTION_ADMISSION_SCHEMA_V1, BENCHMARK_EXECUTION_SPEC_SCHEMA_V1,
    BENCHMARK_PIPELINE_PARAMETERS_SCHEMA_V1, BENCHMARK_PORTFOLIO_SCHEMA_V1,
    BENCHMARK_PORTFOLIO_SCHEMA_V2, BENCHMARK_PROTOCOL_CITATION_SCHEMA_V1,
    BENCHMARK_PUBLICATION_RECEIPT_SCHEMA_V1, BENCHMARK_PUBLICATION_REGISTRY_SCHEMA_V1,
    BENCHMARK_REPLAY_RECEIPT_SCHEMA_V1, BENCHMARK_REPORT_ADMISSION_SCHEMA_V1,
    BENCHMARK_RUN_CASE_INDEX_SCHEMA_V1, BENCHMARK_RUN_DEFINITION_SCHEMA_V1,
    BENCHMARK_RUN_INDEX_SCHEMA_V1, BENCHMARK_RUN_PUBLICATION_POINTER_SCHEMA_V1,
    BENCHMARK_RUN_REGISTRY_SCHEMA_V1, BENCHMARK_SCHEDULER_OBSERVATION_SCHEMA_V1,
    BENCHMARK_SCIENTIFIC_READINESS_SCHEMA_V1, BENCHMARK_SCIENTIFIC_READINESS_SCHEMA_V2,
    BENCHMARK_STAGE_CAMPAIGNS_SCHEMA_V1, BENCHMARK_STAGE_COVERAGE_SCHEMA_V1,
    BENCHMARK_STAGE_INTERFACES_SCHEMA_V1, BENCHMARK_STAGE_REPORT_SCHEMA_V1,
    BENCHMARK_STAGE_RESULT_SCHEMA_V2, BENCHMARK_STAGE_UNIVERSE_SCHEMA_V1, BUNDLE_SCHEMA_V1,
    CORPUS_ACQUISITION_SCHEMA_V1, CORPUS_ACQUISITION_SCHEMA_V2, CORPUS_SCHEMA_V1,
    DECISION_SCHEMA_V1, FASTQ_CORPUS_SOURCE_SCHEMA_V1, FASTQ_PROFILE_READS_EXPECTATIONS_SCHEMA_V1,
    FASTQ_READ_LENGTH_EXPECTATIONS_SCHEMA_V1, FASTQ_READ_LENGTH_EXPECTATIONS_SCHEMA_V2,
    GOVERNED_DATA_CATALOG_REGISTRY_SCHEMA_V1, GOVERNED_DATA_CATALOG_SCHEMA_V1,
    GOVERNED_DATA_CATALOG_SCHEMA_V2, OBSERVATION_SCHEMA_V1, SUITE_SCHEMA_V1, SUMMARY_SCHEMA_V1,
};
pub use crate::diagnostics::BenchError;
pub use crate::model::admission::{
    benchmark_execution_admission_schema, BenchmarkExecutionAdmission,
    BenchmarkExecutionAdmissionPublication, BenchmarkMetricAssessment, BenchmarkOutputFileEvidence,
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
pub use crate::model::graph::{BenchmarkGraphNode, BenchmarkGraphNodeKind, BenchmarkStageEdge};
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
pub use crate::model::publication_registry::{
    benchmark_publication_registry_schema, BenchmarkCurrentPublication,
    BenchmarkPublicationRegistry, BenchmarkPublicationRegistrySummary,
    BenchmarkQuarantinedPublicationSurface, BenchmarkStagePublication,
    BenchmarkStagePublicationDisposition,
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
    BenchmarkReportVerdict, BenchmarkScientificValue, BenchmarkStageReport,
    BenchmarkStageReportDesign, BenchmarkToolReport,
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
    benchmark_stage_result_schema, BenchmarkStageResultCommand, BenchmarkStageResultManifest,
    BenchmarkStageResultOutput, BenchmarkStageResultResourceMetricSource,
    BenchmarkStageResultResourceMetrics, BenchmarkStageResultRuntime, BenchmarkStageResultStatus,
    BenchmarkStageResultTool,
};
pub use crate::model::suite::{
    benchmark_suite_spec_schema, AnalysisRequirements, BenchmarkParamBinding, BenchmarkStageSpec,
    DatasetSpec, DiversityRequirements, ReplicatePolicy, StratificationRequirement,
};
pub use crate::model::summary::{
    benchmark_summary_schema, MetricSummary, SummaryRow, SummaryStratum,
};
pub use crate::model::{
    benchmark_bundle_manifest_schema, benchmark_corpus_manifest_schema,
    benchmark_data_index_schema, fastq_profile_reads_expectations_schema, AdmittedBamCorpus,
    AdmittedBamCorpusFile, AdmittedFastqCorpus, AdmittedFastqCorpusFile, AdmittedFastqCorpusSample,
    BackendComparisonSpec, BenchmarkBundleManifest, BenchmarkCorpusManifest, BenchmarkDataIndex,
    BenchmarkDataIndexFile, BenchmarkDataIndexRecord, BenchmarkDataIndexSummary,
    BenchmarkEligibility, BenchmarkFastqLayout, BenchmarkTemporalProvenance, CorpusAcquisitionFile,
    CorpusAcquisitionLayout, CorpusAcquisitionPlan, CorpusAcquisitionSample, CorpusDatasetSpec,
    CorpusDomain, CorpusMaterializationDisposition, CorpusScale, CorpusSourcePolicy,
    DataIntegrityStatus, DriftScenarioSpec, FastqCorpusSourceSample, FastqCorpusSourceSpec,
    FastqProfileReadsExpectationsProvenance, FastqReadLengthExpectationSample,
    FastqReadLengthExpectations, GovernedDataCatalog, GovernedDataCatalogFile,
    GovernedDataCatalogSample, TruthSetHook, TruthSetStatus,
};
pub use crate::model::{BenchmarkObservation, BenchmarkSuiteSpec, BenchmarkSummary};
pub use crate::policy::{
    gate_decision_schema, GateDecision, GatePolicy, GatePolicyOverrides, GateViolation,
};
pub use crate::stats::robust_estimators::robust_stats;
