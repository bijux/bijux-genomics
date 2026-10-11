//! Owner: pipeline array coverage and dependency ordering over borrowed observations.
mod lineage;
mod rows;
use crate::{
    build_benchmark_local_hpc_array_resource_envelope,
    validate_benchmark_local_hpc_pipeline_node_array, BenchmarkLocalAllDomainSlurmSubmitJob,
    BenchmarkLocalHpcPipelineNodeArrayReport, BenchmarkLocalHpcScratchLayoutJob,
    BenchmarkLocalPipelineDagValidationReport, BENCHMARK_LOCAL_HPC_PIPELINE_NODE_ARRAY_SCHEMA_V1,
};
use anyhow::{anyhow, ensure, Result};
use lineage::output_producer_map;
use rows::{build_pipeline_node_array_row, PipelineRowContext};
use std::collections::{BTreeMap, BTreeSet};
/// Declared pipeline order and current selected-job, scratch and DAG observations.
pub struct HpcPipelineArrayInputs<'a> {
    pub pipeline_order: &'a [&'a str],
    pub pipelines: &'a BTreeMap<String, BenchmarkLocalPipelineDagValidationReport>,
    pub jobs: &'a [BenchmarkLocalAllDomainSlurmSubmitJob],
    pub scratch: &'a [BenchmarkLocalHpcScratchLayoutJob],
    pub script_path: String,
    pub manifest_path: String,
    pub wrapper_log_root: String,
}
/// Assemble an admitted array with explicit source coverage and output lineage.
/// # Errors
/// Rejects empty scope, ambiguous rows, missing coverage, invalid dependencies and output drift.
pub fn build_hpc_pipeline_array_report(
    inputs: HpcPipelineArrayInputs<'_>,
) -> Result<BenchmarkLocalHpcPipelineNodeArrayReport> {
    admit_pipeline_reports(&inputs)?;
    let mut scratch_by_node = BTreeMap::new();
    for job in inputs.scratch {
        if let (Some(pipeline), Some(node)) = (&job.pipeline_id, &job.node_id) {
            ensure!(
                scratch_by_node.insert((pipeline.clone(), node.clone()), job).is_none(),
                "HPC pipeline node array has duplicate scratch node `{pipeline}` / `{node}`"
            );
        }
    }
    let mut selected_job_by_node = BTreeMap::new();
    for job in inputs.jobs {
        if let (Some(pipeline), Some(node), None) = (&job.pipeline_id, &job.node_id, &job.result_id)
        {
            ensure!(
                selected_job_by_node.insert((pipeline.clone(), node.clone()), job).is_none(),
                "HPC pipeline node array has duplicate selected node `{pipeline}` / `{node}`"
            );
        }
    }
    ensure!(
        !selected_job_by_node.is_empty(),
        "HPC pipeline node array requires selected pipeline jobs"
    );
    let mut rows = Vec::new();
    let mut row_index_by_job_id = BTreeMap::<String, usize>::new();
    for pipeline_id in inputs.pipeline_order {
        let pipeline = inputs.pipelines.get(*pipeline_id).ok_or_else(|| {
            anyhow!("HPC pipeline node array is missing DAG validation report for `{pipeline_id}`")
        })?;
        let nodes_by_id = pipeline
            .nodes
            .iter()
            .map(|node| (node.node_id.clone(), node))
            .collect::<BTreeMap<_, _>>();
        let output_producers = output_producer_map(&pipeline.nodes)?;

        for node_id in &pipeline.topological_order {
            let node = nodes_by_id.get(node_id).ok_or_else(|| {
                anyhow!(
                    "HPC pipeline node array is missing node `{node_id}` in pipeline `{pipeline_id}`"
                )
            })?;
            let key = ((*pipeline_id).to_string(), node.node_id.clone());
            let job = selected_job_by_node.get(&key).ok_or_else(|| {
                anyhow!(
                    "HPC pipeline node array is missing selected job coverage for `{pipeline_id}` / `{}`",
                    node.node_id
                )
            })?;
            let scratch = scratch_by_node.get(&key).ok_or_else(|| {
                anyhow!(
                    "HPC pipeline node array is missing scratch layout for `{pipeline_id}` / `{}`",
                    node.node_id
                )
            })?;
            let row = build_pipeline_node_array_row(
                rows.len(),
                pipeline,
                node,
                job,
                scratch,
                &PipelineRowContext {
                    nodes_by_id: &nodes_by_id,
                    output_producers: &output_producers,
                    selected_job_by_node: &selected_job_by_node,
                    row_index_by_job_id: &row_index_by_job_id,
                },
            )?;
            row_index_by_job_id.insert(row.job_id_local.clone(), row.array_index);
            rows.push(row);
        }
    }

    if rows.len() != selected_job_by_node.len() {
        return Err(anyhow!(
            "HPC pipeline node array expected {} selected essential pipeline jobs but rendered {} rows",
            selected_job_by_node.len(),
            rows.len()
        ));
    }
    let resource_envelope =
        build_benchmark_local_hpc_array_resource_envelope(rows.iter().map(|row| &row.resources))?;
    let dependency_count = rows.iter().map(|row| row.dependencies.len()).sum::<usize>();
    let report = BenchmarkLocalHpcPipelineNodeArrayReport {
        schema_version: BENCHMARK_LOCAL_HPC_PIPELINE_NODE_ARRAY_SCHEMA_V1.to_string(),
        script_path: inputs.script_path,
        manifest_path: inputs.manifest_path,
        wrapper_log_root: inputs.wrapper_log_root,
        array_spec: format!("0-{}", rows.len() - 1),
        pipeline_count: inputs.pipelines.len(),
        pipeline_job_count: rows.len(),
        dependency_count,
        resource_envelope,
        rows,
    };
    validate_benchmark_local_hpc_pipeline_node_array(&report)?;
    Ok(report)
}

fn admit_pipeline_reports(inputs: &HpcPipelineArrayInputs<'_>) -> Result<()> {
    let declared = inputs.pipeline_order.iter().copied().collect::<BTreeSet<_>>();
    ensure!(
        !declared.is_empty()
            && declared.len() == inputs.pipeline_order.len()
            && declared == inputs.pipelines.keys().map(String::as_str).collect(),
        "HPC pipeline node array pipeline order must uniquely cover the observed DAG reports"
    );
    for (identity, report) in inputs.pipelines {
        ensure!(
            identity == &report.pipeline_id,
            "HPC pipeline node array DAG map identity differs from its report"
        );
        crate::validate_benchmark_local_pipeline_dag_validation(report)?;
    }
    Ok(())
}
