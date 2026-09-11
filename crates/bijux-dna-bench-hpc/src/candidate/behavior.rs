use super::declarations::{
    SMALL_CPUS_PER_TASK_MAX, SMALL_INPUT_FOOTPRINT_MAX_BYTES, SMALL_MEMORY_MB_MAX,
    SMALL_SCRATCH_GB_MAX, SMALL_WALLTIME_MINUTES_MAX,
};
use bijux_dna_bench_model::{
    BenchmarkLocalHpcCandidateBehavior as LocalHpcCandidateBehavior,
    BenchmarkLocalHpcCandidateJob as LocalHpcCandidateJob,
};
pub(super) fn candidate_behavior(
    selected_rows: &[LocalHpcCandidateJob],
) -> LocalHpcCandidateBehavior {
    let behavior = LocalHpcCandidateBehavior {
        only_benchmark_result_jobs: selected_rows.iter().all(|row| !row.result_id.is_empty()),
        only_dependency_free_jobs: true,
        only_known_assets: selected_rows.iter().all(|row| !row.staged_inputs.is_empty()),
        only_known_execution_modes: selected_rows.iter().all(|row| {
            !row.execution_mode.trim().is_empty() && row.execution_mode != "unclassified"
        }),
        only_available_execution_resolution: selected_rows
            .iter()
            .all(|row| !row.resolution_kind.contains("unavailable")),
        only_small_resource_envelopes: selected_rows.iter().all(|row| {
            row.resource_hint.cpus_per_task <= SMALL_CPUS_PER_TASK_MAX
                && row.resource_hint.memory_mb <= SMALL_MEMORY_MB_MAX
                && row.resource_hint.walltime_minutes <= SMALL_WALLTIME_MINUTES_MAX
                && row.resource_hint.scratch_gb <= SMALL_SCRATCH_GB_MAX
        }),
        only_small_input_footprints: selected_rows
            .iter()
            .all(|row| row.staged_input_total_bytes <= SMALL_INPUT_FOOTPRINT_MAX_BYTES),
        selected_rows_keep_expected_outputs: selected_rows
            .iter()
            .all(|row| !row.expected_outputs.is_empty()),
        selected_rows_keep_stop_conditions: selected_rows
            .iter()
            .all(|row| !row.stop_conditions.is_empty()),
        vcf_rows_excluded_for_unknown_execution_modes: selected_rows.iter().all(|row| {
            row.domain != "vcf"
                || (!row.execution_mode.trim().is_empty() && row.execution_mode != "unclassified")
        }),
        proven: false,
    };
    LocalHpcCandidateBehavior {
        proven: behavior.only_benchmark_result_jobs
            && behavior.only_dependency_free_jobs
            && behavior.only_known_assets
            && behavior.only_known_execution_modes
            && behavior.only_available_execution_resolution
            && behavior.only_small_resource_envelopes
            && behavior.only_small_input_footprints
            && behavior.selected_rows_keep_expected_outputs
            && behavior.selected_rows_keep_stop_conditions
            && behavior.vcf_rows_excluded_for_unknown_execution_modes,
        ..behavior
    }
}
