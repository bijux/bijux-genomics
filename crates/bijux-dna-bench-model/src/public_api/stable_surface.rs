//! Owner: bijux-dna-bench-model
//! Stable crate-root re-exports.

pub use crate::compare;
pub use crate::contract;
pub use crate::contract::{
    BENCHMARK_ATTEMPT_SCHEMA_V1, BENCHMARK_CYCLE_STATUS_SCHEMA_V1,
    BENCHMARK_PIPELINE_PARAMETERS_SCHEMA_V1, BENCHMARK_PORTFOLIO_SCHEMA_V1,
    BENCHMARK_REPORT_ADMISSION_SCHEMA_V1, BENCHMARK_STAGE_CAMPAIGNS_SCHEMA_V1,
    BENCHMARK_STAGE_INTERFACES_SCHEMA_V1, BENCHMARK_STAGE_REPORT_SCHEMA_V1,
};
pub use crate::diagnostics::BenchError;
pub use crate::model::decision::DecisionRationale;
pub use crate::model::graph::{BenchmarkGraphNode, BenchmarkGraphNodeKind, BenchmarkStageEdge};
pub use crate::model::interfaces::{
    BenchmarkParameterValue, BenchmarkPipelineParameterConfig, BenchmarkStageDownstreamProbe,
    BenchmarkStageInterfaceCatalog, BenchmarkStageInterfaceSpec, BenchmarkStageParameterConfig,
};
pub use crate::model::lifecycle::{
    BenchmarkAttemptRecord, BenchmarkCycleStatus, BenchmarkJobReference, BenchmarkLifecycleStatus,
};
pub use crate::model::observation::MetricsEnvelope;
pub use crate::model::planning::{
    BenchmarkCachePolicy, BenchmarkComparisonMetricTolerance, BenchmarkCorpusContract,
    BenchmarkCorpusStatus, BenchmarkEvidenceScope, BenchmarkEvidenceTier,
    BenchmarkExecutionRuntime, BenchmarkExecutionScheduler, BenchmarkMeasurementContract,
    BenchmarkPortfolio, BenchmarkProfileAssignment, BenchmarkScientificProfile,
    BenchmarkStageCampaignRegistry, BenchmarkStageCampaignSpec,
};
pub use crate::model::report::{
    AdmittedBenchmarkStageReport, BenchmarkComparisonMetric, BenchmarkComparisonStatus,
    BenchmarkCrossToolComparison, BenchmarkMetricDistribution, BenchmarkReportComparability,
    BenchmarkReportDeterminism, BenchmarkReportDownstreamProbe, BenchmarkReportImage,
    BenchmarkReportInputLock, BenchmarkReportSlurmJob, BenchmarkReportStageInterface,
    BenchmarkReportVerdict, BenchmarkScientificValue, BenchmarkStageReport,
    BenchmarkStageReportDesign, BenchmarkToolReport,
};
pub use crate::model::suite::{
    AnalysisRequirements, BenchmarkParamBinding, BenchmarkStageSpec, DatasetSpec,
    DiversityRequirements, ReplicatePolicy, StratificationRequirement,
};
pub use crate::model::summary::{MetricSummary, SummaryRow, SummaryStratum};
pub use crate::model::{
    BackendComparisonSpec, BenchmarkBundleManifest, BenchmarkCorpusManifest, CorpusDatasetSpec,
    CorpusDomain, CorpusScale, DriftScenarioSpec, TruthSetHook, TruthSetStatus,
};
pub use crate::model::{
    BenchmarkDecision, BenchmarkObservation, BenchmarkSuiteSpec, BenchmarkSummary,
};
pub use crate::policy::{GateDecision, GatePolicy, GatePolicyOverrides, GateViolation};
pub use crate::stats::robust_estimators::robust_stats;
