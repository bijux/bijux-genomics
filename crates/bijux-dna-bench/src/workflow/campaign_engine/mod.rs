//! Owner: bijux-dna-bench
//! Registry-driven benchmark planning and execution boundary.

mod aggregation;
mod persistence;
mod planning;
mod plugins;
mod reporting;

pub use aggregation::{
    aggregate_benchmark_case_observations, validate_benchmark_case_aggregations,
};
pub use bijux_dna_bench_model::{
    BenchmarkAdmittedCaseObservationSetV1, BenchmarkAdmittedCaseObservationV2,
    BenchmarkCaseAggregationSetV1, BenchmarkCaseAggregationV2, BenchmarkCaseDefinitionV1,
    BenchmarkCaseExecutionEvidenceV1, BenchmarkCaseMatrixV2, BenchmarkCaseMeasurementPlanV1,
    BenchmarkCaseRepetitionObservationV1, BenchmarkCaseSciencePlanV1, BenchmarkCorpusSnapshotV1,
    BenchmarkDataArtifactSnapshotV1, BenchmarkDataSnapshotRoleV1, BenchmarkDataSnapshotV1,
    BENCHMARK_ADMITTED_CASE_OBSERVATION_SCHEMA_V2,
    BENCHMARK_ADMITTED_CASE_OBSERVATION_SET_SCHEMA_V1, BENCHMARK_CASE_AGGREGATION_SCHEMA_V2,
    BENCHMARK_CASE_AGGREGATION_SET_SCHEMA_V1, BENCHMARK_CASE_MATRIX_SCHEMA_V2,
};
pub use persistence::{
    aggregate_benchmark_admitted_case_observation_set, benchmark_case_matrix_sha256,
    build_benchmark_admitted_case_observation_set, ensure_benchmark_admitted_case_observation_set,
    ensure_benchmark_case_aggregation_set, ensure_benchmark_case_matrix,
    load_benchmark_admitted_case_observation_set, load_benchmark_case_aggregation_set,
    load_benchmark_case_matrix, validate_benchmark_admitted_case_observation_set,
    validate_benchmark_case_aggregation_set,
};
pub use planning::{
    plan_benchmark_case_matrix, validate_benchmark_case_matrix, BenchmarkCaseMatrixRequest,
};
pub use plugins::{BenchmarkStageNormalization, BenchmarkStagePluginRegistry};
pub use reporting::build_benchmark_stage_report_from_case_aggregations;
