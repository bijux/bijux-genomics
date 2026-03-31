//! Owner: bijux-dna-bench
//! Typed models for bench (suite, observation, summary, decision).
//! Must not perform IO or depend on repo/policy/compare logic.

pub mod decision;
pub mod graph;
pub mod observation;
pub mod suite;
pub mod summary;

pub use decision::BenchmarkDecision;
#[allow(unused_imports)]
pub use graph::{BenchmarkGraphNode, BenchmarkGraphNodeKind, BenchmarkStageEdge};
pub use observation::{BenchmarkObservation, MetricsEnvelope};
pub use suite::{
    AnalysisRequirements, BenchmarkParamBinding, BenchmarkStageSpec, BenchmarkSuiteSpec,
    DatasetSpec, DiversityRequirements, ReplicatePolicy, StratificationRequirement,
};
#[allow(unused_imports)]
pub use summary::{BenchmarkSummary, MetricSummary, SummaryRow, SummaryStratum};
