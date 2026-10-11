use bijux_dna_bench_vcf_evidence::model::scope::VcfActiveStageToolMatrixRow;
pub const DEFAULT_VCF_ACTIVE_STAGE_TOOL_MATRIX_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/vcf-active-stage-tool-matrix.tsv";
pub(super) const VCF_ACTIVE_STAGE_TOOL_MATRIX_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_active_stage_tool_matrix.v1";
pub(super) const ACTIVE_SCOPE_STATE: &str = "active";
pub(super) const COMPLETE_SCOPE_STATE: &str = "complete";
pub(super) const REMOVED_FROM_SCOPE_STATE: &str = "removed_from_scope";
pub(super) const ACTIVE_SCOPE_DETAIL: &str = "active";
pub(super) const COMPLETE_SCOPE_DETAIL: &str = "complete";
pub(super) const LIFECYCLE_NOT_ACTIVE_SCOPE_DETAIL: &str = "lifecycle_not_active";
pub(super) const BENCHMARK_NOT_READY_SCOPE_DETAIL: &str = "benchmark_not_ready";
pub(super) const ACTIVE_SCOPE_PROOF_PATH: &str =
    "artifacts/benchmark-smoke/readiness/all-domains/active-stage-tool-matrix.tsv";
pub(super) const COMPLETE_SCOPE_PROOF_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/vcf-expected-benchmark-results.tsv";
pub(super) const LIFECYCLE_NOT_ACTIVE_SCOPE_PROOF_PATH: &str =
    "artifacts/benchmark-smoke/readiness/all-domains/no-planned-rows.json";
pub(super) const BENCHMARK_NOT_READY_SCOPE_PROOF_PATH: &str =
    "artifacts/benchmark-smoke/readiness/all-domains/no-not-benchmark-ready-rows.json";

pub(super) fn count_scope_state(rows: &[VcfActiveStageToolMatrixRow], scope_state: &str) -> usize {
    rows.iter().filter(|row| row.scope_state == scope_state).count()
}
