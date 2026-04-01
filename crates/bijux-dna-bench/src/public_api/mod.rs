//! Owner: bijux-dna-bench
//! Curated public API for benchmark workflows and model contracts.

pub use crate::repo::{bench_data_dir, bench_suites_dir};
pub use crate::workflow::{compare, gate, load_suite, summarize, BenchRunOptions};
pub use bijux_dna_bench_model::{
    AnalysisRequirements, BenchError, BenchmarkDecision, BenchmarkObservation, BenchmarkSuiteSpec,
    BenchmarkSummary, DatasetSpec, DecisionRationale, DiversityRequirements, GateDecision,
    GatePolicy, GatePolicyOverrides, GateViolation, MetricSummary, MetricsEnvelope,
    ReplicatePolicy, StratificationRequirement, SummaryRow,
};
