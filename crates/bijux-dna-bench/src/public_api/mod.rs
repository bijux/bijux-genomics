use std::path::PathBuf;

pub use bijux_dna_bench_model::{
    AnalysisRequirements, BenchError, BenchmarkDecision, BenchmarkObservation, BenchmarkSuiteSpec,
    BenchmarkSummary, DatasetSpec, DecisionRationale, DiversityRequirements, GateDecision,
    GatePolicy, GatePolicyOverrides, GateViolation, MetricSummary, MetricsEnvelope,
    ReplicatePolicy, StratificationRequirement, SummaryRow,
};
pub use crate::summary::{compare, gate, load_suite, summarize, BenchRunOptions};

#[must_use]
pub fn bench_data_dir() -> PathBuf {
    crate::repo::resolve_repo_root()
        .map(|root| bijux_dna_infra::bench_data_dir(&root))
        .unwrap_or_else(|_| PathBuf::from("bench/data"))
}

#[must_use]
pub fn bench_suites_dir() -> PathBuf {
    crate::repo::resolve_repo_root()
        .map(|root| bijux_dna_infra::bench_suites_dir(&root))
        .unwrap_or_else(|_| PathBuf::from("bench/data/suites"))
}
