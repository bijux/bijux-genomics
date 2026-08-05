//! Owner: bijux-dna-bench-model
//! Typed models for bench (suite, observation, summary, decision).
//! Must not perform IO or depend on repo/policy/compare logic.

pub mod corpus;
pub mod decision;
pub mod execution;
pub mod graph;
pub mod interfaces;
pub mod lifecycle;
pub mod observation;
pub mod planning;
pub mod report;
pub mod suite;
pub mod summary;

#[allow(unused_imports)]
pub use corpus::{
    AdmittedBamCorpus, AdmittedBamCorpusFile, AdmittedFastqCorpus, AdmittedFastqCorpusFile,
    AdmittedFastqCorpusSample, BackendComparisonSpec, BenchmarkBundleManifest,
    BenchmarkCorpusManifest, BenchmarkFastqLayout, CorpusDatasetSpec, CorpusDomain, CorpusScale,
    DriftScenarioSpec, FastqReadLengthExpectationSample, FastqReadLengthExpectations, TruthSetHook,
    TruthSetStatus,
};
pub use decision::BenchmarkDecision;
#[allow(unused_imports)]
pub use execution::{
    benchmark_execution_spec_schema, BenchmarkAcceptanceExpectedValue, BenchmarkAcceptanceMetric,
    BenchmarkAcceptanceOperator, BenchmarkExecutionAcceptance, BenchmarkExecutionCachePolicy,
    BenchmarkExecutionCommands, BenchmarkExecutionDeterminism, BenchmarkExecutionDownstreamProbe,
    BenchmarkExecutionImage, BenchmarkExecutionInput, BenchmarkExecutionInputKind,
    BenchmarkExecutionInterface, BenchmarkExecutionMeasurement, BenchmarkExecutionOutputs,
    BenchmarkExecutionParameterEvidence, BenchmarkExecutionResources,
    BenchmarkExecutionRetryPolicy, BenchmarkExecutionRuntimePolicy, BenchmarkExecutionSourceLock,
    BenchmarkExecutionSpec,
};
#[allow(unused_imports)]
pub use graph::{BenchmarkGraphNode, BenchmarkGraphNodeKind, BenchmarkStageEdge};
#[allow(unused_imports)]
pub use interfaces::{
    BenchmarkParameterValue, BenchmarkPipelineParameterConfig, BenchmarkStageDownstreamProbe,
    BenchmarkStageInterfaceCatalog, BenchmarkStageInterfaceSpec, BenchmarkStageParameterConfig,
};
#[allow(unused_imports)]
pub use lifecycle::{
    benchmark_campaign_plan_schema, benchmark_scheduler_observation_schema,
    BenchmarkArchiveFileRecord, BenchmarkArchiveSourcePaths, BenchmarkAttemptArchive,
    BenchmarkAttemptDisposition, BenchmarkAttemptIdentity, BenchmarkAttemptMigrationEntry,
    BenchmarkAttemptMigrationExclusion, BenchmarkAttemptMigrationLedger,
    BenchmarkAttemptMigrationLedgerEntry, BenchmarkAttemptMigrationPlan,
    BenchmarkAttemptMigrationState, BenchmarkAttemptRecord, BenchmarkAttemptRecordV2,
    BenchmarkAttemptTransition, BenchmarkCampaignPlan, BenchmarkCycleStatus,
    BenchmarkCycleStatusV2, BenchmarkEvidenceReference, BenchmarkJobReference,
    BenchmarkLifecycleStatus, BenchmarkRunIdentity, BenchmarkSchedulerAccounting,
    BenchmarkSchedulerEvidenceSource, BenchmarkSchedulerObservation,
    BenchmarkSchedulerObservationDraft, BenchmarkSchedulerObservedState,
    BenchmarkSchedulerTerminalState,
};
pub use observation::BenchmarkObservation;
#[allow(unused_imports)]
pub use planning::{
    benchmark_stage_coverage_schema, BenchmarkCachePolicy, BenchmarkComparisonMetricTolerance,
    BenchmarkCorpusContract, BenchmarkCorpusStatus, BenchmarkEvidenceScope, BenchmarkEvidenceTier,
    BenchmarkExecutionRuntime, BenchmarkExecutionScheduler, BenchmarkMeasurementContract,
    BenchmarkPortfolio, BenchmarkProfileAssignment, BenchmarkScientificProfile,
    BenchmarkScientificReadinessCorpus, BenchmarkScientificReadinessDomain,
    BenchmarkScientificReadinessReport, BenchmarkScientificReadinessStage,
    BenchmarkScientificReadinessStatus, BenchmarkScientificReadinessSummary,
    BenchmarkStageCampaignRegistry, BenchmarkStageCampaignSpec, BenchmarkStageCoverage,
    BenchmarkStageCoverageRow, BenchmarkStageCoverageSummary, BenchmarkStageDefinitionStatus,
    BenchmarkStageExecutionStatus, BenchmarkStageInterfaceStatus,
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
pub use suite::{BenchmarkParamBinding, BenchmarkStageSpec, BenchmarkSuiteSpec};
#[allow(unused_imports)]
pub use summary::{BenchmarkSummary, MetricSummary, SummaryRow, SummaryStratum};
