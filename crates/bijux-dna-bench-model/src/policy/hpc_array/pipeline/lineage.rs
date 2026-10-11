//! Owner: unique output producers and declared upstream lineage for HPC pipeline arrays.
use crate::{
    BenchmarkLocalAllDomainSlurmSubmitJob, BenchmarkLocalHpcPipelineNodeArrayExpectedOutput,
    BenchmarkLocalPipelineDagValidationNode,
};
use anyhow::{anyhow, Result};
use std::collections::{BTreeMap, BTreeSet};
pub(super) fn output_producer_map(
    nodes: &[BenchmarkLocalPipelineDagValidationNode],
) -> Result<BTreeMap<String, String>> {
    let mut producers = BTreeMap::new();
    for node in nodes {
        for output_id in &node.outputs {
            if let Some(previous) = producers.insert(output_id.clone(), node.node_id.clone()) {
                return Err(anyhow!(
                    "HPC pipeline node array found duplicate producer ownership for output `{output_id}`: `{previous}` and `{}`",
                    node.node_id
                ));
            }
        }
    }
    Ok(producers)
}

pub(super) fn expected_outputs_for_job(
    node: &BenchmarkLocalPipelineDagValidationNode,
    job: &BenchmarkLocalAllDomainSlurmSubmitJob,
) -> Result<Vec<BenchmarkLocalHpcPipelineNodeArrayExpectedOutput>> {
    if node.outputs.len() != job.outputs.len() {
        return Err(anyhow!(
            "HPC pipeline node array found {} declared outputs in the DAG but {} output paths in the submit manifest for `{}`",
            node.outputs.len(),
            job.outputs.len(),
            job.job_id_local
        ));
    }
    Ok(node
        .outputs
        .iter()
        .zip(job.outputs.iter())
        .map(|(output_id, output_path)| BenchmarkLocalHpcPipelineNodeArrayExpectedOutput {
            output_id: output_id.clone(),
            output_path: output_path.clone(),
        })
        .collect())
}

pub(super) fn group_upstream_result_ids_by_dependency(
    node: &BenchmarkLocalPipelineDagValidationNode,
    output_producers: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, Vec<String>>> {
    let declared_dependencies = node.depends_on.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let mut upstream_result_ids_by_dependency = BTreeMap::<String, Vec<String>>::new();
    for upstream_result_id in &node.upstream_inputs {
        let producer_node_id = output_producers.get(upstream_result_id).ok_or_else(|| {
            anyhow!(
                "HPC pipeline node array cannot resolve producer for upstream result `{upstream_result_id}` in node `{}`",
                node.node_id
            )
        })?;
        if !declared_dependencies.contains(producer_node_id.as_str()) {
            return Err(anyhow!(
                "HPC pipeline node array found undeclared producer `{producer_node_id}` for upstream result `{upstream_result_id}` in node `{}`",
                node.node_id
            ));
        }
        upstream_result_ids_by_dependency
            .entry(producer_node_id.clone())
            .or_default()
            .push(upstream_result_id.clone());
    }
    Ok(upstream_result_ids_by_dependency)
}
