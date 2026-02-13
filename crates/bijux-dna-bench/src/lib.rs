//! Owner: bijux-dna-bench
//! Public API for benchmark loading, summarization, comparison, and gating.
//! Contract: inputs are typed, outputs are deterministic, and raw JSON is confined to repo/artifacts.

mod artifacts;
mod repo;
mod summary;
use std::path::PathBuf;

pub use bijux_dna_bench_model::{
    AnalysisRequirements, BenchError, BenchmarkDecision, BenchmarkObservation, BenchmarkSuiteSpec,
    BenchmarkSummary, DatasetSpec, DecisionRationale, DiversityRequirements, GateDecision,
    GatePolicy, GatePolicyOverrides, GateViolation, MetricSummary, MetricsEnvelope,
    ReplicatePolicy, StratificationRequirement, SummaryRow,
};
pub use summary::{compare, gate, load_suite, summarize, BenchRunOptions};

#[must_use]
pub fn bench_data_dir() -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("workspace root")
        .to_path_buf();
    bijux_dna_infra::bench_data_dir(&root)
}

#[must_use]
pub fn bench_suites_dir() -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("workspace root")
        .to_path_buf();
    bijux_dna_infra::bench_suites_dir(&root)
}
