use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::Path;

use anyhow::{anyhow, Result};
use serde::Serialize;

use super::local_hpc_selected_jobs::load_local_hpc_selected_jobs;
use crate::commands::benchmark::local_all_domain_slurm_submit_manifest::DEFAULT_ALL_DOMAIN_SLURM_SUBMIT_MANIFEST_PATH;

const LOCAL_HPC_JOB_GRAPH_SCHEMA_VERSION: &str = "bijux.bench.local_hpc_job_graph.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcJobGraphNode {
    pub(crate) job_id_local: String,
    pub(crate) job_kind: String,
    pub(crate) result_id: Option<String>,
    pub(crate) pipeline_id: Option<String>,
    pub(crate) node_id: Option<String>,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) dependency_count: usize,
    pub(crate) dependent_count: usize,
    pub(crate) dependencies: Vec<String>,
    pub(crate) dependents: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcJobGraph {
    pub(crate) schema_version: &'static str,
    pub(crate) manifest_path: String,
    pub(crate) job_count: usize,
    pub(crate) benchmark_job_count: usize,
    pub(crate) essential_pipeline_job_count: usize,
    pub(crate) dependency_count: usize,
    pub(crate) root_job_ids: Vec<String>,
    pub(crate) topological_order: Vec<String>,
    pub(crate) nodes: Vec<LocalHpcJobGraphNode>,
}

pub(crate) fn collect_local_hpc_job_graph(repo_root: &Path) -> Result<LocalHpcJobGraph> {
    let selected_jobs = load_local_hpc_selected_jobs(repo_root)?;
    let mut dependency_index = BTreeMap::<String, Vec<String>>::new();
    let mut node_index = BTreeMap::<String, LocalHpcJobGraphNode>::new();
    let mut benchmark_job_count = 0usize;
    let mut essential_pipeline_job_count = 0usize;

    for job in selected_jobs {
        let job_kind = classify_job_kind(
            job.result_id.as_deref(),
            job.pipeline_id.as_deref(),
            job.node_id.as_deref(),
        )?;
        match job_kind {
            "benchmark_result" => benchmark_job_count += 1,
            "essential_pipeline_node" => essential_pipeline_job_count += 1,
            _ => {
                return Err(anyhow!(
                    "HPC job graph encountered unsupported job kind `{job_kind}`"
                ))
            }
        }
        if node_index.contains_key(&job.job_id_local) {
            return Err(anyhow!(
                "HPC job graph requires one unique local job id per selected job, found duplicate `{}`",
                job.job_id_local
            ));
        }
        dependency_index.insert(job.job_id_local.clone(), job.dependencies.clone());
        node_index.insert(
            job.job_id_local.clone(),
            LocalHpcJobGraphNode {
                job_id_local: job.job_id_local,
                job_kind: job_kind.to_string(),
                result_id: job.result_id,
                pipeline_id: job.pipeline_id,
                node_id: job.node_id,
                domain: job.domain,
                stage_id: job.stage_id,
                tool_id: job.tool_id,
                corpus_id: job.corpus_id,
                asset_profile_id: job.asset_profile_id,
                dependency_count: 0,
                dependent_count: 0,
                dependencies: Vec::new(),
                dependents: Vec::new(),
            },
        );
    }

    for (job_id_local, dependencies) in &dependency_index {
        for dependency in dependencies {
            if !node_index.contains_key(dependency) {
                return Err(anyhow!(
                    "HPC job graph job `{job_id_local}` references unknown dependency `{dependency}`"
                ));
            }
        }
    }

    let mut dependent_index = BTreeMap::<String, Vec<String>>::new();
    for job_id_local in dependency_index.keys() {
        dependent_index.entry(job_id_local.clone()).or_default();
    }
    for (job_id_local, dependencies) in &dependency_index {
        for dependency in dependencies {
            dependent_index
                .entry(dependency.clone())
                .or_default()
                .push(job_id_local.clone());
        }
    }

    for node in node_index.values_mut() {
        let dependencies = dependency_index
            .get(&node.job_id_local)
            .cloned()
            .unwrap_or_default();
        let dependents = dependent_index
            .get(&node.job_id_local)
            .cloned()
            .unwrap_or_default();
        node.dependency_count = dependencies.len();
        node.dependent_count = dependents.len();
        node.dependencies = dependencies;
        node.dependents = dependents;
    }

    let topological_order = topological_sort(&node_index)?;
    let root_job_ids = topological_order
        .iter()
        .filter_map(|job_id_local| {
            node_index.get(job_id_local).and_then(|node| {
                if node.dependencies.is_empty() { Some(job_id_local.clone()) } else { None }
            })
        })
        .collect::<Vec<_>>();
    let dependency_count = node_index
        .values()
        .map(|node| node.dependencies.len())
        .sum::<usize>();
    let nodes = topological_order
        .iter()
        .map(|job_id_local| {
            node_index.get(job_id_local).cloned().ok_or_else(|| {
                anyhow!("HPC job graph topological order references unknown job `{job_id_local}`")
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let graph = LocalHpcJobGraph {
        schema_version: LOCAL_HPC_JOB_GRAPH_SCHEMA_VERSION,
        manifest_path: DEFAULT_ALL_DOMAIN_SLURM_SUBMIT_MANIFEST_PATH.to_string(),
        job_count: nodes.len(),
        benchmark_job_count,
        essential_pipeline_job_count,
        dependency_count,
        root_job_ids,
        topological_order,
        nodes,
    };
    ensure_hpc_job_graph_contract(&graph)?;
    Ok(graph)
}

fn classify_job_kind(
    result_id: Option<&str>,
    pipeline_id: Option<&str>,
    node_id: Option<&str>,
) -> Result<&'static str> {
    match (result_id, pipeline_id, node_id) {
        (Some(_), None, None) => Ok("benchmark_result"),
        (None, Some(_), Some(_)) => Ok("essential_pipeline_node"),
        _ => Err(anyhow!(
            "HPC job graph requires selected jobs to be benchmark results or essential pipeline nodes"
        )),
    }
}

fn topological_sort(nodes: &BTreeMap<String, LocalHpcJobGraphNode>) -> Result<Vec<String>> {
    let mut indegree = nodes
        .iter()
        .map(|(job_id_local, node)| (job_id_local.clone(), node.dependencies.len()))
        .collect::<BTreeMap<_, _>>();
    let mut ready = indegree
        .iter()
        .filter_map(|(job_id_local, degree)| {
            if *degree == 0 { Some(job_id_local.clone()) } else { None }
        })
        .collect::<VecDeque<_>>();
    let mut order = Vec::with_capacity(nodes.len());

    while let Some(job_id_local) = ready.pop_front() {
        order.push(job_id_local.clone());
        let dependents = nodes
            .get(&job_id_local)
            .map(|node| node.dependents.clone())
            .unwrap_or_default();
        for dependent in dependents {
            let degree = indegree.get_mut(&dependent).ok_or_else(|| {
                anyhow!("HPC job graph indegree index is missing dependent job `{dependent}`")
            })?;
            if *degree == 0 {
                return Err(anyhow!(
                    "HPC job graph encountered inconsistent zero indegree for `{dependent}`"
                ));
            }
            *degree -= 1;
            if *degree == 0 {
                ready.push_back(dependent);
            }
        }
    }

    if order.len() != nodes.len() {
        let unresolved = indegree
            .into_iter()
            .filter_map(|(job_id_local, degree)| if degree > 0 { Some(job_id_local) } else { None })
            .collect::<BTreeSet<_>>();
        return Err(anyhow!(
            "HPC job graph must stay acyclic, unresolved jobs remain: {}",
            unresolved.into_iter().collect::<Vec<_>>().join(", ")
        ));
    }

    Ok(order)
}

fn ensure_hpc_job_graph_contract(graph: &LocalHpcJobGraph) -> Result<()> {
    if graph.job_count != graph.benchmark_job_count + graph.essential_pipeline_job_count {
        return Err(anyhow!(
            "HPC job graph job count must equal benchmark jobs plus essential pipeline jobs"
        ));
    }
    if graph.topological_order.len() != graph.job_count || graph.nodes.len() != graph.job_count {
        return Err(anyhow!(
            "HPC job graph must keep one node and one topological row per selected job"
        ));
    }
    let unique_job_ids = graph
        .nodes
        .iter()
        .map(|node| node.job_id_local.as_str())
        .collect::<BTreeSet<_>>();
    if unique_job_ids.len() != graph.job_count {
        return Err(anyhow!(
            "HPC job graph must keep one unique local job id per node"
        ));
    }
    for node in &graph.nodes {
        if node.dependencies.len() != node.dependency_count {
            return Err(anyhow!(
                "HPC job graph node `{}` must keep dependency count aligned with dependency rows",
                node.job_id_local
            ));
        }
        if node.dependents.len() != node.dependent_count {
            return Err(anyhow!(
                "HPC job graph node `{}` must keep dependent count aligned with dependent rows",
                node.job_id_local
            ));
        }
    }
    Ok(())
}
