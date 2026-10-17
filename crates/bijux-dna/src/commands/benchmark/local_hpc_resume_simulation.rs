use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_api::v1::api::bench::{
    build_benchmark_local_hpc_job_graph, validate_benchmark_local_hpc_resume_simulation,
    BenchmarkLocalHpcResumeSimulationBehavior, BenchmarkLocalHpcResumeSimulationJob,
    BenchmarkLocalHpcResumeSimulationReport, BENCHMARK_LOCAL_HPC_RESUME_DEPENDENCY_JOB_ID,
    BENCHMARK_LOCAL_HPC_RESUME_FAILED_JOB_ID, BENCHMARK_LOCAL_HPC_RESUME_MISSING_MANIFEST_JOB_ID,
    BENCHMARK_LOCAL_HPC_RESUME_SIMULATION_SCHEMA_V1,
    BENCHMARK_LOCAL_HPC_RESUME_STALE_OUTPUT_JOB_ID, BENCHMARK_LOCAL_HPC_RESUME_UNRELATED_JOB_ID,
    BENCHMARK_LOCAL_HPC_RESUME_VALID_JOB_ID,
};

use super::local_all_domain_slurm_submit_manifest::{
    BenchLocalAllDomainSlurmSubmitJob, DEFAULT_ALL_DOMAIN_SLURM_SUBMIT_MANIFEST_PATH,
};
use super::local_hpc_job_completion::{
    classify_local_hpc_job_completion, resolve_local_hpc_job_result_paths,
    LocalHpcJobCompletionState,
};
use super::local_hpc_job_graph::LocalHpcJobGraphNode;
use super::local_hpc_selected_jobs::load_local_hpc_selected_jobs;
use super::local_hpc_simulation_tree::{
    remap_job_to_simulation_root, remove_one_declared_output, remove_stage_result_manifest,
    write_stage_result_manifest,
};
use super::local_stage_result_manifest::{path_relative_to_repo, BenchStageResultStatus};
use super::path_resolution::{ensure_path_stays_within_benchmark_runs_root, BenchmarkPathResolver};
use crate::commands::cli::parse;
use crate::commands::cli::render;

const LOCAL_HPC_RESUME_SIMULATION_SCHEMA_VERSION: &str =
    BENCHMARK_LOCAL_HPC_RESUME_SIMULATION_SCHEMA_V1;
pub(crate) const DEFAULT_HPC_RESUME_SIMULATION_PATH: &str =
    "artifacts/runs/bench/hpc-dry-run/resume-simulation.json";
const DEFAULT_HPC_RESUME_SIMULATION_TREE_NAME: &str = "resume-simulation-tree";
const FAILED_STAGE_RESULT_JOB_ID: &str = BENCHMARK_LOCAL_HPC_RESUME_FAILED_JOB_ID;
const DEPENDENCY_RERUN_JOB_ID: &str = BENCHMARK_LOCAL_HPC_RESUME_DEPENDENCY_JOB_ID;
const UNRELATED_VALID_JOB_ID: &str = BENCHMARK_LOCAL_HPC_RESUME_UNRELATED_JOB_ID;
const MISSING_MANIFEST_JOB_ID: &str = BENCHMARK_LOCAL_HPC_RESUME_MISSING_MANIFEST_JOB_ID;
const STALE_PARTIAL_OUTPUT_JOB_ID: &str = BENCHMARK_LOCAL_HPC_RESUME_STALE_OUTPUT_JOB_ID;
const VALID_COMPLETED_JOB_ID: &str = BENCHMARK_LOCAL_HPC_RESUME_VALID_JOB_ID;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LocalHpcResumeAction {
    Skip,
    Rerun,
}

impl LocalHpcResumeAction {
    fn as_str(self) -> &'static str {
        match self {
            Self::Skip => "skip",
            Self::Rerun => "rerun",
        }
    }
}

pub(crate) type LocalHpcResumeSimulationBehavior = BenchmarkLocalHpcResumeSimulationBehavior;
pub(crate) type LocalHpcResumeSimulationJob = BenchmarkLocalHpcResumeSimulationJob;
pub(crate) type LocalHpcResumeSimulationReport = BenchmarkLocalHpcResumeSimulationReport;

pub(crate) fn run_render_hpc_resume_simulation(
    args: &parse::BenchLocalRenderHpcResumeSimulationArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let output_path = args.output.clone().unwrap_or_else(|| {
        benchmark_paths.resolve_repo_relative(Path::new(DEFAULT_HPC_RESUME_SIMULATION_PATH))
    });
    let report = render_hpc_resume_simulation(&repo_root, output_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn run_validate_hpc_resume_simulation(
    args: &parse::BenchLocalValidateHpcResumeSimulationArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let manifest_path = args.manifest.clone().unwrap_or_else(|| {
        benchmark_paths.resolve_repo_relative(Path::new(DEFAULT_HPC_RESUME_SIMULATION_PATH))
    });
    let report = validate_hpc_resume_simulation_path(&repo_root, &manifest_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_hpc_resume_simulation(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<LocalHpcResumeSimulationReport> {
    let absolute_output =
        if output_path.is_absolute() { output_path } else { repo_root.join(&output_path) };
    let report = build_hpc_resume_simulation(repo_root, &absolute_output)?;
    if let Some(parent) = absolute_output.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(&absolute_output, &report)?;
    Ok(report)
}

pub(crate) fn collect_hpc_resume_simulation_with_jobs<
    J: AsRef<[BenchLocalAllDomainSlurmSubmitJob]>,
>(
    repo_root: &Path,
    acquire_jobs: impl FnOnce() -> Result<J>,
) -> Result<LocalHpcResumeSimulationReport> {
    let benchmark_paths = BenchmarkPathResolver::new(repo_root, None);
    let output_path =
        benchmark_paths.resolve_repo_relative(Path::new(DEFAULT_HPC_RESUME_SIMULATION_PATH));
    build_hpc_resume_simulation_with_jobs(repo_root, &output_path, || {
        acquire_jobs().map(|jobs| jobs.as_ref().to_vec())
    })
}

pub(crate) fn validate_hpc_resume_simulation_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<LocalHpcResumeSimulationReport> {
    let absolute_manifest_path = if manifest_path.is_absolute() {
        manifest_path.to_path_buf()
    } else {
        repo_root.join(manifest_path)
    };
    let report = build_hpc_resume_simulation(repo_root, &absolute_manifest_path)?;
    let observed = fs::read(&absolute_manifest_path)
        .with_context(|| format!("read {}", absolute_manifest_path.display()))?;
    let expected =
        serde_json::to_vec_pretty(&report).context("serialize governed HPC resume simulation")?;
    if observed != expected {
        return Err(anyhow!(
            "HPC resume simulation `{}` drifted from governed dry-run inputs; rerun `bijux-dna bench local render-hpc-resume-simulation --output {}`",
            absolute_manifest_path.display(),
            report.output_path
        ));
    }
    Ok(report)
}

fn build_hpc_resume_simulation(
    repo_root: &Path,
    absolute_output_path: &Path,
) -> Result<LocalHpcResumeSimulationReport> {
    build_hpc_resume_simulation_with_jobs(repo_root, absolute_output_path, || {
        load_local_hpc_selected_jobs(repo_root)
    })
}

fn build_hpc_resume_simulation_with_jobs(
    repo_root: &Path,
    absolute_output_path: &Path,
    acquire_jobs: impl FnOnce() -> Result<Vec<BenchLocalAllDomainSlurmSubmitJob>>,
) -> Result<LocalHpcResumeSimulationReport> {
    ensure_path_stays_within_benchmark_runs_root(
        repo_root,
        absolute_output_path,
        "HPC resume simulation output",
    )?;
    let simulation_root = absolute_output_path
        .parent()
        .ok_or_else(|| {
            anyhow!(
                "HPC resume simulation output `{}` has no parent",
                absolute_output_path.display()
            )
        })?
        .join(DEFAULT_HPC_RESUME_SIMULATION_TREE_NAME);
    ensure_path_stays_within_benchmark_runs_root(
        repo_root,
        &simulation_root,
        "HPC resume simulation tree",
    )?;

    if simulation_root.exists() {
        fs::remove_dir_all(&simulation_root)
            .with_context(|| format!("remove {}", simulation_root.display()))?;
    }
    fs::create_dir_all(&simulation_root)
        .with_context(|| format!("create {}", simulation_root.display()))?;

    let selected_jobs = acquire_jobs()?;
    let graph = build_benchmark_local_hpc_job_graph(
        DEFAULT_ALL_DOMAIN_SLURM_SUBMIT_MANIFEST_PATH,
        selected_jobs.clone(),
    )?;
    let jobs_by_id = selected_jobs
        .into_iter()
        .map(|job| (job.job_id_local.clone(), job))
        .collect::<BTreeMap<_, _>>();
    let nodes_by_id = graph
        .nodes
        .iter()
        .map(|node| (node.job_id_local.clone(), node))
        .collect::<BTreeMap<_, _>>();

    validate_seeded_job_presence(&jobs_by_id, &nodes_by_id)?;

    let simulated_jobs = jobs_by_id
        .iter()
        .map(|(job_id_local, job)| {
            remap_job_to_simulation_root(repo_root, &simulation_root, job)
                .map(|simulated_job| (job_id_local.clone(), simulated_job))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;

    for job in simulated_jobs.values() {
        write_stage_result_manifest(repo_root, job, BenchStageResultStatus::Succeeded)?;
    }
    rewrite_stage_result_status(
        repo_root,
        simulated_jobs.get(FAILED_STAGE_RESULT_JOB_ID).expect("validated seeded job"),
        BenchStageResultStatus::Failed,
    )?;
    remove_stage_result_manifest(
        repo_root,
        simulated_jobs.get(MISSING_MANIFEST_JOB_ID).expect("validated seeded job"),
    )?;
    remove_one_declared_output(
        repo_root,
        simulated_jobs.get(STALE_PARTIAL_OUTPUT_JOB_ID).expect("validated seeded job"),
    )?;

    let mut rerun_job_ids = BTreeSet::<String>::new();
    let mut jobs = Vec::with_capacity(graph.job_count);
    let mut valid_completed_job_count = 0usize;
    let mut failed_stage_result_job_count = 0usize;
    let mut invalid_stage_result_job_count = 0usize;
    let mut missing_stage_result_job_count = 0usize;
    let mut stale_partial_output_job_count = 0usize;
    let mut skip_job_count = 0usize;
    let mut dependency_rerun_job_count = 0usize;

    for job_id_local in &graph.topological_order {
        let simulated_job = simulated_jobs.get(job_id_local).ok_or_else(|| {
            anyhow!("HPC resume simulation is missing simulated job `{job_id_local}`")
        })?;
        let node = nodes_by_id.get(job_id_local).ok_or_else(|| {
            anyhow!("HPC resume simulation is missing graph node `{job_id_local}`")
        })?;
        let result_paths = resolve_local_hpc_job_result_paths(repo_root, simulated_job)?;
        let classification =
            classify_local_hpc_job_completion(repo_root, simulated_job, &result_paths);
        match classification.state {
            LocalHpcJobCompletionState::ValidCompleted => valid_completed_job_count += 1,
            LocalHpcJobCompletionState::FailedStageResultManifest => {
                failed_stage_result_job_count += 1;
            }
            LocalHpcJobCompletionState::InvalidStageResultManifest => {
                invalid_stage_result_job_count += 1;
            }
            LocalHpcJobCompletionState::MissingStageResultManifest => {
                missing_stage_result_job_count += 1;
            }
            LocalHpcJobCompletionState::StalePartialOutputs => stale_partial_output_job_count += 1,
        }

        let rerun_dependencies = node
            .dependencies
            .iter()
            .filter(|dependency| rerun_job_ids.contains(*dependency))
            .cloned()
            .collect::<Vec<_>>();
        let reason = if classification.state != LocalHpcJobCompletionState::ValidCompleted {
            classification.state.as_str().to_string()
        } else if !rerun_dependencies.is_empty() {
            dependency_rerun_job_count += 1;
            "upstream_dependency_rerun".to_string()
        } else {
            "valid_completed".to_string()
        };
        let resume_action = if reason == "valid_completed" {
            skip_job_count += 1;
            LocalHpcResumeAction::Skip
        } else {
            rerun_job_ids.insert(job_id_local.clone());
            LocalHpcResumeAction::Rerun
        };

        jobs.push(LocalHpcResumeSimulationJob {
            job_id_local: node.job_id_local.clone(),
            job_kind: node.job_kind.clone(),
            result_id: node.result_id.clone(),
            pipeline_id: node.pipeline_id.clone(),
            node_id: node.node_id.clone(),
            stage_id: node.stage_id.clone(),
            tool_id: node.tool_id.clone(),
            dependency_count: node.dependency_count,
            dependencies: node.dependencies.clone(),
            completion_state: classification.state.as_str().to_string(),
            completion_detail: classification.detail,
            manifest_valid: classification.manifest_valid,
            runtime_status: classification.runtime_status,
            declared_output_count: classification.declared_output_count,
            present_output_count: classification.present_output_count,
            missing_output_paths: classification.missing_output_paths,
            resume_action: resume_action.as_str().to_string(),
            reason,
            rerun_dependencies,
            result_root: path_relative_to_repo(repo_root, &result_paths.result_root),
            stage_result_manifest_path: path_relative_to_repo(
                repo_root,
                &result_paths.stage_result_manifest_path,
            ),
        });
    }

    let behavior = build_behavior(&jobs)?;
    let report = LocalHpcResumeSimulationReport {
        schema_version: LOCAL_HPC_RESUME_SIMULATION_SCHEMA_VERSION.to_string(),
        manifest_path: graph.manifest_path,
        output_path: path_relative_to_repo(repo_root, absolute_output_path),
        simulation_root: path_relative_to_repo(repo_root, &simulation_root),
        job_count: graph.job_count,
        benchmark_job_count: graph.benchmark_job_count,
        essential_pipeline_job_count: graph.essential_pipeline_job_count,
        dependency_count: graph.dependency_count,
        valid_completed_job_count,
        failed_stage_result_job_count,
        invalid_stage_result_job_count,
        missing_stage_result_job_count,
        stale_partial_output_job_count,
        skip_job_count,
        rerun_job_count: rerun_job_ids.len(),
        dependency_rerun_job_count,
        behavior,
        jobs,
    };
    validate_benchmark_local_hpc_resume_simulation(&report)?;
    Ok(report)
}

fn validate_seeded_job_presence(
    jobs_by_id: &BTreeMap<String, BenchLocalAllDomainSlurmSubmitJob>,
    nodes_by_id: &BTreeMap<String, &LocalHpcJobGraphNode>,
) -> Result<()> {
    for job_id_local in [
        FAILED_STAGE_RESULT_JOB_ID,
        DEPENDENCY_RERUN_JOB_ID,
        UNRELATED_VALID_JOB_ID,
        MISSING_MANIFEST_JOB_ID,
        STALE_PARTIAL_OUTPUT_JOB_ID,
        VALID_COMPLETED_JOB_ID,
    ] {
        if !jobs_by_id.contains_key(job_id_local) || !nodes_by_id.contains_key(job_id_local) {
            return Err(anyhow!(
                "HPC resume simulation requires governed seeded job `{job_id_local}`"
            ));
        }
    }
    Ok(())
}

fn rewrite_stage_result_status(
    repo_root: &Path,
    job: &BenchLocalAllDomainSlurmSubmitJob,
    status: BenchStageResultStatus,
) -> Result<()> {
    write_stage_result_manifest(repo_root, job, status)
}

fn build_behavior(
    jobs: &[LocalHpcResumeSimulationJob],
) -> Result<LocalHpcResumeSimulationBehavior> {
    let jobs_by_id =
        jobs.iter().map(|job| (job.job_id_local.as_str(), job)).collect::<BTreeMap<_, _>>();
    let failed_stage_result = jobs_by_id.get(FAILED_STAGE_RESULT_JOB_ID).ok_or_else(|| {
        anyhow!("HPC resume simulation is missing `{FAILED_STAGE_RESULT_JOB_ID}`")
    })?;
    let dependency_rerun = jobs_by_id
        .get(DEPENDENCY_RERUN_JOB_ID)
        .ok_or_else(|| anyhow!("HPC resume simulation is missing `{DEPENDENCY_RERUN_JOB_ID}`"))?;
    let unrelated_valid = jobs_by_id
        .get(UNRELATED_VALID_JOB_ID)
        .ok_or_else(|| anyhow!("HPC resume simulation is missing `{UNRELATED_VALID_JOB_ID}`"))?;
    let missing_manifest = jobs_by_id
        .get(MISSING_MANIFEST_JOB_ID)
        .ok_or_else(|| anyhow!("HPC resume simulation is missing `{MISSING_MANIFEST_JOB_ID}`"))?;
    let stale_partial_output = jobs_by_id.get(STALE_PARTIAL_OUTPUT_JOB_ID).ok_or_else(|| {
        anyhow!("HPC resume simulation is missing `{STALE_PARTIAL_OUTPUT_JOB_ID}`")
    })?;
    let valid_completed = jobs_by_id
        .get(VALID_COMPLETED_JOB_ID)
        .ok_or_else(|| anyhow!("HPC resume simulation is missing `{VALID_COMPLETED_JOB_ID}`"))?;

    let behavior = LocalHpcResumeSimulationBehavior {
        failed_stage_result_job_id: FAILED_STAGE_RESULT_JOB_ID.to_string(),
        dependency_rerun_job_id: DEPENDENCY_RERUN_JOB_ID.to_string(),
        unrelated_valid_job_id: UNRELATED_VALID_JOB_ID.to_string(),
        missing_manifest_job_id: MISSING_MANIFEST_JOB_ID.to_string(),
        stale_partial_output_job_id: STALE_PARTIAL_OUTPUT_JOB_ID.to_string(),
        valid_completed_job_id: VALID_COMPLETED_JOB_ID.to_string(),
        failed_stage_result_reruns: failed_stage_result.completion_state
            == LocalHpcJobCompletionState::FailedStageResultManifest.as_str()
            && failed_stage_result.resume_action == LocalHpcResumeAction::Rerun.as_str(),
        dependency_rerun_propagates: dependency_rerun.resume_action
            == LocalHpcResumeAction::Rerun.as_str()
            && dependency_rerun.reason == "upstream_dependency_rerun"
            && dependency_rerun
                .rerun_dependencies
                .iter()
                .any(|dependency| dependency == FAILED_STAGE_RESULT_JOB_ID),
        unrelated_valid_job_skips: unrelated_valid.completion_state
            == LocalHpcJobCompletionState::ValidCompleted.as_str()
            && unrelated_valid.resume_action == LocalHpcResumeAction::Skip.as_str(),
        missing_manifest_reruns: missing_manifest.completion_state
            == LocalHpcJobCompletionState::MissingStageResultManifest.as_str()
            && missing_manifest.resume_action == LocalHpcResumeAction::Rerun.as_str(),
        stale_partial_output_reruns: stale_partial_output.completion_state
            == LocalHpcJobCompletionState::StalePartialOutputs.as_str()
            && stale_partial_output.resume_action == LocalHpcResumeAction::Rerun.as_str(),
        valid_completed_job_skips: valid_completed.completion_state
            == LocalHpcJobCompletionState::ValidCompleted.as_str()
            && valid_completed.resume_action == LocalHpcResumeAction::Skip.as_str(),
        proven: false,
    };
    Ok(LocalHpcResumeSimulationBehavior {
        proven: behavior.failed_stage_result_reruns
            && behavior.dependency_rerun_propagates
            && behavior.unrelated_valid_job_skips
            && behavior.missing_manifest_reruns
            && behavior.stale_partial_output_reruns
            && behavior.valid_completed_job_skips,
        ..behavior
    })
}
