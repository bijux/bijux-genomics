//! Owner: bijux-bench
//! Public API for benchmark loading, summarization, comparison, and gating.
//! Contract: inputs are typed, outputs are deterministic, and raw JSON is confined to repo/artifacts.

mod artifacts;
mod compare;
mod contract;
mod error;
mod model;
mod policy;
mod repo;
mod stats;

mod summarize;

pub use error::BenchError;
pub use model::decision::DecisionRationale;
pub use model::observation::MetricsEnvelope;
pub use model::suite::{DatasetSpec, ReplicatePolicy};
pub use model::summary::{MetricSummary, SummaryRow};
pub use model::{BenchmarkDecision, BenchmarkObservation, BenchmarkSuiteSpec, BenchmarkSummary};
pub use policy::{GateDecision, GatePolicy, GatePolicyOverrides, GateViolation};
pub use summarize::{compare, gate, load_suite, summarize, BenchRunOptions};
