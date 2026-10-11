use super::REQUIRED_METRIC_NAMES;
use bijux_dna_bench_vcf_evidence::snapshot::VcfStageReadinessBinding;
pub(super) fn required_metric_names() -> Vec<String> {
    REQUIRED_METRIC_NAMES.iter().map(|metric| (*metric).to_string()).collect()
}
pub(super) fn contains_artifact_id(values: &[String], artifact_id: &str) -> bool {
    values.iter().any(|value| {
        value
            .split_once('=')
            .is_some_and(|(current_artifact_id, _)| current_artifact_id == artifact_id)
    })
}
pub(super) fn expected_result_id(
    binding: &bijux_dna_bench_vcf_evidence::model::expectations::VcfExpectedBenchmarkResultRow,
) -> String {
    bijux_dna_core::ids::build_asset_profile_benchmark_result_id(
        "vcf",
        &binding.corpus_id,
        &binding.stage_id,
        &binding.asset_profile_id,
        &binding.tool_id,
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
