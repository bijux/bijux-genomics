//! Owner: bijux-dna-benchmark
//! Public API for benchmark loading, summarization, comparison, and gating.
//! Contract: inputs are typed, outputs are deterministic, and raw JSON is confined to repo/artifacts.

mod artifacts;
mod legacy;
mod repo;
mod summary;

pub use bijux_dna_benchmark_model::{
    AnalysisRequirements, BenchError, BenchmarkDecision, BenchmarkObservation, BenchmarkSuiteSpec,
    BenchmarkSummary, DatasetSpec, DecisionRationale, DiversityRequirements, GateDecision,
    GatePolicy, GatePolicyOverrides, GateViolation, MetricSummary, MetricsEnvelope,
    ReplicatePolicy, StratificationRequirement, SummaryRow,
};
pub use legacy::fastq::{
    benchmark_runs, write_benchmark_exports, BenchmarkSummary as LegacyBenchmarkSummary,
    RunBenchmarkRecord, ToolRanking,
};
pub use summary::{compare, gate, load_suite, summarize, BenchRunOptions};
