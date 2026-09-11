use super::{NO_VALUE, REQUIRED_METRIC_NAMES};
use bijux_dna_bench_vcf_evidence::model::scope::VcfActiveStageToolMatrixRow;
pub(super) fn retained_result_id(row: &VcfActiveStageToolMatrixRow) -> String {
    bijux_dna_core::ids::build_asset_profile_benchmark_result_id(
        "vcf",
        &row.corpus_id,
        &row.stage_id,
        &row.asset_profile_id,
        &row.tool_id,
    )
}
pub(super) fn contains_artifact_id(entries: &[String], expected_id: &str) -> bool {
    entries.iter().any(|entry| artifact_id(entry) == expected_id)
}
pub(super) fn artifact_id(entry: &str) -> &str {
    entry.split_once('=').map_or(entry, |(id, _)| id)
}
pub(super) fn required_metric_names() -> Vec<String> {
    REQUIRED_METRIC_NAMES.iter().map(|value| (*value).to_string()).collect()
}
pub(super) fn no_value_string() -> String {
    NO_VALUE.to_string()
}
pub(super) fn expected_result_id(
    row: &bijux_dna_bench_vcf_evidence::model::expectations::VcfExpectedBenchmarkResultRow,
) -> String {
    bijux_dna_core::ids::build_asset_profile_benchmark_result_id(
        "vcf",
        &row.corpus_id,
        &row.stage_id,
        &row.asset_profile_id,
        &row.tool_id,
    )
}
