use super::{NO_VALUE, REQUIRED_LIKELIHOOD_FIELDS, REQUIRED_METRIC_NAMES};
use bijux_dna_bench_vcf_evidence::snapshot::VcfStageReadinessBinding;
use std::collections::BTreeSet;
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
pub(super) fn retained_result_id(binding: &VcfStageReadinessBinding) -> String {
    bijux_dna_core::ids::build_asset_profile_benchmark_result_id(
        "vcf",
        &binding.retained_row.corpus_id,
        &binding.retained_row.stage_id,
        &binding.retained_row.asset_profile_id,
        &binding.retained_row.tool_id,
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
pub(super) fn required_likelihood_fields() -> Vec<String> {
    REQUIRED_LIKELIHOOD_FIELDS.iter().map(|value| (*value).to_string()).collect()
}
pub(super) fn likelihood_field_sets_match(left: &[String], right: &[String]) -> bool {
    left.iter().collect::<BTreeSet<_>>() == right.iter().collect::<BTreeSet<_>>()
}
pub(super) fn no_value_string() -> String {
    NO_VALUE.to_string()
}
