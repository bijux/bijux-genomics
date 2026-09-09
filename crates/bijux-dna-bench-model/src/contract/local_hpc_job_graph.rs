//! Owner: bijux-dna-bench-model
//! Deterministic dependency graph construction from typed selected submission jobs.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use anyhow::{anyhow, Result};

use crate::{
    validate_benchmark_local_hpc_job_graph, BenchmarkLocalAllDomainSlurmSubmitJob,
    BenchmarkLocalHpcJobGraph, BenchmarkLocalHpcJobGraphNode,
    BENCHMARK_LOCAL_HPC_JOB_GRAPH_SCHEMA_V1,
};

/// Build the dependency projection of selected submission jobs without repository IO.
/// The caller supplies the manifest identity; process resources and output paths are
/// admitted by the submission contract before execution, outside this graph projection.
///
/// # Errors
/// Rejects ambiguous job kinds, duplicate identities, missing dependencies, cycles,
/// and graph fields that fail the persisted graph's semantic contract.
pub fn build_benchmark_local_hpc_job_graph(
    manifest_path: &str,
    selected_jobs: Vec<BenchmarkLocalAllDomainSlurmSubmitJob>,
) -> Result<BenchmarkLocalHpcJobGraph> {
    let mut dependency_index = BTreeMap::<String, Vec<String>>::new();
    let mut node_index = BTreeMap::<String, BenchmarkLocalHpcJobGraphNode>::new();
    for job in selected_jobs {
        let dependencies = job.dependencies.clone();
        let node = project_job_node(job)?;
        if node_index.contains_key(&node.job_id_local) {
            return Err(anyhow!(
                "HPC job graph requires one unique local job id per selected job, found duplicate `{}`",
                node.job_id_local
            ));
        }
        dependency_index.insert(node.job_id_local.clone(), dependencies);
        node_index.insert(node.job_id_local.clone(), node);
    }
    let benchmark_job_count =
        node_index.values().filter(|node| node.job_kind == "benchmark_result").count();
    let essential_pipeline_job_count = node_index.len() - benchmark_job_count;

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
            dependent_index.entry(dependency.clone()).or_default().push(job_id_local.clone());
        }
    }

    for node in node_index.values_mut() {
        let dependencies = dependency_index.get(&node.job_id_local).cloned().unwrap_or_default();
        let dependents = dependent_index.get(&node.job_id_local).cloned().unwrap_or_default();
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
                if node.dependencies.is_empty() {
                    Some(job_id_local.clone())
                } else {
                    None
                }
            })
        })
        .collect::<Vec<_>>();
    let dependency_count = node_index.values().map(|node| node.dependencies.len()).sum::<usize>();
    let nodes = topological_order
        .iter()
        .map(|job_id_local| {
            node_index.get(job_id_local).cloned().ok_or_else(|| {
                anyhow!("HPC job graph topological order references unknown job `{job_id_local}`")
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let graph = BenchmarkLocalHpcJobGraph {
        schema_version: BENCHMARK_LOCAL_HPC_JOB_GRAPH_SCHEMA_V1.to_string(),
        manifest_path: manifest_path.to_string(),
        job_count: nodes.len(),
        benchmark_job_count,
        essential_pipeline_job_count,
        dependency_count,
        root_job_ids,
        topological_order,
        nodes,
    };
    validate_benchmark_local_hpc_job_graph(&graph)?;
    Ok(graph)
}

fn project_job_node(
    job: BenchmarkLocalAllDomainSlurmSubmitJob,
) -> Result<BenchmarkLocalHpcJobGraphNode> {
    let job_kind = classify_job_kind(
        job.result_id.as_deref(),
        job.pipeline_id.as_deref(),
        job.node_id.as_deref(),
    )?;
    Ok(BenchmarkLocalHpcJobGraphNode {
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
    })
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

fn topological_sort(
    nodes: &BTreeMap<String, BenchmarkLocalHpcJobGraphNode>,
) -> Result<Vec<String>> {
    let mut indegree = nodes
        .iter()
        .map(|(job_id_local, node)| (job_id_local.clone(), node.dependencies.len()))
        .collect::<BTreeMap<_, _>>();
    let mut ready = indegree
        .iter()
        .filter_map(
            |(job_id_local, degree)| {
                if *degree == 0 {
                    Some(job_id_local.clone())
                } else {
                    None
                }
            },
        )
        .collect::<VecDeque<_>>();
    let mut order = Vec::with_capacity(nodes.len());

    while let Some(job_id_local) = ready.pop_front() {
        order.push(job_id_local.clone());
        let dependents =
            nodes.get(&job_id_local).map(|node| node.dependents.clone()).unwrap_or_default();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BenchmarkLocalSlurmSubmitResources;

    fn job(id: &str, dependencies: &[&str]) -> BenchmarkLocalAllDomainSlurmSubmitJob {
        BenchmarkLocalAllDomainSlurmSubmitJob {
            job_id_local: id.to_string(),
            domain: "fastq".to_string(),
            stage_id: "fastq.validate_reads".to_string(),
            pipeline_id: None,
            node_id: None,
            tool_id: "fastp".to_string(),
            corpus_id: "corpus".to_string(),
            asset_profile_id: "profile".to_string(),
            result_id: Some(format!("result-{id}")),
            script_path: format!("{id}.sh"),
            stdout: format!("{id}.out"),
            stderr: format!("{id}.err"),
            outputs: vec![format!("{id}.json")],
            dependencies: dependencies.iter().map(|id| (*id).to_string()).collect(),
            resources: BenchmarkLocalSlurmSubmitResources {
                cpus_per_task: 1,
                memory_mb: 128,
                time_limit: "00:01:00".to_string(),
            },
        }
    }

    #[test]
    fn graph_projection_preserves_edges_and_orders_permuted_inputs_deterministically() -> Result<()>
    {
        let a = job("a", &[]);
        let b = job("b", &[]);
        let mut c = job("c", &["a", "b"]);
        c.result_id = None;
        c.pipeline_id = Some("pipeline".to_string());
        c.node_id = Some("node".to_string());
        let first = build_benchmark_local_hpc_job_graph(
            "submit.json",
            vec![c.clone(), b.clone(), a.clone()],
        )?;
        let second = build_benchmark_local_hpc_job_graph("submit.json", vec![a, c, b])?;
        assert_eq!(first, second);
        assert_eq!(first.manifest_path, "submit.json");
        assert_eq!(first.topological_order, ["a", "b", "c"]);
        assert_eq!(first.root_job_ids, ["a", "b"]);
        assert_eq!(
            (
                first.job_count,
                first.benchmark_job_count,
                first.essential_pipeline_job_count,
                first.dependency_count
            ),
            (3, 2, 1, 2)
        );
        assert_eq!(first.nodes[0].dependents, ["c"]);
        assert_eq!(first.nodes[2].dependencies, ["a", "b"]);
        let json = serde_json::to_vec(&first)?;
        let restored: BenchmarkLocalHpcJobGraph = serde_json::from_slice(&json)?;
        validate_benchmark_local_hpc_job_graph(&restored)?;
        assert_eq!(restored, first);
        Ok(())
    }

    #[test]
    fn graph_projection_rejects_duplicate_missing_cyclic_and_ambiguous_identities() {
        let mut ambiguous = job("a", &[]);
        ambiguous.pipeline_id = Some("pipeline".to_string());
        let cases = [
            (vec![job("a", &[]), job("a", &[])], "duplicate `a`"),
            (vec![job("a", &["missing"])], "unknown dependency `missing`"),
            (vec![job("a", &["b"]), job("b", &["a"])], "acyclic"),
            (vec![ambiguous], "benchmark results or essential pipeline nodes"),
            (vec![], "non-empty nodes"),
        ];
        for (jobs, message) in cases {
            let Err(error) = build_benchmark_local_hpc_job_graph("submit.json", jobs) else {
                panic!("invalid graph accepted: {message}");
            };
            assert!(error.to_string().contains(message), "{error}");
        }
    }

    #[test]
    fn graph_projection_keeps_transport_fields_outside_dependency_semantics() -> Result<()> {
        let original = job("a", &[]);
        let mut transported = original.clone();
        transported.script_path = "another.sh".to_string();
        transported.stdout = "another.stdout".to_string();
        transported.resources.cpus_per_task = 4;
        assert_eq!(
            build_benchmark_local_hpc_job_graph("submit.json", vec![original])?,
            build_benchmark_local_hpc_job_graph("submit.json", vec![transported])?
        );
        assert!(build_benchmark_local_hpc_job_graph(" ", vec![job("a", &[])]).is_err());
        Ok(())
    }
}
