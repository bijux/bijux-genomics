//! Owner: bijux-dna-bench
//! Registry-driven benchmark planning and execution boundary.

mod model;
mod planning;

pub use model::{
    BenchmarkCaseDefinitionV1, BenchmarkCaseMatrixV1, BenchmarkCaseMeasurementPlanV1,
    BenchmarkCaseSciencePlanV1, BenchmarkCorpusSnapshotV1, BENCHMARK_CASE_MATRIX_SCHEMA_V1,
};
pub use planning::{plan_benchmark_case_matrix, BenchmarkCaseMatrixRequest};
