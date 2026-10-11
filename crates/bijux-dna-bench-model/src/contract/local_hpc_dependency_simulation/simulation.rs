//! Owner: bijux-dna-bench-model
//! Pure dependency failure simulation over an admitted HPC job graph.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::{
    validate_benchmark_local_hpc_dependency_simulation, validate_benchmark_local_hpc_job_graph,
    BenchmarkLocalHpcDependencySimulationCase, BenchmarkLocalHpcDependencySimulationJob,
    BenchmarkLocalHpcDependencySimulationReport, BenchmarkLocalHpcJobGraph,
    BENCHMARK_LOCAL_HPC_DEPENDENCY_SIMULATION_SCHEMA_V1,
    BENCHMARK_LOCAL_HPC_REFERENCE_BLOCKED_JOB_ID, BENCHMARK_LOCAL_HPC_REFERENCE_CONTINUED_JOB_ID,
    BENCHMARK_LOCAL_HPC_REFERENCE_FAILURE_JOB_ID, BENCHMARK_LOCAL_HPC_RELATEDNESS_BLOCKED_JOB_ID,
    BENCHMARK_LOCAL_HPC_RELATEDNESS_CONTINUED_JOB_ID,
    BENCHMARK_LOCAL_HPC_RELATEDNESS_FAILURE_JOB_ID,
};
use anyhow::{anyhow, Result};

const RELATEDNESS_IBD_FAILURE_JOB_ID: &str = BENCHMARK_LOCAL_HPC_RELATEDNESS_FAILURE_JOB_ID;
const RELATEDNESS_BLOCKED_DESCENDANT_JOB_ID: &str = BENCHMARK_LOCAL_HPC_RELATEDNESS_BLOCKED_JOB_ID;
const RELATEDNESS_CONTINUED_SIBLING_JOB_ID: &str = BENCHMARK_LOCAL_HPC_RELATEDNESS_CONTINUED_JOB_ID;
const REFERENCE_PANEL_FAILURE_JOB_ID: &str = BENCHMARK_LOCAL_HPC_REFERENCE_FAILURE_JOB_ID;
const REFERENCE_PANEL_BLOCKED_DESCENDANT_JOB_ID: &str =
    BENCHMARK_LOCAL_HPC_REFERENCE_BLOCKED_JOB_ID;
const REFERENCE_PANEL_CONTINUED_SIBLING_JOB_ID: &str =
    BENCHMARK_LOCAL_HPC_REFERENCE_CONTINUED_JOB_ID;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LocalHpcDependencySimulationStatus {
    Completed,
    Failed,
    Blocked,
}

impl LocalHpcDependencySimulationStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Blocked => "blocked",
        }
    }
}

type LocalHpcJobGraph = BenchmarkLocalHpcJobGraph;
type LocalHpcDependencySimulationJob = BenchmarkLocalHpcDependencySimulationJob;
type LocalHpcDependencySimulationCase = BenchmarkLocalHpcDependencySimulationCase;
type LocalHpcDependencySimulationReport = BenchmarkLocalHpcDependencySimulationReport;

/// Simulate the governed branch-isolation proofs without filesystem or process access.
///
/// # Errors
/// Rejects invalid graph topology, missing proof jobs, invalid output identity and incomplete proofs.
pub fn build_benchmark_local_hpc_dependency_simulation(
    graph: &BenchmarkLocalHpcJobGraph,
    output_path: &str,
) -> Result<BenchmarkLocalHpcDependencySimulationReport> {
    validate_benchmark_local_hpc_job_graph(graph)?;
    let cases = vec![
        simulate_failure_case(
            graph,
            "relatedness_ibd_failure_isolates_demography_from_roh",
            RELATEDNESS_IBD_FAILURE_JOB_ID,
            RELATEDNESS_BLOCKED_DESCENDANT_JOB_ID,
            RELATEDNESS_CONTINUED_SIBLING_JOB_ID,
        )?,
        simulate_failure_case(
            graph,
            "reference_panel_prepare_failure_blocks_phasing_without_blocking_qc",
            REFERENCE_PANEL_FAILURE_JOB_ID,
            REFERENCE_PANEL_BLOCKED_DESCENDANT_JOB_ID,
            REFERENCE_PANEL_CONTINUED_SIBLING_JOB_ID,
        )?,
    ];
    let all_cases_proven = cases.iter().all(|case_report| case_report.proven);
    let report = LocalHpcDependencySimulationReport {
        schema_version: BENCHMARK_LOCAL_HPC_DEPENDENCY_SIMULATION_SCHEMA_V1.to_string(),
        manifest_path: graph.manifest_path.clone(),
        output_path: output_path.to_string(),
        job_count: graph.job_count,
        benchmark_job_count: graph.benchmark_job_count,
        essential_pipeline_job_count: graph.essential_pipeline_job_count,
        dependency_count: graph.dependency_count,
        case_count: cases.len(),
        all_cases_proven,
        cases,
    };
    validate_benchmark_local_hpc_dependency_simulation(&report)?;
    Ok(report)
}
fn simulate_failure_case(
    graph: &LocalHpcJobGraph,
    case_id: &str,
    failed_job_id: &str,
    blocked_descendant_job_id: &str,
    continued_sibling_job_id: &str,
) -> Result<LocalHpcDependencySimulationCase> {
    let descendant_job_ids = descendants_of(graph, failed_job_id)?;
    let descendant_set = descendant_job_ids.iter().cloned().collect::<BTreeSet<_>>();
    let jobs = simulate_jobs(graph, case_id, failed_job_id, &descendant_set)?;

    let blocked_job_ids = jobs
        .iter()
        .filter(|job| job.status == LocalHpcDependencySimulationStatus::Blocked.as_str())
        .map(|job| job.job_id_local.clone())
        .collect::<Vec<_>>();
    let continued_unrelated_job_ids = jobs
        .iter()
        .filter(|job| {
            job.status == LocalHpcDependencySimulationStatus::Completed.as_str()
                && job.unrelated_to_failed_job
        })
        .map(|job| job.job_id_local.clone())
        .collect::<Vec<_>>();
    let continued_benchmark_job_ids = jobs
        .iter()
        .filter(|job| {
            job.status == LocalHpcDependencySimulationStatus::Completed.as_str()
                && job.unrelated_to_failed_job
                && job.job_kind == "benchmark_result"
        })
        .map(|job| job.job_id_local.clone())
        .collect::<Vec<_>>();
    let blocked_only_descendants =
        blocked_job_ids.iter().all(|job_id_local| descendant_set.contains(job_id_local));
    let unrelated_branches_continue = continued_unrelated_job_ids
        .iter()
        .any(|job_id_local| job_id_local == continued_sibling_job_id);
    let benchmark_jobs_continue = !continued_benchmark_job_ids.is_empty();
    let blocked_expected_descendant =
        blocked_job_ids.iter().any(|job_id_local| job_id_local == blocked_descendant_job_id);
    let failed_status_ok = jobs.iter().any(|job| {
        job.job_id_local == failed_job_id
            && job.status == LocalHpcDependencySimulationStatus::Failed.as_str()
    });
    let proven = blocked_only_descendants
        && unrelated_branches_continue
        && benchmark_jobs_continue
        && blocked_expected_descendant
        && failed_status_ok;
    let simulated_makespan_seconds = jobs.iter().map(|job| job.finish_second).max().unwrap_or(0);

    Ok(LocalHpcDependencySimulationCase {
        case_id: case_id.to_string(),
        failed_job_id: failed_job_id.to_string(),
        blocked_descendant_job_id: blocked_descendant_job_id.to_string(),
        continued_sibling_job_id: continued_sibling_job_id.to_string(),
        descendant_job_ids,
        blocked_job_ids,
        continued_unrelated_job_ids,
        continued_benchmark_job_ids,
        simulated_makespan_seconds,
        blocked_only_descendants,
        unrelated_branches_continue,
        benchmark_jobs_continue,
        proven,
        jobs,
    })
}

fn descendants_of(graph: &LocalHpcJobGraph, failed_job_id: &str) -> Result<Vec<String>> {
    let nodes_by_job_id = graph
        .nodes
        .iter()
        .map(|node| (node.job_id_local.clone(), node))
        .collect::<BTreeMap<_, _>>();
    let mut seen = BTreeSet::<String>::new();
    let mut queue = nodes_by_job_id
        .get(failed_job_id)
        .map(|node| node.dependents.iter().cloned().collect::<VecDeque<_>>())
        .ok_or_else(|| {
            anyhow!("HPC dependency simulation is missing failed job `{failed_job_id}`")
        })?;
    let mut descendants = Vec::new();

    while let Some(job_id_local) = queue.pop_front() {
        if !seen.insert(job_id_local.clone()) {
            continue;
        }
        let node = nodes_by_job_id.get(&job_id_local).ok_or_else(|| {
            anyhow!("HPC dependency simulation descendant walk is missing job `{job_id_local}`")
        })?;
        descendants.push(job_id_local);
        for dependent in &node.dependents {
            queue.push_back(dependent.clone());
        }
    }

    Ok(descendants)
}

fn simulate_jobs(
    graph: &LocalHpcJobGraph,
    case_id: &str,
    failed_job_id: &str,
    descendant_set: &BTreeSet<String>,
) -> Result<Vec<LocalHpcDependencySimulationJob>> {
    let nodes_by_job_id = graph
        .nodes
        .iter()
        .map(|node| (node.job_id_local.clone(), node))
        .collect::<BTreeMap<_, _>>();
    let mut status_by_job_id = BTreeMap::<String, LocalHpcDependencySimulationStatus>::new();
    let mut finish_by_job_id = BTreeMap::<String, u64>::new();
    let mut jobs = Vec::with_capacity(graph.job_count);

    for job_id_local in &graph.topological_order {
        let node = nodes_by_job_id.get(job_id_local).ok_or_else(|| {
            anyhow!("HPC dependency simulation case `{case_id}` is missing job `{job_id_local}`")
        })?;
        let blocking_dependencies = node
            .dependencies
            .iter()
            .filter(|dependency| {
                status_by_job_id.get(*dependency).is_some_and(|status| {
                    matches!(
                        status,
                        LocalHpcDependencySimulationStatus::Failed
                            | LocalHpcDependencySimulationStatus::Blocked
                    )
                })
            })
            .cloned()
            .collect::<Vec<_>>();
        let max_dependency_finish = node
            .dependencies
            .iter()
            .filter_map(|dependency| finish_by_job_id.get(dependency).copied())
            .max()
            .unwrap_or(0);
        let status = if node.job_id_local == failed_job_id {
            LocalHpcDependencySimulationStatus::Failed
        } else if blocking_dependencies.is_empty() {
            LocalHpcDependencySimulationStatus::Completed
        } else {
            LocalHpcDependencySimulationStatus::Blocked
        };
        let (start_second, finish_second, duration_seconds) =
            if matches!(status, LocalHpcDependencySimulationStatus::Blocked) {
                (0, 0, 0)
            } else {
                (max_dependency_finish, max_dependency_finish + 1, 1)
            };
        status_by_job_id.insert(node.job_id_local.clone(), status);
        finish_by_job_id.insert(node.job_id_local.clone(), finish_second);
        let descendant_of_failed_job = descendant_set.contains(&node.job_id_local);
        jobs.push(LocalHpcDependencySimulationJob {
            job_id_local: node.job_id_local.clone(),
            job_kind: node.job_kind.clone(),
            result_id: node.result_id.clone(),
            pipeline_id: node.pipeline_id.clone(),
            node_id: node.node_id.clone(),
            stage_id: node.stage_id.clone(),
            dependency_count: node.dependency_count,
            dependencies: node.dependencies.clone(),
            dependent_count: node.dependent_count,
            dependents: node.dependents.clone(),
            blocking_dependencies,
            status: status.as_str().to_string(),
            descendant_of_failed_job,
            unrelated_to_failed_job: node.job_id_local != failed_job_id
                && !descendant_of_failed_job,
            start_second,
            finish_second,
            duration_seconds,
        });
    }

    Ok(jobs)
}
