//! Owner: bijux-dna-bench-model
//! Stable crate-root re-exports.

pub use crate::compare;
pub use crate::contract;
pub use crate::contract::{
    append_attempt_transition, benchmark_run_directory_name, migrate_benchmark_attempt_v1,
    parse_benchmark_attempt_document, parse_benchmark_cycle_document, validate_attempt_archive,
    validate_attempt_identity, validate_attempt_record_v1, validate_attempt_record_v2,
    validate_attempt_transition, validate_benchmark_case_definition, validate_benchmark_data_index,
    validate_benchmark_run_definition, validate_benchmark_stage_result_manifest,
    validate_campaign_plan, validate_corpus_acquisition_plan, validate_cycle_status_v1,
    validate_cycle_status_v2, validate_execution_spec, validate_fastq_read_length_expectations,
    validate_governed_data_catalog, validate_scheduler_observation, validate_scientific_readiness,
    validate_utc_timestamp, BENCHMARK_ATTEMPT_ARCHIVE_SCHEMA_V1,
    BENCHMARK_ATTEMPT_MIGRATION_LEDGER_SCHEMA_V1, BENCHMARK_ATTEMPT_MIGRATION_PLAN_SCHEMA_V1,
    BENCHMARK_ATTEMPT_SCHEMA_V1, BENCHMARK_ATTEMPT_SCHEMA_V2, BENCHMARK_CAMPAIGN_PLAN_SCHEMA_V1,
    BENCHMARK_CASE_DEFINITION_SCHEMA_V1, BENCHMARK_CYCLE_STATUS_SCHEMA_V1,
    BENCHMARK_CYCLE_STATUS_SCHEMA_V2, BENCHMARK_DATA_INDEX_SCHEMA_V1,
    BENCHMARK_EXECUTION_SPEC_SCHEMA_V1, BENCHMARK_PIPELINE_PARAMETERS_SCHEMA_V1,
    BENCHMARK_PORTFOLIO_SCHEMA_V1, BENCHMARK_REPORT_ADMISSION_SCHEMA_V1,
    BENCHMARK_RUN_DEFINITION_SCHEMA_V1, BENCHMARK_SCHEDULER_OBSERVATION_SCHEMA_V1,
    BENCHMARK_SCIENTIFIC_READINESS_SCHEMA_V1, BENCHMARK_STAGE_CAMPAIGNS_SCHEMA_V1,
    BENCHMARK_STAGE_COVERAGE_SCHEMA_V1, BENCHMARK_STAGE_INTERFACES_SCHEMA_V1,
    BENCHMARK_STAGE_REPORT_SCHEMA_V1, BENCHMARK_STAGE_RESULT_SCHEMA_V2, BUNDLE_SCHEMA_V1,
    CORPUS_ACQUISITION_SCHEMA_V1, CORPUS_SCHEMA_V1, DECISION_SCHEMA_V1,
    FASTQ_READ_LENGTH_EXPECTATIONS_SCHEMA_V1, FASTQ_READ_LENGTH_EXPECTATIONS_SCHEMA_V2,
    GOVERNED_DATA_CATALOG_SCHEMA_V1, OBSERVATION_SCHEMA_V1, SUITE_SCHEMA_V1, SUMMARY_SCHEMA_V1,
};
pub use crate::diagnostics::BenchError;
pub use crate::model::execution::{
    benchmark_execution_spec_schema, BenchmarkAcceptanceExpectedValue, BenchmarkAcceptanceMetric,
    BenchmarkAcceptanceOperator, BenchmarkExecutionAcceptance, BenchmarkExecutionCachePolicy,
    BenchmarkExecutionCommands, BenchmarkExecutionDeterminism, BenchmarkExecutionDownstreamProbe,
    BenchmarkExecutionImage, BenchmarkExecutionInput, BenchmarkExecutionInputKind,
    BenchmarkExecutionInterface, BenchmarkExecutionMeasurement, BenchmarkExecutionOutputs,
    BenchmarkExecutionParameterEvidence, BenchmarkExecutionResources,
    BenchmarkExecutionRetryPolicy, BenchmarkExecutionRuntimePolicy, BenchmarkExecutionSourceLock,
    BenchmarkExecutionSpec,
};
pub use crate::model::graph::{BenchmarkGraphNode, BenchmarkGraphNodeKind, BenchmarkStageEdge};
pub use crate::model::interfaces::{
    benchmark_pipeline_parameter_config_schema, benchmark_stage_interface_catalog_schema,
    BenchmarkParameterValue, BenchmarkPipelineParameterConfig, BenchmarkStageDownstreamProbe,
    BenchmarkStageInterfaceCatalog, BenchmarkStageInterfaceSpec, BenchmarkStageParameterConfig,
};
pub use crate::model::lifecycle::{
    benchmark_attempt_archive_schema, benchmark_attempt_migration_ledger_schema,
    benchmark_attempt_migration_plan_schema, benchmark_attempt_schema, benchmark_attempt_v2_schema,
    benchmark_campaign_plan_schema, benchmark_cycle_status_schema,
    benchmark_cycle_status_v2_schema, benchmark_scheduler_observation_schema,
    BenchmarkArchiveFileRecord, BenchmarkArchiveSourcePaths, BenchmarkAttemptArchive,
    BenchmarkAttemptDisposition, BenchmarkAttemptDocument, BenchmarkAttemptIdentity,
    BenchmarkAttemptMigrationEntry, BenchmarkAttemptMigrationExclusion,
    BenchmarkAttemptMigrationLedger, BenchmarkAttemptMigrationLedgerEntry,
    BenchmarkAttemptMigrationPlan, BenchmarkAttemptMigrationState, BenchmarkAttemptRecord,
    BenchmarkAttemptRecordV2, BenchmarkAttemptTransition, BenchmarkAttemptV1MigrationContext,
    BenchmarkCampaignPlan, BenchmarkCycleDocument, BenchmarkCycleStatus, BenchmarkCycleStatusV2,
    BenchmarkEvidenceReference, BenchmarkJobReference, BenchmarkLifecycleStatus,
    BenchmarkRunIdentity, BenchmarkSchedulerAccounting, BenchmarkSchedulerEvidenceSource,
    BenchmarkSchedulerObservation, BenchmarkSchedulerObservationDraft,
    BenchmarkSchedulerObservedState, BenchmarkSchedulerTerminalState,
};
pub use crate::model::materialization::{
    corpus_acquisition_plan_schema, governed_data_catalog_schema,
};
pub use crate::model::observation::benchmark_observation_schema;
pub use crate::model::observation::MetricsEnvelope;
pub use crate::model::planning::{
    benchmark_portfolio_schema, benchmark_scientific_readiness_schema,
    benchmark_stage_campaign_registry_schema, benchmark_stage_coverage_schema,
    BenchmarkCachePolicy, BenchmarkComparisonMetricTolerance, BenchmarkCorpusContract,
    BenchmarkCorpusStatus, BenchmarkEvidenceScope, BenchmarkEvidenceTier,
    BenchmarkExecutionRuntime, BenchmarkExecutionScheduler, BenchmarkMeasurementContract,
    BenchmarkPortfolio, BenchmarkProfileAssignment, BenchmarkScientificProfile,
    BenchmarkScientificReadinessCorpus, BenchmarkScientificReadinessDomain,
    BenchmarkScientificReadinessReport, BenchmarkScientificReadinessStage,
    BenchmarkScientificReadinessStatus, BenchmarkScientificReadinessSummary,
    BenchmarkStageCampaignRegistry, BenchmarkStageCampaignSpec, BenchmarkStageCoverage,
    BenchmarkStageCoverageRow, BenchmarkStageCoverageSummary, BenchmarkStageDefinitionStatus,
    BenchmarkStageExecutionStatus, BenchmarkStageInterfaceStatus,
};
pub use crate::model::protocol::{
    benchmark_case_definition_schema, benchmark_run_definition_schema, BenchmarkCaseDefinition,
    BenchmarkCaseInputLock, BenchmarkCaseQualificationReceipt, BenchmarkContractLock,
    BenchmarkCorpusRole, BenchmarkRunDefinition, BenchmarkRunState,
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
    benchmark_data_index_schema, AdmittedBamCorpus, AdmittedBamCorpusFile, AdmittedFastqCorpus,
    AdmittedFastqCorpusFile, AdmittedFastqCorpusSample, BackendComparisonSpec,
    BenchmarkBundleManifest, BenchmarkCorpusManifest, BenchmarkDataIndex, BenchmarkDataIndexFile,
    BenchmarkDataIndexRecord, BenchmarkDataIndexSummary, BenchmarkEligibility,
    BenchmarkFastqLayout, BenchmarkTemporalProvenance, CorpusAcquisitionFile,
    CorpusAcquisitionLayout, CorpusAcquisitionPlan, CorpusAcquisitionSample, CorpusDatasetSpec,
    CorpusDomain, CorpusMaterializationDisposition, CorpusScale, DataIntegrityStatus,
    DriftScenarioSpec, FastqReadLengthExpectationSample, FastqReadLengthExpectations,
    GovernedDataCatalog, GovernedDataCatalogFile, GovernedDataCatalogSample, TruthSetHook,
    TruthSetStatus,
};
pub use crate::model::{BenchmarkObservation, BenchmarkSuiteSpec, BenchmarkSummary};
pub use crate::policy::{
    gate_decision_schema, GateDecision, GatePolicy, GatePolicyOverrides, GateViolation,
};
pub use crate::stats::robust_estimators::robust_stats;
