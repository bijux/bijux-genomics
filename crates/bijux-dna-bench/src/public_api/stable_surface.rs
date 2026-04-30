//! Owner: bijux-dna-bench
//! Stable public benchmark surface re-exported by the crate root.

pub use crate::repo::{bench_corpora_dir, bench_data_dir, bench_suites_dir};
pub use crate::workflow::{
    compare, gate, load_corpus_catalog, load_corpus_manifest, load_suite, summarize,
    BenchRunOptions,
};
pub use bijux_dna_bench_model::{
    AnalysisRequirements, BenchError, BenchmarkCorpusManifest, BenchmarkDecision,
    BenchmarkObservation, BenchmarkSuiteSpec, BenchmarkSummary, CorpusDatasetSpec, CorpusDomain,
    CorpusScale, DatasetSpec, DecisionRationale, DiversityRequirements, GateDecision, GatePolicy,
    GatePolicyOverrides, GateViolation, MetricSummary, MetricsEnvelope, ReplicatePolicy,
    StratificationRequirement, SummaryRow,
};
