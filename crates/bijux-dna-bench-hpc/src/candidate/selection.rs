use super::{
    declarations::{
        EXCLUSION_SUPERSEDED_REPRESENTATIVE, LOCAL_HPC_CANDIDATE_RUN_MANIFEST_SCHEMA_VERSION,
        SMALL_CANDIDATE_PROFILE_ID, SMALL_CPUS_PER_TASK_MAX, SMALL_INPUT_FOOTPRINT_MAX_BYTES,
        SMALL_MEMORY_MB_MAX, SMALL_SCRATCH_GB_MAX, SMALL_WALLTIME_MINUTES_MAX,
    },
    eligibility::{collect_eligible, increment_count},
    inputs::CandidateInputs,
    paths::path_relative_to_repo,
    projection::{candidate_sort_key, to_candidate_job},
    CandidateContext,
};
use anyhow::Result;
use bijux_dna_bench_model::{
    BenchmarkLocalHpcCandidateResourceCeiling as LocalHpcCandidateResourceCeiling,
    BenchmarkLocalHpcCandidateRunManifest as LocalHpcCandidateRunManifest,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
pub(super) fn select_candidate_run_rows(
    context: &CandidateContext<'_>,
    absolute_output_path: &Path,
    inputs: &CandidateInputs,
) -> Result<LocalHpcCandidateRunManifest> {
    let repo_root = context.repo_root;
    let admitted = collect_eligible(context, inputs)?;
    let mut eligible = admitted.rows;
    let benchmark_job_count = admitted.benchmark_job_count;
    let root_benchmark_job_count = admitted.root_benchmark_job_count;
    let mut exclusion_reason_counts = admitted.exclusions;
    eligible.sort_by_key(candidate_sort_key);
    let eligible_small_job_count = eligible.len();
    let mut selected_rows = Vec::new();
    let mut selected_groups = BTreeSet::<String>::new();
    for row in eligible {
        if !selected_groups.insert(row.representative_group_id.clone()) {
            increment_count(&mut exclusion_reason_counts, EXCLUSION_SUPERSEDED_REPRESENTATIVE);
            continue;
        }
        selected_rows.push(to_candidate_job(row));
    }
    selected_rows.sort_by(|left, right| {
        left.representative_group_id
            .cmp(&right.representative_group_id)
            .then_with(|| left.result_id.cmp(&right.result_id))
    });

    let mut selected_domain_counts = BTreeMap::<String, usize>::new();
    let mut selected_execution_mode_counts = BTreeMap::<String, usize>::new();
    let mut selected_resolution_kind_counts = BTreeMap::<String, usize>::new();
    let mut selected_staged_input_count = 0usize;
    let mut selected_staged_input_bytes = 0u64;
    for row in &selected_rows {
        *selected_domain_counts.entry(row.domain.clone()).or_default() += 1;
        *selected_execution_mode_counts.entry(row.execution_mode.clone()).or_default() += 1;
        *selected_resolution_kind_counts.entry(row.resolution_kind.clone()).or_default() += 1;
        selected_staged_input_count += row.staged_input_count;
        selected_staged_input_bytes += row.staged_input_total_bytes;
    }

    let behavior = super::behavior::candidate_behavior(&selected_rows);

    Ok(LocalHpcCandidateRunManifest {
        schema_version: LOCAL_HPC_CANDIDATE_RUN_MANIFEST_SCHEMA_VERSION.to_string(),
        output_path: path_relative_to_repo(repo_root, absolute_output_path),
        selection_profile_id: SMALL_CANDIDATE_PROFILE_ID.to_string(),
        evaluated_job_count: inputs.jobs.len(),
        benchmark_job_count,
        root_benchmark_job_count,
        eligible_small_job_count,
        representative_group_count: selected_groups.len(),
        selected_job_count: selected_rows.len(),
        selected_staged_input_count,
        selected_staged_input_bytes,
        selected_domain_counts,
        selected_execution_mode_counts,
        selected_resolution_kind_counts,
        exclusion_reason_counts,
        resource_ceiling: LocalHpcCandidateResourceCeiling {
            cpus_per_task_max: SMALL_CPUS_PER_TASK_MAX,
            memory_mb_max: SMALL_MEMORY_MB_MAX,
            walltime_minutes_max: SMALL_WALLTIME_MINUTES_MAX,
            scratch_gb_max: SMALL_SCRATCH_GB_MAX,
            staged_input_bytes_max: SMALL_INPUT_FOOTPRINT_MAX_BYTES,
        },
        behavior,
        rows: selected_rows,
    })
}
