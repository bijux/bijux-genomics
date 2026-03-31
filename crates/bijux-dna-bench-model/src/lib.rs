//! Owner: bijux-dna-bench-model
//! Public API for benchmark models, policies, and summarization.

pub mod compare;
pub mod contract;
mod error;
mod model;
pub mod policy;
pub mod stats;

pub use error::BenchError;
pub use model::decision::DecisionRationale;
pub use model::{
    AnalysisRequirements, BenchmarkDecision, BenchmarkGraphNode, BenchmarkGraphNodeKind,
    BenchmarkObservation, BenchmarkParamBinding, BenchmarkStageEdge, BenchmarkStageSpec,
    BenchmarkSuiteSpec, BenchmarkSummary, DatasetSpec, DiversityRequirements, MetricSummary,
    MetricsEnvelope, ReplicatePolicy, StratificationRequirement, SummaryRow, SummaryStratum,
};
pub use policy::{GateDecision, GatePolicy, GatePolicyOverrides, GateViolation};
pub use stats::robust_estimators::robust_stats;
