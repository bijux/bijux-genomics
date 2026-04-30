//! Owner: bijux-dna-bench-model
//! Typed models for bench (suite, observation, summary, decision).
//! Must not perform IO or depend on repo/policy/compare logic.

pub mod decision;
pub mod graph;
pub mod observation;
pub mod corpus;
pub mod suite;
pub mod summary;

pub use decision::BenchmarkDecision;
#[allow(unused_imports)]
pub use graph::{BenchmarkGraphNode, BenchmarkGraphNodeKind, BenchmarkStageEdge};
pub use observation::BenchmarkObservation;
#[allow(unused_imports)]
pub use corpus::{
    BackendComparisonSpec, BenchmarkCorpusManifest, CorpusDatasetSpec, CorpusDomain, CorpusScale, TruthSetHook,
    TruthSetStatus,
};
#[allow(unused_imports)]
pub use suite::{BenchmarkParamBinding, BenchmarkStageSpec, BenchmarkSuiteSpec};
#[allow(unused_imports)]
pub use summary::{BenchmarkSummary, MetricSummary, SummaryRow, SummaryStratum};
