use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::local_all_domain_slurm_submit_manifest::BenchLocalAllDomainSlurmSubmitJob;
use super::local_hpc_job_completion::{
    classify_local_hpc_job_completion, resolve_local_hpc_job_result_paths,
    LocalHpcJobCompletionState,
};
use super::local_hpc_job_graph::{collect_local_hpc_job_graph, LocalHpcJobGraphNode};
use super::local_hpc_selected_jobs::load_local_hpc_selected_jobs;
use super::local_stage_result_manifest::{
    path_relative_to_repo, validate_stage_result_manifest, BenchStageResultCommandV1,
    BenchStageResultManifestV1, BenchStageResultOutputV1, BenchStageResultResourceMetricSource,
    BenchStageResultResourceMetricsV1, BenchStageResultRuntimeV1, BenchStageResultStatus,
    BenchStageResultToolV1, BENCH_STAGE_RESULT_SCHEMA_VERSION,
};
use super::path_resolution::{ensure_path_stays_within_benchmark_runs_root, BenchmarkPathResolver};
use crate::commands::cli::parse;
use crate::commands::cli::render;

const LOCAL_HPC_RESUME_SIMULATION_SCHEMA_VERSION: &str =
    "bijux.bench.local_hpc_resume_simulation.v1";
pub(crate) const DEFAULT_HPC_RESUME_SIMULATION_PATH: &str =
    "runs/bench/hpc-dry-run/resume-simulation.json";
const DEFAULT_HPC_RESUME_SIMULATION_TREE_NAME: &str = "resume-simulation-tree";
const FAILED_STAGE_RESULT_JOB_ID: &str = "pipeline:relatedness-segments-vcf:vcf.ibd";
const DEPENDENCY_RERUN_JOB_ID: &str = "pipeline:relatedness-segments-vcf:vcf.demography";
const UNRELATED_VALID_JOB_ID: &str = "pipeline:relatedness-segments-vcf:vcf.roh";
const MISSING_MANIFEST_JOB_ID: &str =
    "benchmark:vcf:vcf_production_regression:vcf.stats:vcf_cohort:bcftools";
const STALE_PARTIAL_OUTPUT_JOB_ID: &str =
    "benchmark:bam:corpus-01-adna-bam-mini:bam.contamination:sample-set:contammix";
const VALID_COMPLETED_JOB_ID: &str =
    "benchmark:vcf:vcf_production_regression:vcf.qc:vcf_cohort:bcftools";

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcResumeSimulationBehavior {
    pub(crate) failed_stage_result_job_id: String,
    pub(crate) dependency_rerun_job_id: String,
    pub(crate) unrelated_valid_job_id: String,
    pub(crate) missing_manifest_job_id: String,
    pub(crate) stale_partial_output_job_id: String,
    pub(crate) valid_completed_job_id: String,
    pub(crate) failed_stage_result_reruns: bool,
    pub(crate) dependency_rerun_propagates: bool,
    pub(crate) unrelated_valid_job_skips: bool,
    pub(crate) missing_manifest_reruns: bool,
    pub(crate) stale_partial_output_reruns: bool,
    pub(crate) valid_completed_job_skips: bool,
    pub(crate) proven: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcResumeSimulationJob {
    pub(crate) job_id_local: String,
    pub(crate) job_kind: String,
    pub(crate) result_id: Option<String>,
    pub(crate) pipeline_id: Option<String>,
    pub(crate) node_id: Option<String>,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) dependency_count: usize,
    pub(crate) dependencies: Vec<String>,
    pub(crate) completion_state: String,
    pub(crate) completion_detail: String,
    pub(crate) manifest_valid: bool,
    pub(crate) runtime_status: Option<String>,
    pub(crate) declared_output_count: usize,
    pub(crate) present_output_count: usize,
    pub(crate) missing_output_paths: Vec<String>,
    pub(crate) resume_action: String,
    pub(crate) reason: String,
    pub(crate) rerun_dependencies: Vec<String>,
    pub(crate) result_root: String,
    pub(crate) stage_result_manifest_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcResumeSimulationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) manifest_path: String,
    pub(crate) output_path: String,
    pub(crate) simulation_root: String,
    pub(crate) job_count: usize,
    pub(crate) benchmark_job_count: usize,
    pub(crate) essential_pipeline_job_count: usize,
    pub(crate) dependency_count: usize,
    pub(crate) valid_completed_job_count: usize,
    pub(crate) failed_stage_result_job_count: usize,
    pub(crate) invalid_stage_result_job_count: usize,
    pub(crate) missing_stage_result_job_count: usize,
    pub(crate) stale_partial_output_job_count: usize,
    pub(crate) skip_job_count: usize,
    pub(crate) rerun_job_count: usize,
    pub(crate) dependency_rerun_job_count: usize,
    pub(crate) behavior: LocalHpcResumeSimulationBehavior,
    pub(crate) jobs: Vec<LocalHpcResumeSimulationJob>,
}

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

    let graph = collect_local_hpc_job_graph(repo_root)?;
    let selected_jobs = load_local_hpc_selected_jobs(repo_root)?;
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
        schema_version: LOCAL_HPC_RESUME_SIMULATION_SCHEMA_VERSION,
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
    ensure_hpc_resume_simulation_contract(&report)?;
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

fn remap_job_to_simulation_root(
    repo_root: &Path,
    simulation_root: &Path,
    job: &BenchLocalAllDomainSlurmSubmitJob,
) -> Result<BenchLocalAllDomainSlurmSubmitJob> {
    Ok(BenchLocalAllDomainSlurmSubmitJob {
        job_id_local: job.job_id_local.clone(),
        domain: job.domain.clone(),
        stage_id: job.stage_id.clone(),
        pipeline_id: job.pipeline_id.clone(),
        node_id: job.node_id.clone(),
        tool_id: job.tool_id.clone(),
        corpus_id: job.corpus_id.clone(),
        asset_profile_id: job.asset_profile_id.clone(),
        result_id: job.result_id.clone(),
        script_path: job.script_path.clone(),
        stdout: remap_repo_relative_path(repo_root, simulation_root, &job.stdout)?,
        stderr: remap_repo_relative_path(repo_root, simulation_root, &job.stderr)?,
        outputs: job
            .outputs
            .iter()
            .map(|path| remap_repo_relative_path(repo_root, simulation_root, path))
            .collect::<Result<Vec<_>>>()?,
        dependencies: job.dependencies.clone(),
        resources: job.resources.clone(),
    })
}

fn remap_repo_relative_path(
    repo_root: &Path,
    simulation_root: &Path,
    original: &str,
) -> Result<String> {
    let original_path = Path::new(original);
    let absolute_original = if original_path.is_absolute() {
        original_path.to_path_buf()
    } else {
        repo_root.join(original_path)
    };
    let repo_relative = absolute_original.strip_prefix(repo_root).map_err(|_| {
        anyhow!(
            "HPC resume simulation can only remap repo-owned paths, got `{}`",
            absolute_original.display()
        )
    })?;
    Ok(path_relative_to_repo(repo_root, &simulation_root.join(repo_relative)))
}

fn write_stage_result_manifest(
    repo_root: &Path,
    job: &BenchLocalAllDomainSlurmSubmitJob,
    status: BenchStageResultStatus,
) -> Result<()> {
    let result_paths = resolve_local_hpc_job_result_paths(repo_root, job)?;
    fs::create_dir_all(&result_paths.result_root)
        .with_context(|| format!("create {}", result_paths.result_root.display()))?;
    fs::write(repo_root.join(&job.stdout), "stdout\n")
        .with_context(|| format!("write {}", repo_root.join(&job.stdout).display()))?;
    fs::write(repo_root.join(&job.stderr), "stderr\n")
        .with_context(|| format!("write {}", repo_root.join(&job.stderr).display()))?;
    for output_path in &result_paths.declared_output_paths {
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        fs::write(output_path, "artifact\n")
            .with_context(|| format!("write {}", output_path.display()))?;
    }
    let exit_code = if status == BenchStageResultStatus::Succeeded { 0 } else { 1 };
    let manifest = BenchStageResultManifestV1 {
        schema_version: BENCH_STAGE_RESULT_SCHEMA_VERSION.to_string(),
        stage_id: job.stage_id.clone(),
        tool: BenchStageResultToolV1 { id: job.tool_id.clone() },
        command: BenchStageResultCommandV1 { rendered: format!("sbatch {}", job.script_path) },
        runtime: BenchStageResultRuntimeV1 {
            mode: "hpc_resume_simulation".to_string(),
            status,
            started_at: "1970-01-01T00:00:00Z".to_string(),
            finished_at: "1970-01-01T00:00:01Z".to_string(),
            elapsed_seconds: 1.0,
            exit_code,
        },
        resource_metrics: BenchStageResultResourceMetricsV1 {
            source: BenchStageResultResourceMetricSource::Estimated,
            memory_mb: Some(f64::from(job.resources.memory_mb)),
            cpu_threads: Some(job.resources.cpus_per_task),
        },
        outputs: result_paths
            .declared_output_paths
            .iter()
            .map(|path| BenchStageResultOutputV1 {
                artifact_id: path
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .unwrap_or("artifact")
                    .to_string(),
                declared_path: path_relative_to_repo(repo_root, path),
                realized_path: path_relative_to_repo(repo_root, path),
                role: "declared_output".to_string(),
                optional: false,
                exists: true,
            })
            .collect(),
    };
    validate_stage_result_manifest(&manifest)?;
    bijux_dna_infra::atomic_write_json(&result_paths.stage_result_manifest_path, &manifest)?;
    Ok(())
}

fn rewrite_stage_result_status(
    repo_root: &Path,
    job: &BenchLocalAllDomainSlurmSubmitJob,
    status: BenchStageResultStatus,
) -> Result<()> {
    write_stage_result_manifest(repo_root, job, status)
}

fn remove_stage_result_manifest(
    repo_root: &Path,
    job: &BenchLocalAllDomainSlurmSubmitJob,
) -> Result<()> {
    let result_paths = resolve_local_hpc_job_result_paths(repo_root, job)?;
    fs::remove_file(&result_paths.stage_result_manifest_path)
        .with_context(|| format!("remove {}", result_paths.stage_result_manifest_path.display()))?;
    Ok(())
}

fn remove_one_declared_output(
    repo_root: &Path,
    job: &BenchLocalAllDomainSlurmSubmitJob,
) -> Result<()> {
    let result_paths = resolve_local_hpc_job_result_paths(repo_root, job)?;
    let stale_output = result_paths.declared_output_paths.first().ok_or_else(|| {
        anyhow!(
            "HPC resume simulation stale-output seed requires at least one declared output for `{}`",
            job.job_id_local
        )
    })?;
    fs::remove_file(stale_output).with_context(|| format!("remove {}", stale_output.display()))?;
    Ok(())
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

fn ensure_hpc_resume_simulation_contract(report: &LocalHpcResumeSimulationReport) -> Result<()> {
    if report.job_count != report.benchmark_job_count + report.essential_pipeline_job_count {
        return Err(anyhow!(
            "HPC resume simulation job count must equal benchmark jobs plus essential pipeline jobs"
        ));
    }
    if report.jobs.len() != report.job_count {
        return Err(anyhow!(
            "HPC resume simulation must keep one governed row per selected HPC job"
        ));
    }
    if report.skip_job_count + report.rerun_job_count != report.job_count {
        return Err(anyhow!(
            "HPC resume simulation skip and rerun counts must partition the governed job set"
        ));
    }
    let classified_job_count = report.valid_completed_job_count
        + report.failed_stage_result_job_count
        + report.invalid_stage_result_job_count
        + report.missing_stage_result_job_count
        + report.stale_partial_output_job_count;
    if classified_job_count != report.job_count {
        return Err(anyhow!(
            "HPC resume simulation completion-state counts must partition the governed job set"
        ));
    }
    if report.dependency_rerun_job_count >= report.rerun_job_count {
        return Err(anyhow!(
            "HPC resume simulation must keep at least one direct rerun cause besides dependency propagation"
        ));
    }
    if !report.behavior.proven {
        return Err(anyhow!(
            "HPC resume simulation must prove valid skips, failed reruns, missing-manifest reruns, stale-output reruns, and dependency propagation"
        ));
    }
    Ok(())
}
