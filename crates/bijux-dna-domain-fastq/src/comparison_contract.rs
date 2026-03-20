use bijux_dna_core::ids::StageId;

const BENCHMARK_COMPARISON_ARTIFACTS: &[&str] = &[
    "benchmark_cohort_json",
    "stage_tool_comparison_json",
    "stage_tool_normalization_json",
];

#[must_use]
pub fn comparison_artifact_ids_for_stage(stage_id: &StageId) -> Vec<&'static str> {
    if crate::benchmark_scenarios_for_stage(stage_id).is_empty() {
        Vec::new()
    } else {
        BENCHMARK_COMPARISON_ARTIFACTS.to_vec()
    }
}

#[must_use]
pub fn benchmark_comparison_artifact_ids() -> Vec<&'static str> {
    BENCHMARK_COMPARISON_ARTIFACTS.to_vec()
}
