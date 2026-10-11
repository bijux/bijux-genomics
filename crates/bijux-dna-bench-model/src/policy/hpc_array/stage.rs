//! Owner: stage array joins over explicit selected-job, scratch and execution observations.
use super::commands::{render_shell_command, rendered_benchmark_result_execution_argv};
use crate::{
    build_benchmark_local_hpc_array_resource_envelope,
    validate_benchmark_local_hpc_stage_benchmark_array, BenchmarkLocalAllDomainSlurmSubmitJob,
    BenchmarkLocalHpcArrayResources, BenchmarkLocalHpcExecutionResolverRow,
    BenchmarkLocalHpcScratchLayoutJob, BenchmarkLocalHpcStageBenchmarkArrayReport,
    BenchmarkLocalHpcStageBenchmarkArrayRow, BENCHMARK_LOCAL_HPC_STAGE_BENCHMARK_ARRAY_SCHEMA_V1,
};
use anyhow::{anyhow, ensure, Result};
use std::collections::BTreeMap;
/// Borrowed observations and admitted destination metadata for one stage array.
pub struct HpcStageArrayInputs<'a> {
    pub jobs: &'a [BenchmarkLocalAllDomainSlurmSubmitJob],
    pub scratch: &'a [BenchmarkLocalHpcScratchLayoutJob],
    pub executions: &'a [BenchmarkLocalHpcExecutionResolverRow],
    pub script_path: String,
    pub manifest_path: String,
    pub wrapper_log_root: String,
}
/// Join selected benchmark jobs and admit the resulting persisted array.
/// # Errors
/// Rejects empty selection, ambiguous observations, missing coverage and invalid resource envelopes.
pub fn build_hpc_stage_array_report(
    inputs: HpcStageArrayInputs<'_>,
) -> Result<BenchmarkLocalHpcStageBenchmarkArrayReport> {
    let mut scratch_by_result = BTreeMap::new();
    for job in inputs.scratch {
        if let Some(result) = job.result_id.as_deref() {
            ensure!(
                scratch_by_result.insert(result, job).is_none(),
                "HPC stage benchmark array has duplicate scratch result `{result}`"
            );
        }
    }
    let mut execution_by_tool = BTreeMap::new();
    for execution in inputs.executions {
        ensure!(
            execution_by_tool.insert(execution.tool_id.as_str(), execution).is_none(),
            "HPC stage benchmark array has duplicate execution tool `{}`",
            execution.tool_id
        );
    }
    let mut jobs = inputs
        .jobs
        .iter()
        .filter(|job| job.result_id.is_some() && job.pipeline_id.is_none() && job.node_id.is_none())
        .collect::<Vec<_>>();
    ensure!(
        !jobs.is_empty(),
        "HPC stage benchmark array requires at least one selected benchmark job"
    );
    jobs.sort_by(|left, right| left.result_id.cmp(&right.result_id));
    let rows = jobs
        .iter()
        .enumerate()
        .map(|(index, job)| {
            build_stage_benchmark_array_row(index, job, &scratch_by_result, &execution_by_tool)
        })
        .collect::<Result<Vec<_>>>()?;
    let resource_envelope =
        build_benchmark_local_hpc_array_resource_envelope(rows.iter().map(|row| &row.resources))?;
    let report = BenchmarkLocalHpcStageBenchmarkArrayReport {
        schema_version: BENCHMARK_LOCAL_HPC_STAGE_BENCHMARK_ARRAY_SCHEMA_V1.to_string(),
        script_path: inputs.script_path,
        manifest_path: inputs.manifest_path,
        wrapper_log_root: inputs.wrapper_log_root,
        array_spec: format!("0-{}", rows.len() - 1),
        benchmark_job_count: rows.len(),
        resource_envelope,
        rows,
    };
    validate_benchmark_local_hpc_stage_benchmark_array(&report)?;
    Ok(report)
}
fn build_stage_benchmark_array_row(
    array_index: usize,
    job: &BenchmarkLocalAllDomainSlurmSubmitJob,
    scratch_by_result: &BTreeMap<&str, &BenchmarkLocalHpcScratchLayoutJob>,
    execution_by_tool: &BTreeMap<&str, &BenchmarkLocalHpcExecutionResolverRow>,
) -> Result<BenchmarkLocalHpcStageBenchmarkArrayRow> {
    let result_id = job.result_id.clone().ok_or_else(|| {
        anyhow!(
            "HPC stage benchmark array benchmark job `{}` is missing result_id",
            job.job_id_local
        )
    })?;
    let scratch = scratch_by_result.get(result_id.as_str()).ok_or_else(|| {
        anyhow!(
            "HPC stage benchmark array is missing scratch layout for benchmark result `{result_id}`"
        )
    })?;
    ensure!(
        scratch.job_id_local == job.job_id_local
            && scratch.domain == job.domain
            && scratch.stage_id == job.stage_id
            && scratch.tool_id == job.tool_id
            && scratch.corpus_id == job.corpus_id
            && scratch.asset_profile_id == job.asset_profile_id
            && scratch.pipeline_id.is_none()
            && scratch.node_id.is_none(),
        "HPC stage benchmark array scratch identity differs from selected job `{}`",
        job.job_id_local
    );
    let execution = execution_by_tool.get(job.tool_id.as_str()).ok_or_else(|| {
        anyhow!(
            "HPC stage benchmark array is missing execution resolver coverage for tool `{}`",
            job.tool_id
        )
    })?;
    if !execution.selected_job_ids.contains(&job.job_id_local) {
        return Err(anyhow!(
            "HPC stage benchmark array tool `{}` does not cover selected job `{}` in the execution resolver",
            job.tool_id,
            job.job_id_local
        ));
    }
    if !execution.selected_result_ids.iter().any(|value| value == &result_id) {
        return Err(anyhow!(
            "HPC stage benchmark array tool `{}` does not cover result `{result_id}` in the execution resolver",
            job.tool_id
        ));
    }
    let benchmark_result_argv = rendered_benchmark_result_execution_argv(&result_id);
    let benchmark_result_command = render_shell_command(&benchmark_result_argv);
    Ok(BenchmarkLocalHpcStageBenchmarkArrayRow {
        array_index,
        job_id_local: job.job_id_local.clone(),
        result_id,
        domain: job.domain.clone(),
        stage_id: job.stage_id.clone(),
        tool_id: job.tool_id.clone(),
        corpus_id: job.corpus_id.clone(),
        asset_profile_id: job.asset_profile_id.clone(),
        execution_mode: execution.execution_mode.clone(),
        execution_mode_summary: execution.execution_mode_summary.clone(),
        resolution_kind: execution.resolution_kind.clone(),
        resolution_target: execution.resolution_target.clone(),
        unavailable_reason: execution.unavailable_reason.clone(),
        script_path: job.script_path.clone(),
        benchmark_result_argv,
        benchmark_result_command,
        scratch_root: scratch.scratch_root.clone(),
        input_links_root: scratch.input_links_root.clone(),
        output_root: scratch.output_root.clone(),
        log_root: scratch.log_root.clone(),
        stdout_path: scratch.stdout_path.clone(),
        stderr_path: scratch.stderr_path.clone(),
        output_paths: job.outputs.clone(),
        resources: BenchmarkLocalHpcArrayResources {
            cpus_per_task: scratch.resources.cpus_per_task,
            memory_mb: scratch.resources.memory_mb,
            time_limit: scratch.resources.time_limit.clone(),
            scratch_gb: scratch.resources.scratch_gb,
        },
    })
}
