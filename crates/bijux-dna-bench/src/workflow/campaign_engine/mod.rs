//! Owner: bijux-dna-bench
//! Registry-driven benchmark planning and execution boundary.

mod aggregation;
mod planning;

pub use aggregation::aggregate_benchmark_case_observations;
pub use bijux_dna_bench_model::{
    BenchmarkAdmittedCaseObservationV1, BenchmarkCaseAggregationV1, BenchmarkCaseDefinitionV1,
    BenchmarkCaseMatrixV1, BenchmarkCaseMeasurementPlanV1, BenchmarkCaseRepetitionObservationV1,
    BenchmarkCaseSciencePlanV1, BenchmarkCorpusSnapshotV1,
    BENCHMARK_ADMITTED_CASE_OBSERVATION_SCHEMA_V1, BENCHMARK_CASE_AGGREGATION_SCHEMA_V1,
    BENCHMARK_CASE_MATRIX_SCHEMA_V1,
};
pub use planning::{
    plan_benchmark_case_matrix, validate_benchmark_case_matrix, BenchmarkCaseMatrixRequest,
};
