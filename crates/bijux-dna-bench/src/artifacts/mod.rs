//! Owner: bijux-dna-bench
//! Artifact writers for bench.

pub mod admission;
pub mod writer;

pub use admission::{
    admit_benchmark_execution_outputs, BenchmarkExecutionAdmission,
    BenchmarkExecutionAdmissionPublication, BenchmarkMetricAssessment, BenchmarkOutputFileEvidence,
    BENCHMARK_EXECUTION_ADMISSION_SCHEMA_V1,
};
pub use writer::{
    read_observations_jsonl, write_decision_json, write_decisions_json, write_observations_jsonl,
    write_summary_json, WriteMode,
};
