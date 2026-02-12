//! Owner: bijux-dna-bench
//! Public API for benchmark loading, summarization, comparison, and gating.
//! Contract: inputs are typed, outputs are deterministic, and raw JSON is confined to repo/artifacts.

mod artifacts;
mod repo;
mod summary;

pub use bijux_dna_bench_model::{
    AnalysisRequirements, BenchError, BenchmarkDecision, BenchmarkObservation, BenchmarkSuiteSpec,
    BenchmarkSummary, DatasetSpec, DecisionRationale, DiversityRequirements, GateDecision,
    GatePolicy, GatePolicyOverrides, GateViolation, MetricSummary, MetricsEnvelope,
    ReplicatePolicy, StratificationRequirement, SummaryRow,
};
pub use summary::{compare, gate, load_suite, summarize, BenchRunOptions};
