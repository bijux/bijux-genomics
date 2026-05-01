//! Owner: bijux-dna-bench-model
//! Typed models for bench (suite, observation, summary, decision).
//! Must not perform IO or depend on repo/policy/compare logic.

pub mod corpus;
pub mod decision;
pub mod graph;
pub mod observation;
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
pub use observation::BenchmarkObservation;
#[allow(unused_imports)]
pub use suite::{BenchmarkParamBinding, BenchmarkStageSpec, BenchmarkSuiteSpec};
#[allow(unused_imports)]
pub use summary::{BenchmarkSummary, MetricSummary, SummaryRow, SummaryStratum};
