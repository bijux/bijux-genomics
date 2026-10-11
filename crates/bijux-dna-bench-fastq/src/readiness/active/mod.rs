mod model;
pub use model::{
    FastqActiveStageToolMatrixCollection, FastqActiveStageToolMatrixReport,
    FastqActiveStageToolMatrixRow,
};
pub const DEFAULT_FASTQ_ACTIVE_STAGE_TOOL_MATRIX_PATH: &str =
    "artifacts/benchmark-smoke/readiness/fastq/fastq-active-stage-tool-matrix.tsv";
pub const FASTQ_ACTIVE_STAGE_TOOL_MATRIX_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.fastq_active_stage_tool_matrix.v1";
const ACTIVE_SCOPE_PROOF_PATH: &str =
    "artifacts/benchmark-smoke/readiness/all-domains/active-stage-tool-matrix.tsv";
const REMOVED_FROM_SCOPE_PROOF_PATH: &str =
    "artifacts/benchmark-smoke/readiness/removed-from-scope.tsv";
const BENCHMARK_READY_STATUS: &str = "benchmark_ready";
const PLANNED_SUPPORT_STATUS: &str = "planned_contract";
const DECLARED_ONLY_ADAPTER_STATUS: &str = "declared_only";
mod contract;
mod projection;
mod serialization;
pub(crate) use contract::ensure_fastq_active_stage_tool_matrix_contract;
pub use projection::collect_fastq_active_stage_tool_matrix_rows;
mod service;
pub use service::render_fastq_active_stage_tool_matrix;
