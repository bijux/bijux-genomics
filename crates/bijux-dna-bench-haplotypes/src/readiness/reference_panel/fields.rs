use super::REQUIRED_METRIC_NAMES;
use bijux_dna_bench_vcf_evidence::snapshot::VcfStageReadinessBinding;
use std::collections::BTreeSet;
pub(super) fn retained_result_id(binding: &VcfStageReadinessBinding) -> String {
    bijux_dna_core::ids::build_asset_profile_benchmark_result_id(
        "vcf",
        &binding.retained_row.corpus_id,
        &binding.retained_row.stage_id,
        &binding.retained_row.asset_profile_id,
        &binding.retained_row.tool_id,
    )
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
pub(super) fn primary_executables_from_binding(
    row: &bijux_dna_bench_vcf_evidence::model::commands::VcfRenderedCommandRow,
) -> Vec<String> {
    let mut values = BTreeSet::new();
    for step in &row.command_steps {
        if let Some(executable) = step.argv.first() {
            values.insert(executable.clone());
        }
    }
    values.into_iter().collect()
}
pub(super) fn contains_artifact_id(values: &[String], artifact_id: &str) -> bool {
    values.iter().any(|value| value.split('=').next() == Some(artifact_id))
}
pub(super) fn required_metric_names() -> Vec<String> {
    REQUIRED_METRIC_NAMES.iter().map(|metric| (*metric).to_string()).collect()
}
