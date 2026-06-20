use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::local_hpc_job_graph::{collect_local_hpc_job_graph, LocalHpcJobGraph};
use super::path_resolution::{ensure_path_stays_within_benchmark_runs_root, BenchmarkPathResolver};
use crate::commands::cli::parse;
use crate::commands::cli::render;

const LOCAL_HPC_DEPENDENCY_SIMULATION_SCHEMA_VERSION: &str =
    "bijux.bench.local_hpc_dependency_simulation.v1";
pub(crate) const DEFAULT_HPC_DEPENDENCY_SIMULATION_PATH: &str =
    "runs/bench/hpc-dry-run/slurm-dependency-simulation.json";
const RELATEDNESS_IBD_FAILURE_JOB_ID: &str = "pipeline:relatedness-segments-vcf:vcf.ibd";
const RELATEDNESS_BLOCKED_DESCENDANT_JOB_ID: &str =
    "pipeline:relatedness-segments-vcf:vcf.demography";
const RELATEDNESS_CONTINUED_SIBLING_JOB_ID: &str = "pipeline:relatedness-segments-vcf:vcf.roh";
const REFERENCE_PANEL_FAILURE_JOB_ID: &str =
    "pipeline:reference-panel-imputation:vcf.prepare_reference_panel";
const REFERENCE_PANEL_BLOCKED_DESCENDANT_JOB_ID: &str =
    "pipeline:reference-panel-imputation:vcf.phasing";
const REFERENCE_PANEL_CONTINUED_SIBLING_JOB_ID: &str = "pipeline:reference-panel-imputation:vcf.qc";

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcDependencySimulationJob {
    pub(crate) job_id_local: String,
    pub(crate) job_kind: String,
    pub(crate) result_id: Option<String>,
    pub(crate) pipeline_id: Option<String>,
    pub(crate) node_id: Option<String>,
    pub(crate) stage_id: String,
    pub(crate) dependency_count: usize,
    pub(crate) dependencies: Vec<String>,
    pub(crate) dependent_count: usize,
    pub(crate) dependents: Vec<String>,
    pub(crate) blocking_dependencies: Vec<String>,
    pub(crate) status: String,
    pub(crate) descendant_of_failed_job: bool,
    pub(crate) unrelated_to_failed_job: bool,
    pub(crate) start_second: u64,
    pub(crate) finish_second: u64,
    pub(crate) duration_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcDependencySimulationCase {
    pub(crate) case_id: String,
    pub(crate) failed_job_id: String,
    pub(crate) blocked_descendant_job_id: String,
    pub(crate) continued_sibling_job_id: String,
    pub(crate) descendant_job_ids: Vec<String>,
    pub(crate) blocked_job_ids: Vec<String>,
    pub(crate) continued_unrelated_job_ids: Vec<String>,
    pub(crate) continued_benchmark_job_ids: Vec<String>,
    pub(crate) simulated_makespan_seconds: u64,
    pub(crate) blocked_only_descendants: bool,
    pub(crate) unrelated_branches_continue: bool,
    pub(crate) benchmark_jobs_continue: bool,
    pub(crate) proven: bool,
    pub(crate) jobs: Vec<LocalHpcDependencySimulationJob>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcDependencySimulationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) manifest_path: String,
    pub(crate) output_path: String,
    pub(crate) job_count: usize,
    pub(crate) benchmark_job_count: usize,
    pub(crate) essential_pipeline_job_count: usize,
    pub(crate) dependency_count: usize,
    pub(crate) case_count: usize,
    pub(crate) all_cases_proven: bool,
    pub(crate) cases: Vec<LocalHpcDependencySimulationCase>,
}

pub(crate) fn run_render_hpc_dependency_simulation(
    args: &parse::BenchLocalRenderHpcDependencySimulationArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let output_path = args.output.clone().unwrap_or_else(|| {
        benchmark_paths.resolve_repo_relative(Path::new(DEFAULT_HPC_DEPENDENCY_SIMULATION_PATH))
    });
    let report = render_hpc_dependency_simulation(&repo_root, output_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn run_validate_hpc_dependency_simulation(
    args: &parse::BenchLocalValidateHpcDependencySimulationArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let manifest_path = args.manifest.clone().unwrap_or_else(|| {
        benchmark_paths.resolve_repo_relative(Path::new(DEFAULT_HPC_DEPENDENCY_SIMULATION_PATH))
    });
    let report = validate_hpc_dependency_simulation_path(&repo_root, &manifest_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_hpc_dependency_simulation(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<LocalHpcDependencySimulationReport> {
    let absolute_output =
        if output_path.is_absolute() { output_path } else { repo_root.join(&output_path) };
    let report = build_hpc_dependency_simulation(repo_root, &absolute_output)?;
    if let Some(parent) = absolute_output.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(&absolute_output, &report)?;
    Ok(report)
}

pub(crate) fn validate_hpc_dependency_simulation_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<LocalHpcDependencySimulationReport> {
    let absolute_manifest_path = if manifest_path.is_absolute() {
        manifest_path.to_path_buf()
    } else {
        repo_root.join(manifest_path)
    };
    let report = build_hpc_dependency_simulation(repo_root, &absolute_manifest_path)?;
    let observed = fs::read(&absolute_manifest_path)
        .with_context(|| format!("read {}", absolute_manifest_path.display()))?;
    let expected = serde_json::to_vec_pretty(&report)
        .context("serialize governed HPC dependency simulation report")?;
    if observed != expected {
        return Err(anyhow!(
            "HPC dependency simulation `{}` drifted from governed dry-run inputs; rerun `bijux-dna bench local render-hpc-dependency-simulation --output {}`",
            absolute_manifest_path.display(),
            report.output_path
        ));
    }
    Ok(report)
}

fn build_hpc_dependency_simulation(
    repo_root: &Path,
    absolute_output_path: &Path,
) -> Result<LocalHpcDependencySimulationReport> {
    ensure_path_stays_within_benchmark_runs_root(
        repo_root,
        absolute_output_path,
        "HPC dependency simulation output",
    )?;
    let graph = collect_local_hpc_job_graph(repo_root)?;
    let cases = vec![
        simulate_failure_case(
            &graph,
            "relatedness_ibd_failure_isolates_demography_from_roh",
            RELATEDNESS_IBD_FAILURE_JOB_ID,
            RELATEDNESS_BLOCKED_DESCENDANT_JOB_ID,
            RELATEDNESS_CONTINUED_SIBLING_JOB_ID,
        )?,
        simulate_failure_case(
            &graph,
            "reference_panel_prepare_failure_blocks_phasing_without_blocking_qc",
            REFERENCE_PANEL_FAILURE_JOB_ID,
            REFERENCE_PANEL_BLOCKED_DESCENDANT_JOB_ID,
            REFERENCE_PANEL_CONTINUED_SIBLING_JOB_ID,
        )?,
    ];
    let all_cases_proven = cases.iter().all(|case_report| case_report.proven);
    let report = LocalHpcDependencySimulationReport {
        schema_version: LOCAL_HPC_DEPENDENCY_SIMULATION_SCHEMA_VERSION,
        manifest_path: graph.manifest_path.clone(),
        output_path: path_relative_to_repo(repo_root, absolute_output_path),
        job_count: graph.job_count,
        benchmark_job_count: graph.benchmark_job_count,
        essential_pipeline_job_count: graph.essential_pipeline_job_count,
        dependency_count: graph.dependency_count,
        case_count: cases.len(),
        all_cases_proven,
        cases,
    };
    ensure_hpc_dependency_simulation_contract(&graph, &report)?;
    Ok(report)
}

fn simulate_failure_case(
    graph: &LocalHpcJobGraph,
    case_id: &str,
    failed_job_id: &str,
    blocked_descendant_job_id: &str,
    continued_sibling_job_id: &str,
) -> Result<LocalHpcDependencySimulationCase> {
    let nodes_by_job_id = graph
        .nodes
        .iter()
        .map(|node| (node.job_id_local.clone(), node))
        .collect::<BTreeMap<_, _>>();
    let _failed_node = nodes_by_job_id.get(failed_job_id).ok_or_else(|| {
        anyhow!(
            "HPC dependency simulation case `{case_id}` is missing failed job `{failed_job_id}`"
        )
    })?;
    let descendant_job_ids = descendants_of(graph, failed_job_id)?;
    let descendant_set = descendant_job_ids.iter().cloned().collect::<BTreeSet<_>>();
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
        let (status, start_second, finish_second, duration_seconds) =
            if node.job_id_local == failed_job_id {
                (
                    LocalHpcDependencySimulationStatus::Failed,
                    max_dependency_finish,
                    max_dependency_finish + 1,
                    1,
                )
            } else if !blocking_dependencies.is_empty() {
                (LocalHpcDependencySimulationStatus::Blocked, 0, 0, 0)
            } else {
                (
                    LocalHpcDependencySimulationStatus::Completed,
                    max_dependency_finish,
                    max_dependency_finish + 1,
                    1,
                )
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

fn ensure_hpc_dependency_simulation_contract(
    graph: &LocalHpcJobGraph,
    report: &LocalHpcDependencySimulationReport,
) -> Result<()> {
    if report.job_count != graph.job_count
        || report.benchmark_job_count != graph.benchmark_job_count
        || report.essential_pipeline_job_count != graph.essential_pipeline_job_count
        || report.dependency_count != graph.dependency_count
    {
        return Err(anyhow!(
            "HPC dependency simulation report must stay aligned with the governed HPC job graph"
        ));
    }
    if report.case_count != report.cases.len() || report.cases.is_empty() {
        return Err(anyhow!(
            "HPC dependency simulation report must keep at least one proven case row"
        ));
    }
    let valid_job_ids =
        graph.nodes.iter().map(|node| node.job_id_local.as_str()).collect::<BTreeSet<_>>();
    for case_report in &report.cases {
        if case_report.jobs.len() != graph.job_count {
            return Err(anyhow!(
                "HPC dependency simulation case `{}` must keep one job row per governed HPC job",
                case_report.case_id
            ));
        }
        for job_id_local in case_report
            .blocked_job_ids
            .iter()
            .chain(case_report.continued_unrelated_job_ids.iter())
            .chain(case_report.continued_benchmark_job_ids.iter())
            .chain(case_report.descendant_job_ids.iter())
            .chain(std::iter::once(&case_report.failed_job_id))
        {
            if !valid_job_ids.contains(job_id_local.as_str()) {
                return Err(anyhow!(
                    "HPC dependency simulation case `{}` references unknown job `{job_id_local}`",
                    case_report.case_id
                ));
            }
        }
        if !case_report.proven {
            return Err(anyhow!(
                "HPC dependency simulation case `{}` did not prove the required descendant-only blocking invariant",
                case_report.case_id
            ));
        }
    }
    Ok(())
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).map_or_else(
        |_| path.to_string_lossy().replace('\\', "/"),
        |relative| relative.to_string_lossy().replace('\\', "/"),
    )
}
