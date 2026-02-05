//! Owner: bijux-benchmark-model
//! Public API for benchmark models, policies, and summarization.

pub mod compare;
pub mod contract;
mod error;
mod model;
pub mod policy;
pub mod stats;

pub use error::BenchError;
pub use model::decision::DecisionRationale;
pub use model::observation::MetricsEnvelope;
pub use model::suite::{
    AnalysisRequirements, DatasetSpec, DiversityRequirements, ReplicatePolicy,
    StratificationRequirement,
};
pub use model::summary::{MetricSummary, SummaryRow, SummaryStratum};
pub use model::{BenchmarkDecision, BenchmarkObservation, BenchmarkSuiteSpec, BenchmarkSummary};
pub use policy::{GateDecision, GatePolicy, GatePolicyOverrides, GateViolation};
pub use stats::robust::robust_stats;
