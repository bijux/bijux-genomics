//! Owner: typed pipeline-node array rows bound to selected jobs and dependency observations.
use super::super::commands::{
    render_shell_command, rendered_essential_pipeline_node_execution_argv,
};
use super::lineage::{expected_outputs_for_job, group_upstream_result_ids_by_dependency};
use crate::{
    BenchmarkLocalAllDomainSlurmSubmitJob, BenchmarkLocalHpcArrayResources,
    BenchmarkLocalHpcPipelineNodeArrayDependency, BenchmarkLocalHpcPipelineNodeArrayRow,
    BenchmarkLocalHpcScratchLayoutJob, BenchmarkLocalPipelineDagValidationNode,
    BenchmarkLocalPipelineDagValidationReport,
};
use anyhow::{anyhow, ensure, Result};
use std::collections::BTreeMap;
pub(super) struct PipelineRowContext<'a> {
    pub(super) nodes_by_id: &'a BTreeMap<String, &'a BenchmarkLocalPipelineDagValidationNode>,
    pub(super) output_producers: &'a BTreeMap<String, String>,
    pub(super) selected_job_by_node:
        &'a BTreeMap<(String, String), &'a BenchmarkLocalAllDomainSlurmSubmitJob>,
    pub(super) row_index_by_job_id: &'a BTreeMap<String, usize>,
}
pub(super) fn build_pipeline_node_array_row(
    array_index: usize,
    pipeline: &BenchmarkLocalPipelineDagValidationReport,
    node: &BenchmarkLocalPipelineDagValidationNode,
    job: &BenchmarkLocalAllDomainSlurmSubmitJob,
    scratch: &BenchmarkLocalHpcScratchLayoutJob,
    context: &PipelineRowContext<'_>,
) -> Result<BenchmarkLocalHpcPipelineNodeArrayRow> {
    let pipeline_id = job.pipeline_id.clone().ok_or_else(|| {
        anyhow!(
            "HPC pipeline node array selected job `{}` is missing pipeline_id",
            job.job_id_local
        )
    })?;
    let node_id = job.node_id.clone().ok_or_else(|| {
        anyhow!("HPC pipeline node array selected job `{}` is missing node_id", job.job_id_local)
    })?;
    if pipeline_id != pipeline.pipeline_id || node_id != node.node_id {
        return Err(anyhow!(
            "HPC pipeline node array received mismatched job coverage for `{pipeline_id}` / `{node_id}`"
        ));
    }
    ensure!(
        scratch.job_id_local == job.job_id_local
            && scratch.domain == job.domain
            && scratch.stage_id == job.stage_id
            && scratch.tool_id == job.tool_id
            && scratch.corpus_id == job.corpus_id
            && scratch.asset_profile_id == job.asset_profile_id
            && scratch.result_id.is_none(),
        "HPC pipeline node array scratch identity differs from selected job `{}`",
        job.job_id_local
    );
    let expected_outputs = expected_outputs_for_job(node, job)?;
    let upstream_result_ids = node.upstream_inputs.clone();
    let dependency_job_ids = node
        .depends_on
        .iter()
        .map(|dependency_node_id| format!("pipeline:{pipeline_id}:{dependency_node_id}"))
        .collect::<Vec<_>>();
    if job.dependencies != dependency_job_ids {
        return Err(anyhow!(
            "HPC pipeline node array found submit-manifest dependency drift for `{pipeline_id}` / `{node_id}`"
        ));
    }

    let dependencies = build_dependencies(&pipeline_id, &node_id, node, context)?;

    let pipeline_node_argv =
        rendered_essential_pipeline_node_execution_argv(&pipeline_id, &node_id);
    let pipeline_node_command = render_shell_command(&pipeline_node_argv);
    Ok(BenchmarkLocalHpcPipelineNodeArrayRow {
        array_index,
        job_id_local: job.job_id_local.clone(),
        pipeline_id,
        node_id,
        domain: job.domain.clone(),
        stage_id: job.stage_id.clone(),
        tool_id: job.tool_id.clone(),
        corpus_id: job.corpus_id.clone(),
        asset_profile_id: job.asset_profile_id.clone(),
        dependency_job_ids,
        upstream_result_ids,
        dependencies,
        expected_outputs,
        script_path: job.script_path.clone(),
        pipeline_node_argv,
        pipeline_node_command,
        scratch_root: scratch.scratch_root.clone(),
        input_links_root: scratch.input_links_root.clone(),
        output_root: scratch.output_root.clone(),
        log_root: scratch.log_root.clone(),
        stdout_path: scratch.stdout_path.clone(),
        stderr_path: scratch.stderr_path.clone(),
        resources: BenchmarkLocalHpcArrayResources {
            cpus_per_task: scratch.resources.cpus_per_task,
            memory_mb: scratch.resources.memory_mb,
            time_limit: scratch.resources.time_limit.clone(),
            scratch_gb: scratch.resources.scratch_gb,
        },
    })
}

fn build_dependencies(
    pipeline_id: &str,
    node_id: &str,
    node: &BenchmarkLocalPipelineDagValidationNode,
    context: &PipelineRowContext<'_>,
) -> Result<Vec<BenchmarkLocalHpcPipelineNodeArrayDependency>> {
    let PipelineRowContext {
        nodes_by_id,
        output_producers,
        selected_job_by_node,
        row_index_by_job_id,
    } = context;
    let upstream_result_ids_by_dependency =
        group_upstream_result_ids_by_dependency(node, output_producers)?;
    let mut dependencies = Vec::with_capacity(node.depends_on.len());
    for dependency_node_id in &node.depends_on {
        let dependency_node = nodes_by_id.get(dependency_node_id).ok_or_else(|| {
            anyhow!(
                "HPC pipeline node array is missing dependency node `{dependency_node_id}` in pipeline `{pipeline_id}`"
            )
        })?;
        let dependency_job = selected_job_by_node
            .get(&(pipeline_id.to_string(), dependency_node_id.clone()))
            .ok_or_else(|| {
                anyhow!(
                    "HPC pipeline node array is missing selected dependency job `{pipeline_id}` / `{dependency_node_id}`"
                )
            })?;
        let dependency_job_id = dependency_job.job_id_local.clone();
        let dependency_array_index = row_index_by_job_id.get(&dependency_job_id).copied().ok_or_else(
            || {
                anyhow!(
                    "HPC pipeline node array dependency `{dependency_job_id}` for `{pipeline_id}` / `{node_id}` requires manual reordering"
                )
            },
        )?;
        let dependency_outputs = expected_outputs_for_job(dependency_node, dependency_job)?;
        let dependency_output_by_id = dependency_outputs
            .into_iter()
            .map(|output| (output.output_id.clone(), output))
            .collect::<BTreeMap<_, _>>();
        let upstream_result_ids =
            upstream_result_ids_by_dependency.get(dependency_node_id).cloned().unwrap_or_default();
        let expected_outputs = if upstream_result_ids.is_empty() {
            dependency_output_by_id.into_values().collect::<Vec<_>>()
        } else {
            upstream_result_ids
                .iter()
                .map(|result_id| {
                    dependency_output_by_id.get(result_id).cloned().ok_or_else(|| {
                        anyhow!(
                            "HPC pipeline node array dependency `{dependency_job_id}` is missing output `{result_id}` required by `{pipeline_id}` / `{node_id}`"
                        )
                    })
                })
                .collect::<Result<Vec<_>>>()?
        };
        dependencies.push(BenchmarkLocalHpcPipelineNodeArrayDependency {
            node_id: dependency_node_id.clone(),
            job_id_local: dependency_job_id,
            array_index: dependency_array_index,
            upstream_result_ids,
            expected_outputs,
        });
    }

    Ok(dependencies)
}
