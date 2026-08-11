//! Owner: bijux-dna-bench
//! Immutable admission of completed benchmark execution outputs.

mod model;
mod validation;

pub use model::{
    BenchmarkExecutionAdmission, BenchmarkExecutionAdmissionPublication, BenchmarkMetricAssessment,
    BenchmarkOutputFileEvidence, BENCHMARK_EXECUTION_ADMISSION_SCHEMA_V1,
};
pub use validation::{admit_benchmark_execution_outputs, revalidate_benchmark_execution_outputs};
