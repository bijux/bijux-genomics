//! Owner: bijux-dna-bench-model
//! Typed models for bench (suite, observation, summary, decision).
//! Must not perform IO or depend on repo/policy/compare logic.

pub mod admission;
pub mod campaign_engine;
pub mod corpus;
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
pub mod publication_registry;
pub mod replay;
pub mod report;
pub mod stage_result;
pub mod suite;
pub mod summary;

#[allow(unused_imports)]
pub use admission::{
    benchmark_execution_admission_schema, BenchmarkExecutionAdmission,
    BenchmarkExecutionAdmissionPublication, BenchmarkMetricAssessment, BenchmarkOutputFileEvidence,
};
#[allow(unused_imports)]
pub use campaign_engine::{
    benchmark_admitted_case_observation_schema, benchmark_case_aggregation_schema,
    benchmark_case_matrix_schema, BenchmarkAdmittedCaseObservationV1, BenchmarkCaseAggregationV1,
    BenchmarkCaseDefinitionV1, BenchmarkCaseMatrixV1, BenchmarkCaseMeasurementPlanV1,
    BenchmarkCaseRepetitionObservationV1, BenchmarkCaseSciencePlanV1, BenchmarkCorpusSnapshotV1,
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
    benchmark_campaign_plan_schema, benchmark_scheduler_observation_schema,
    BenchmarkArchiveFileRecord, BenchmarkArchiveSourcePaths, BenchmarkAttemptArchive,
    BenchmarkAttemptDisposition, BenchmarkAttemptDocument, BenchmarkAttemptIdentity,
    BenchmarkAttemptMigrationEntry, BenchmarkAttemptMigrationExclusion,
    BenchmarkAttemptMigrationLedger, BenchmarkAttemptMigrationLedgerEntry,
    BenchmarkAttemptMigrationPlan, BenchmarkAttemptMigrationState,
    BenchmarkAttemptMigrationVerification, BenchmarkAttemptMigrationVerificationEntry,
    BenchmarkAttemptRecord, BenchmarkAttemptRecordV2, BenchmarkAttemptTransition,
    BenchmarkAttemptV1MigrationContext, BenchmarkCampaignPlan, BenchmarkCycleDocument,
    BenchmarkCycleStatus, BenchmarkCycleStatusV2, BenchmarkEvidenceReference,
    BenchmarkJobReference, BenchmarkLifecycleStatus, BenchmarkRunIdentity,
    BenchmarkSchedulerAccounting, BenchmarkSchedulerEvidenceSource, BenchmarkSchedulerObservation,
    BenchmarkSchedulerObservationDraft, BenchmarkSchedulerObservedState,
    BenchmarkSchedulerTerminalState,
};
#[allow(unused_imports)]
pub use materialization::{
    corpus_acquisition_plan_schema, corpus_materialization_publication_schema,
    governed_data_catalog_schema, CorpusAcquisitionFile, CorpusAcquisitionLayout,
    CorpusAcquisitionPlan, CorpusAcquisitionSample, CorpusMaterializationDisposition,
    CorpusMaterializationPublication, GovernedDataCatalog, GovernedDataCatalogFile,
    GovernedDataCatalogSample,
};
pub use observation::BenchmarkObservation;
#[allow(unused_imports)]
pub use planning::{
    benchmark_run_registry_schema, benchmark_stage_coverage_schema,
    benchmark_stage_universe_schema, BenchmarkCachePolicy, BenchmarkComparisonMetricTolerance,
    BenchmarkCorpusContract, BenchmarkCorpusStatus, BenchmarkDisposition, BenchmarkEvidenceScope,
    BenchmarkEvidenceTier, BenchmarkExecutionRuntime, BenchmarkExecutionScheduler,
    BenchmarkMeasurementContract, BenchmarkPortfolio, BenchmarkProfileAssignment,
    BenchmarkRunNamespacePolicy, BenchmarkRunRegistry, BenchmarkRunRegistryRow,
    BenchmarkRunRegistryState, BenchmarkRunRegistrySummary, BenchmarkScientificProfile,
    BenchmarkScientificReadinessCorpus, BenchmarkScientificReadinessDomain,
    BenchmarkScientificReadinessReport, BenchmarkScientificReadinessStage,
    BenchmarkScientificReadinessStatus, BenchmarkScientificReadinessSummary,
    BenchmarkStageCampaignRegistry, BenchmarkStageCampaignSpec, BenchmarkStageCoverage,
    BenchmarkStageCoverageRow, BenchmarkStageCoverageSummary, BenchmarkStageDefinitionStatus,
    BenchmarkStageExecutionStatus, BenchmarkStageInterfaceStatus, BenchmarkStageUniverse,
    BenchmarkStageUniverseRow, BenchmarkStageUniverseSummary,
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
