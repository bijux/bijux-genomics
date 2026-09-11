use super::eligibility::EligibleCandidateJob;
use bijux_dna_bench_model::{
    BenchmarkLocalHpcCandidateJob as LocalHpcCandidateJob,
    BenchmarkLocalHpcCandidateResourceHint as LocalHpcCandidateResourceHint,
    BenchmarkLocalHpcCandidateStopCondition as LocalHpcCandidateStopCondition,
};
type CandidateOrder = (String, u64, u32, u32, u32, String, String, String);
pub(super) fn candidate_sort_key(row: &EligibleCandidateJob<'_>) -> CandidateOrder {
    (
        row.representative_group_id.clone(),
        row.staged_input_total_bytes,
        row.resource_hint.cpus_per_task,
        row.resource_hint.memory_mb,
        row.walltime_minutes,
        row.submit_job.stage_id.clone(),
        row.submit_job.tool_id.clone(),
        row.result_id.clone(),
    )
}
pub(super) fn to_candidate_job(row: EligibleCandidateJob<'_>) -> LocalHpcCandidateJob {
    let stop_conditions = candidate_stop_conditions(&row);
    let submit_job = row.submit_job;
    LocalHpcCandidateJob {
        representative_group_id: row.representative_group_id,
        selection_reason: row.selection_reason,
        job_id_local: submit_job.job_id_local.clone(),
        result_id: row.result_id,
        domain: submit_job.domain.clone(),
        stage_id: submit_job.stage_id.clone(),
        tool_id: submit_job.tool_id.clone(),
        corpus_id: submit_job.corpus_id.clone(),
        asset_profile_id: submit_job.asset_profile_id.clone(),
        execution_mode: row.execution.execution_mode.clone(),
        execution_mode_summary: row.execution.execution_mode_summary.clone(),
        resolution_kind: row.execution.resolution_kind.clone(),
        resolution_target: row.execution.resolution_target.clone(),
        command_entrypoint: row.execution.command_entrypoint.clone(),
        expected_stage_result_manifest_path: row.stage_result_manifest_path,
        expected_outputs: submit_job.outputs.clone(),
        resource_hint: LocalHpcCandidateResourceHint {
            cpus_per_task: row.resource_hint.cpus_per_task,
            memory_mb: row.resource_hint.memory_mb,
            time_limit: row.resource_hint.time_limit.clone(),
            walltime_minutes: row.walltime_minutes,
            scratch_gb: row.resource_hint.scratch_gb,
        },
        staged_input_count: row.staged_inputs.len(),
        staged_input_total_bytes: row.staged_input_total_bytes,
        staged_inputs: row.staged_inputs,
        stop_conditions,
    }
}
fn candidate_stop_conditions(
    row: &EligibleCandidateJob<'_>,
) -> Vec<LocalHpcCandidateStopCondition> {
    let submit_job = row.submit_job;
    let manifest_path = &row.stage_result_manifest_path;
    let result_root = &row.result_root;
    vec![
        LocalHpcCandidateStopCondition {
            condition_id: "missing_stage_result_manifest".to_string(),
            source_surface: "local_hpc_job_completion".to_string(),
            detail: format!(
                "stop the candidate run if `{manifest_path}` is missing after `{}` finishes under `{result_root}`",
                submit_job.job_id_local
            ),
        },
        LocalHpcCandidateStopCondition {
            condition_id: "failed_stage_result_manifest".to_string(),
            source_surface: "local_hpc_job_completion".to_string(),
            detail: format!(
                "stop the candidate run if `{manifest_path}` records runtime status `failed` for `{}`",
                submit_job.job_id_local
            ),
        },
        LocalHpcCandidateStopCondition {
            condition_id: "stale_partial_outputs".to_string(),
            source_surface: "local_hpc_job_completion".to_string(),
            detail: format!(
                "stop the candidate run if `{}` reports success but any declared output under `{result_root}` is missing or mismatched",
                submit_job.job_id_local
            ),
        },
        LocalHpcCandidateStopCondition {
            condition_id: "unexpected_collection_status".to_string(),
            source_surface: "local_hpc_result_collection_simulation".to_string(),
            detail: format!(
                "stop the candidate run if result collection would classify `{}` as anything other than `complete` once its declared outputs land",
                submit_job.job_id_local
            ),
        },
    ]
}
