//! Owner: bijux-dna-bench-model
//! Typed models for bench (suite, observation, summary, decision).
//! Must not perform IO or depend on repo/policy/compare logic.

pub mod corpus;
pub mod decision;
pub mod graph;
pub mod interfaces;
pub mod lifecycle;
pub mod observation;
pub mod planning;
pub mod suite;
pub mod summary;

#[allow(unused_imports)]
pub use corpus::{
    BackendComparisonSpec, BenchmarkBundleManifest, BenchmarkCorpusManifest, CorpusDatasetSpec,
    CorpusDomain, CorpusScale, DriftScenarioSpec, TruthSetHook, TruthSetStatus,
};
pub use decision::BenchmarkDecision;
#[allow(unused_imports)]
pub use graph::{BenchmarkGraphNode, BenchmarkGraphNodeKind, BenchmarkStageEdge};
#[allow(unused_imports)]
pub use interfaces::{
    BenchmarkParameterValue, BenchmarkPipelineParameterConfig, BenchmarkStageDownstreamProbe,
    BenchmarkStageInterfaceCatalog, BenchmarkStageInterfaceSpec, BenchmarkStageParameterConfig,
};
#[allow(unused_imports)]
pub use lifecycle::{
    BenchmarkAttemptRecord, BenchmarkCycleStatus, BenchmarkJobReference, BenchmarkLifecycleStatus,
};
pub use observation::BenchmarkObservation;
#[allow(unused_imports)]
pub use planning::{
    BenchmarkCachePolicy, BenchmarkComparisonMetricTolerance, BenchmarkCorpusContract,
    BenchmarkCorpusStatus, BenchmarkEvidenceScope, BenchmarkEvidenceTier,
    BenchmarkExecutionRuntime, BenchmarkExecutionScheduler, BenchmarkMeasurementContract,
    BenchmarkPortfolio, BenchmarkProfileAssignment, BenchmarkScientificProfile,
    BenchmarkStageCampaignRegistry, BenchmarkStageCampaignSpec,
};
#[allow(unused_imports)]
pub use suite::{BenchmarkParamBinding, BenchmarkStageSpec, BenchmarkSuiteSpec};
#[allow(unused_imports)]
pub use summary::{BenchmarkSummary, MetricSummary, SummaryRow, SummaryStratum};
