//! Owner: bijux-dna-bench
//! Registry-driven benchmark planning and execution boundary.

mod aggregation;
mod planning;
mod reporting;

pub use aggregation::{
    aggregate_benchmark_case_observations, validate_benchmark_case_aggregations,
};
pub use bijux_dna_bench_model::{
    BenchmarkAdmittedCaseObservationV2, BenchmarkCaseAggregationV2, BenchmarkCaseDefinitionV1,
    BenchmarkCaseExecutionEvidenceV1, BenchmarkCaseMatrixV1, BenchmarkCaseMeasurementPlanV1,
    BenchmarkCaseRepetitionObservationV1, BenchmarkCaseSciencePlanV1, BenchmarkCorpusSnapshotV1,
    BENCHMARK_ADMITTED_CASE_OBSERVATION_SCHEMA_V2, BENCHMARK_CASE_AGGREGATION_SCHEMA_V2,
    BENCHMARK_CASE_MATRIX_SCHEMA_V1,
};
pub use planning::{
    plan_benchmark_case_matrix, validate_benchmark_case_matrix, BenchmarkCaseMatrixRequest,
};
pub use reporting::build_benchmark_stage_report_from_case_aggregations;
