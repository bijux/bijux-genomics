use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use super::local_all_domain_slurm_submit_manifest::BenchLocalAllDomainSlurmSubmitJob;
use super::local_hpc_input_discovery::{
    collect_local_hpc_job_source_inputs, collect_local_hpc_stage_input_hints, materialize_stage_id,
    path_relative_to_repo, LocalHpcDiscoveredSourceInput, LocalHpcStageInputHint,
};
use super::local_hpc_job_resources::{
    load_local_hpc_job_resource_hints, resolve_local_hpc_job_resource_hint,
};
use super::local_hpc_selected_jobs::load_local_hpc_selected_jobs;
use super::path_resolution::{ensure_path_stays_within_benchmark_runs_root, BenchmarkPathResolver};
use crate::commands::cli::parse;
use crate::commands::cli::render;
use crate::commands::benchmark::readiness::all_domain_rendered_commands::{
    render_all_domain_commands, DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_PATH,
};
use crate::commands::benchmark::readiness::essential_pipeline_rendered_commands::{
    render_essential_pipeline_commands, DEFAULT_ESSENTIAL_PIPELINE_RENDERED_COMMANDS_PATH,
};

const LOCAL_HPC_SCRATCH_LAYOUT_SCHEMA_VERSION: &str = "bijux.bench.local_hpc_scratch_layout.v1";
pub(crate) const DEFAULT_HPC_SCRATCH_LAYOUT_PATH: &str =
    "runs/bench/hpc-dry-run/scratch-layout.json";
const CLEANUP_POLICY_ID: &str = "copy_back_outputs_remove_successful_scratch";
const DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_ARGV_PATH: &str =
    "benchmarks/readiness/rendered-commands-all-domains.argv.jsonl";
const DEFAULT_ESSENTIAL_PIPELINE_RENDERED_COMMANDS_ARGV_PATH: &str =
    "benchmarks/readiness/essential-pipelines-rendered-commands.argv.jsonl";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcScratchLayout {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) staging_root: String,
    pub(crate) scratch_root: String,
    pub(crate) selected_job_count: usize,
    pub(crate) benchmark_job_count: usize,
    pub(crate) essential_pipeline_job_count: usize,
    pub(crate) input_link_count: usize,
    pub(crate) jobs: Vec<LocalHpcScratchLayoutJob>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcScratchLayoutJob {
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
    pub(crate) scratch_root: String,
    pub(crate) input_links_root: String,
    pub(crate) output_root: String,
    pub(crate) log_root: String,
    pub(crate) stdout_path: String,
    pub(crate) stderr_path: String,
    pub(crate) resources: LocalHpcScratchJobResources,
    pub(crate) cleanup_policy: LocalHpcScratchCleanupPolicy,
    pub(crate) input_links: Vec<LocalHpcScratchInputLink>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcScratchJobResources {
    pub(crate) cpus_per_task: u32,
    pub(crate) memory_mb: u32,
    pub(crate) time_limit: String,
    pub(crate) scratch_gb: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcScratchCleanupPolicy {
    pub(crate) policy_id: String,
    pub(crate) remove_scratch_after_successful_sync: bool,
    pub(crate) retain_scratch_on_failure: bool,
    pub(crate) preserve_output_root: bool,
    pub(crate) preserve_log_root: bool,
    pub(crate) preserve_staged_inputs: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcScratchInputLink {
    pub(crate) artifact_id: Option<String>,
    pub(crate) artifact_role: Option<String>,
    pub(crate) source_kind: String,
    pub(crate) source_path: String,
    pub(crate) staged_path: String,
    pub(crate) link_path: String,
    pub(crate) checksum_sha256: String,
    pub(crate) size_bytes: u64,
    pub(crate) member_count: usize,
}

#[derive(Debug, Clone, Deserialize)]
struct LoadedLocalHpcScratchLayout {
    schema_version: String,
    output_path: String,
    staging_root: String,
    scratch_root: String,
    selected_job_count: usize,
    benchmark_job_count: usize,
    essential_pipeline_job_count: usize,
    input_link_count: usize,
    jobs: Vec<LoadedLocalHpcScratchLayoutJob>,
}

#[derive(Debug, Clone, Deserialize)]
struct LoadedLocalHpcScratchLayoutJob {
    job_id_local: String,
    job_kind: String,
    result_id: Option<String>,
    pipeline_id: Option<String>,
    node_id: Option<String>,
    domain: String,
    stage_id: String,
    tool_id: String,
    corpus_id: String,
    asset_profile_id: String,
    scratch_root: String,
    input_links_root: String,
    output_root: String,
    log_root: String,
    stdout_path: String,
    stderr_path: String,
    resources: LoadedLocalHpcScratchJobResources,
    cleanup_policy: LoadedLocalHpcScratchCleanupPolicy,
    input_links: Vec<LoadedLocalHpcScratchInputLink>,
}

#[derive(Debug, Clone, Deserialize)]
struct LoadedLocalHpcScratchJobResources {
    cpus_per_task: u32,
    memory_mb: u32,
    time_limit: String,
    scratch_gb: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct LoadedLocalHpcScratchCleanupPolicy {
    policy_id: String,
    remove_scratch_after_successful_sync: bool,
    retain_scratch_on_failure: bool,
    preserve_output_root: bool,
    preserve_log_root: bool,
    preserve_staged_inputs: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct LoadedLocalHpcScratchInputLink {
    artifact_id: Option<String>,
    artifact_role: Option<String>,
    source_kind: String,
    source_path: String,
    staged_path: String,
    link_path: String,
    checksum_sha256: String,
    size_bytes: u64,
    member_count: usize,
}

#[derive(Debug, Clone, Deserialize)]
struct ScratchLayoutAllDomainCommandStep {
    argv: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ScratchLayoutAllDomainCommandRow {
    result_id: String,
    domain: String,
    stage_id: String,
    tool_id: String,
    corpus_id: String,
    asset_profile_id: String,
    command_steps: Vec<ScratchLayoutAllDomainCommandStep>,
}

#[derive(Debug, Clone, Deserialize)]
struct ScratchLayoutEssentialPipelineCommandStep {
    argv: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ScratchLayoutEssentialPipelineCommandRow {
    pipeline_id: String,
    node_id: String,
    stage_id: String,
    tool_id: String,
    domain: String,
    render_status: String,
    command_steps: Vec<ScratchLayoutEssentialPipelineCommandStep>,
}

pub(crate) fn run_render_hpc_scratch_layout(
    args: &parse::BenchLocalRenderHpcScratchLayoutArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let output_path = args.output.clone().unwrap_or_else(|| {
        benchmark_paths.benchmark_hpc_dry_run_root().join("scratch-layout.json")
    });
    let manifest = render_hpc_scratch_layout(&repo_root, output_path)?;
    if args.json {
        render::json::print_pretty(&manifest)?;
    } else {
        println!("{}", manifest.output_path);
    }
    Ok(())
}

pub(crate) fn run_validate_hpc_scratch_layout(
    args: &parse::BenchLocalValidateHpcScratchLayoutArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let manifest_path = args.manifest.clone().unwrap_or_else(|| {
        benchmark_paths.benchmark_hpc_dry_run_root().join("scratch-layout.json")
    });
    let manifest = validate_hpc_scratch_layout_path(&repo_root, &manifest_path)?;
    if args.json {
        render::json::print_pretty(&manifest)?;
    } else {
        println!("{}", manifest.output_path);
    }
    Ok(())
}

pub(crate) fn render_hpc_scratch_layout(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<LocalHpcScratchLayout> {
    let absolute_output =
        if output_path.is_absolute() { output_path } else { repo_root.join(&output_path) };
    let manifest = build_hpc_scratch_layout(repo_root, &absolute_output)?;
    bijux_dna_infra::atomic_write_json(&absolute_output, &manifest)?;
    Ok(manifest)
}

pub(crate) fn collect_hpc_scratch_layout(repo_root: &Path) -> Result<LocalHpcScratchLayout> {
    let benchmark_paths = BenchmarkPathResolver::new(repo_root, None);
    let output_path = benchmark_paths.benchmark_hpc_dry_run_root().join("scratch-layout.json");
    build_hpc_scratch_layout(repo_root, &output_path)
}

pub(crate) fn load_validated_hpc_scratch_layout_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<LocalHpcScratchLayout> {
    let absolute_manifest_path = if manifest_path.is_absolute() {
        manifest_path.to_path_buf()
    } else {
        repo_root.join(manifest_path)
    };
    let raw = fs::read_to_string(&absolute_manifest_path)
        .with_context(|| format!("read {}", absolute_manifest_path.display()))?;
    let loaded = serde_json::from_str::<LoadedLocalHpcScratchLayout>(&raw)
        .with_context(|| format!("parse {}", absolute_manifest_path.display()))?;
    if loaded.schema_version != LOCAL_HPC_SCRATCH_LAYOUT_SCHEMA_VERSION {
        return Err(anyhow!(
            "HPC scratch layout `{}` uses schema `{}` instead of `{}`",
            absolute_manifest_path.display(),
            loaded.schema_version,
            LOCAL_HPC_SCRATCH_LAYOUT_SCHEMA_VERSION
        ));
    }
    let manifest = LocalHpcScratchLayout {
        schema_version: LOCAL_HPC_SCRATCH_LAYOUT_SCHEMA_VERSION,
        output_path: loaded.output_path,
        staging_root: loaded.staging_root,
        scratch_root: loaded.scratch_root,
        selected_job_count: loaded.selected_job_count,
        benchmark_job_count: loaded.benchmark_job_count,
        essential_pipeline_job_count: loaded.essential_pipeline_job_count,
        input_link_count: loaded.input_link_count,
        jobs: loaded
            .jobs
            .into_iter()
            .map(|job| LocalHpcScratchLayoutJob {
                job_id_local: job.job_id_local,
                job_kind: job.job_kind,
                result_id: job.result_id,
                pipeline_id: job.pipeline_id,
                node_id: job.node_id,
                domain: job.domain,
                stage_id: job.stage_id,
                tool_id: job.tool_id,
                corpus_id: job.corpus_id,
                asset_profile_id: job.asset_profile_id,
                scratch_root: job.scratch_root,
                input_links_root: job.input_links_root,
                output_root: job.output_root,
                log_root: job.log_root,
                stdout_path: job.stdout_path,
                stderr_path: job.stderr_path,
                resources: LocalHpcScratchJobResources {
                    cpus_per_task: job.resources.cpus_per_task,
                    memory_mb: job.resources.memory_mb,
                    time_limit: job.resources.time_limit,
                    scratch_gb: job.resources.scratch_gb,
                },
                cleanup_policy: LocalHpcScratchCleanupPolicy {
                    policy_id: job.cleanup_policy.policy_id,
                    remove_scratch_after_successful_sync: job
                        .cleanup_policy
                        .remove_scratch_after_successful_sync,
                    retain_scratch_on_failure: job.cleanup_policy.retain_scratch_on_failure,
                    preserve_output_root: job.cleanup_policy.preserve_output_root,
                    preserve_log_root: job.cleanup_policy.preserve_log_root,
                    preserve_staged_inputs: job.cleanup_policy.preserve_staged_inputs,
                },
                input_links: job
                    .input_links
                    .into_iter()
                    .map(|input| LocalHpcScratchInputLink {
                        artifact_id: input.artifact_id,
                        artifact_role: input.artifact_role,
                        source_kind: input.source_kind,
                        source_path: input.source_path,
                        staged_path: input.staged_path,
                        link_path: input.link_path,
                        checksum_sha256: input.checksum_sha256,
                        size_bytes: input.size_bytes,
                        member_count: input.member_count,
                    })
                    .collect(),
            })
            .collect(),
    };
    ensure_scratch_layout_contract(&manifest)?;
    let expected_output_path = path_relative_to_repo(repo_root, &absolute_manifest_path);
    if manifest.output_path != expected_output_path {
        return Err(anyhow!(
            "HPC scratch layout `{}` declares output_path `{}` but is stored at `{}`",
            absolute_manifest_path.display(),
            manifest.output_path,
            expected_output_path
        ));
    }
    Ok(manifest)
}

pub(crate) fn validate_hpc_scratch_layout_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<LocalHpcScratchLayout> {
    let absolute_manifest_path = if manifest_path.is_absolute() {
        manifest_path.to_path_buf()
    } else {
        repo_root.join(manifest_path)
    };
    let observed = load_validated_hpc_scratch_layout_path(repo_root, &absolute_manifest_path)?;
    let expected = build_hpc_scratch_layout(repo_root, &absolute_manifest_path)?;
    ensure_scratch_layout_matches_expected(&absolute_manifest_path, &observed, &expected)?;
    Ok(observed)
}

fn build_hpc_scratch_layout(
    repo_root: &Path,
    absolute_output: &Path,
) -> Result<LocalHpcScratchLayout> {
    ensure_path_stays_within_benchmark_runs_root(
        repo_root,
        absolute_output,
        "HPC scratch layout output",
    )?;
    if let Some(parent) = absolute_output.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let benchmark_paths = BenchmarkPathResolver::new(repo_root, None);
    let staging_root = benchmark_paths.benchmark_hpc_dry_run_root().join("staged");
    let scratch_root = benchmark_paths.benchmark_hpc_dry_run_root().join("scratch");
    let submit_jobs = load_local_hpc_selected_jobs(repo_root)?;

    let benchmark_rows = load_all_domain_rendered_command_argv_rows(repo_root)?
        .into_iter()
        .map(|row| (row.result_id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let essential_rows = load_essential_pipeline_rendered_command_argv_rows(repo_root)?
        .into_iter()
        .map(|row| ((row.pipeline_id.clone(), row.node_id.clone()), row))
        .collect::<BTreeMap<_, _>>();
    let stage_ids = collect_stage_ids(&benchmark_rows, &essential_rows, &submit_jobs);
    let stage_input_hints = collect_local_hpc_stage_input_hints(repo_root, &stage_ids)?;
    let resource_hints = load_local_hpc_job_resource_hints(repo_root)?;

    let mut jobs = Vec::with_capacity(submit_jobs.len());
    let mut benchmark_job_count = 0usize;
    let mut essential_pipeline_job_count = 0usize;
    let mut input_link_count = 0usize;

    for job in &submit_jobs {
        let resources = resolve_local_hpc_job_resource_hint(
            &resource_hints,
            &job.domain,
            &job.stage_id,
            &job.tool_id,
        );
        let scratch_job =
            match (job.result_id.as_ref(), job.pipeline_id.as_ref(), job.node_id.as_ref()) {
                (Some(_), None, None) => {
                    benchmark_job_count += 1;
                    build_benchmark_scratch_job(
                        repo_root,
                        &staging_root,
                        &scratch_root,
                        job,
                        benchmark_rows
                            .get(job.result_id.as_deref().ok_or_else(|| {
                                anyhow!(
                                    "benchmark scratch layout job `{}` is missing result_id",
                                    job.job_id_local
                                )
                            })?)
                            .ok_or_else(|| {
                                anyhow!(
                                "benchmark scratch layout is missing rendered command row for `{}`",
                                job.job_id_local
                            )
                            })?,
                        &stage_input_hints,
                        &resources,
                    )?
                }
                (None, Some(_), Some(_)) => {
                    essential_pipeline_job_count += 1;
                    build_essential_pipeline_scratch_job(
                        repo_root,
                        &staging_root,
                        &scratch_root,
                        job,
                        essential_rows
                            .get(&(
                                job.pipeline_id.clone().ok_or_else(|| {
                                    anyhow!(
                                        "pipeline scratch layout job `{}` is missing pipeline_id",
                                        job.job_id_local
                                    )
                                })?,
                                job.node_id.clone().ok_or_else(|| {
                                    anyhow!(
                                        "pipeline scratch layout job `{}` is missing node_id",
                                        job.job_id_local
                                    )
                                })?,
                            ))
                            .ok_or_else(|| {
                                anyhow!(
                                "pipeline scratch layout is missing rendered command row for `{}`",
                                job.job_id_local
                            )
                            })?,
                        &stage_input_hints,
                        &resources,
                    )?
                }
                _ => {
                    return Err(anyhow!(
                        "HPC scratch layout could not classify submit-manifest job `{}`",
                        job.job_id_local
                    ))
                }
            };
        input_link_count += scratch_job.input_links.len();
        jobs.push(scratch_job);
    }

    let manifest = LocalHpcScratchLayout {
        schema_version: LOCAL_HPC_SCRATCH_LAYOUT_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, absolute_output),
        staging_root: path_relative_to_repo(repo_root, &staging_root),
        scratch_root: path_relative_to_repo(repo_root, &scratch_root),
        selected_job_count: jobs.len(),
        benchmark_job_count,
        essential_pipeline_job_count,
        input_link_count,
        jobs,
    };
    ensure_scratch_layout_contract(&manifest)?;
    Ok(manifest)
}

fn load_all_domain_rendered_command_argv_rows(
    repo_root: &Path,
) -> Result<Vec<ScratchLayoutAllDomainCommandRow>> {
    let path = repo_root.join(DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_ARGV_PATH);
    if !path.is_file() {
        render_all_domain_commands(repo_root, DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_PATH.into())
            .with_context(|| format!("render {}", path.display()))?;
    }
    load_jsonl_rows(
        &path,
        "all-domain rendered command argv row",
    )
}

fn load_essential_pipeline_rendered_command_argv_rows(
    repo_root: &Path,
) -> Result<Vec<ScratchLayoutEssentialPipelineCommandRow>> {
    let path = repo_root.join(DEFAULT_ESSENTIAL_PIPELINE_RENDERED_COMMANDS_ARGV_PATH);
    if !path.is_file() {
        render_essential_pipeline_commands(
            repo_root,
            DEFAULT_ESSENTIAL_PIPELINE_RENDERED_COMMANDS_PATH.into(),
        )
        .with_context(|| format!("render {}", path.display()))?;
    }
    let rows = load_jsonl_rows(
        &path,
        "essential pipeline rendered command argv row",
    )?;
    if rows
        .iter()
        .any(|row: &ScratchLayoutEssentialPipelineCommandRow| row.render_status != "rendered")
    {
        return Err(anyhow!(
            "HPC scratch layout requires rendered essential pipeline command argv rows"
        ));
    }
    Ok(rows)
}

fn load_jsonl_rows<T>(path: &Path, row_label: &str) -> Result<Vec<T>>
where
    T: for<'de> Deserialize<'de>,
{
    let body = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    body.lines()
        .enumerate()
        .map(|(index, line)| {
            serde_json::from_str::<T>(line)
                .with_context(|| format!("parse {row_label} {} from {}", index + 1, path.display()))
        })
        .collect()
}

fn collect_stage_ids(
    benchmark_rows: &BTreeMap<String, ScratchLayoutAllDomainCommandRow>,
    essential_rows: &BTreeMap<(String, String), ScratchLayoutEssentialPipelineCommandRow>,
    jobs: &[BenchLocalAllDomainSlurmSubmitJob],
) -> BTreeSet<String> {
    let mut stage_ids = jobs.iter().map(|job| job.stage_id.clone()).collect::<BTreeSet<_>>();
    for row in benchmark_rows.values() {
        stage_ids
            .extend(row.command_steps.iter().filter_map(|step| materialize_stage_id(&step.argv)));
    }
    for row in essential_rows.values() {
        stage_ids
            .extend(row.command_steps.iter().filter_map(|step| materialize_stage_id(&step.argv)));
    }
    stage_ids
}

fn build_benchmark_scratch_job(
    repo_root: &Path,
    staging_root: &Path,
    scratch_root: &Path,
    job: &BenchLocalAllDomainSlurmSubmitJob,
    row: &ScratchLayoutAllDomainCommandRow,
    stage_input_hints: &BTreeMap<String, Vec<LocalHpcStageInputHint>>,
    resources: &super::local_hpc_job_resources::LocalHpcJobResourceHint,
) -> Result<LocalHpcScratchLayoutJob> {
    let result_id = row.result_id.clone();
    let job_scratch_root = scratch_root
        .join("benchmark-results")
        .join(&row.domain)
        .join(&row.corpus_id)
        .join(&row.stage_id)
        .join(&row.asset_profile_id)
        .join(&row.tool_id)
        .join(&row.result_id);
    let input_links_root = job_scratch_root.join("inputs");
    let input_links = collect_local_hpc_job_source_inputs(
        repo_root,
        &row.result_id,
        &row.stage_id,
        row.command_steps.iter().map(|step| step.argv.as_slice()),
        stage_input_hints,
    )?
    .into_iter()
    .map(|input| to_input_link(repo_root, staging_root, &input_links_root, input))
    .collect::<Result<Vec<_>>>()?;
    build_scratch_job(
        repo_root,
        job,
        Some(result_id),
        None,
        None,
        job_scratch_root,
        input_links_root,
        input_links,
        resources,
    )
}

fn build_essential_pipeline_scratch_job(
    repo_root: &Path,
    staging_root: &Path,
    scratch_root: &Path,
    job: &BenchLocalAllDomainSlurmSubmitJob,
    row: &ScratchLayoutEssentialPipelineCommandRow,
    stage_input_hints: &BTreeMap<String, Vec<LocalHpcStageInputHint>>,
    resources: &super::local_hpc_job_resources::LocalHpcJobResourceHint,
) -> Result<LocalHpcScratchLayoutJob> {
    let pipeline_id = row.pipeline_id.clone();
    let node_id = row.node_id.clone();
    let job_scratch_root =
        scratch_root.join("essential-pipelines").join(&row.pipeline_id).join(&row.node_id);
    let input_links_root = job_scratch_root.join("inputs");
    let input_links = collect_local_hpc_job_source_inputs(
        repo_root,
        &format!("{}::{}", row.pipeline_id, row.node_id),
        &row.stage_id,
        row.command_steps.iter().map(|step| step.argv.as_slice()),
        stage_input_hints,
    )?
    .into_iter()
    .map(|input| to_input_link(repo_root, staging_root, &input_links_root, input))
    .collect::<Result<Vec<_>>>()?;
    build_scratch_job(
        repo_root,
        job,
        None,
        Some(pipeline_id),
        Some(node_id),
        job_scratch_root,
        input_links_root,
        input_links,
        resources,
    )
}

#[allow(clippy::too_many_arguments)]
fn build_scratch_job(
    repo_root: &Path,
    job: &BenchLocalAllDomainSlurmSubmitJob,
    result_id: Option<String>,
    pipeline_id: Option<String>,
    node_id: Option<String>,
    job_scratch_root: PathBuf,
    input_links_root: PathBuf,
    input_links: Vec<LocalHpcScratchInputLink>,
    resources: &super::local_hpc_job_resources::LocalHpcJobResourceHint,
) -> Result<LocalHpcScratchLayoutJob> {
    Ok(LocalHpcScratchLayoutJob {
        job_id_local: job.job_id_local.clone(),
        job_kind: if result_id.is_some() {
            "benchmark_result".to_string()
        } else {
            "essential_pipeline_node".to_string()
        },
        result_id,
        pipeline_id,
        node_id,
        domain: job.domain.clone(),
        stage_id: job.stage_id.clone(),
        tool_id: job.tool_id.clone(),
        corpus_id: job.corpus_id.clone(),
        asset_profile_id: job.asset_profile_id.clone(),
        scratch_root: path_relative_to_repo(repo_root, &job_scratch_root),
        input_links_root: path_relative_to_repo(repo_root, &input_links_root),
        output_root: output_root_for_job(&job.outputs)?,
        log_root: log_root_for_job(&job.stdout, &job.stderr)?,
        stdout_path: job.stdout.clone(),
        stderr_path: job.stderr.clone(),
        resources: LocalHpcScratchJobResources {
            cpus_per_task: resources.cpus_per_task,
            memory_mb: resources.memory_mb,
            time_limit: resources.time_limit.clone(),
            scratch_gb: resources.scratch_gb,
        },
        cleanup_policy: default_cleanup_policy(),
        input_links,
    })
}

fn to_input_link(
    repo_root: &Path,
    staging_root: &Path,
    input_links_root: &Path,
    input: LocalHpcDiscoveredSourceInput,
) -> Result<LocalHpcScratchInputLink> {
    let staged_path =
        path_relative_to_repo(repo_root, &staging_root.join(&input.source_asset.source_path));
    let link_path =
        path_relative_to_repo(repo_root, &input_links_root.join(&input.source_asset.source_path));
    Ok(LocalHpcScratchInputLink {
        artifact_id: input.artifact_id,
        artifact_role: input.artifact_role,
        source_kind: input.source_asset.source_kind,
        source_path: input.source_asset.source_path,
        staged_path,
        link_path,
        checksum_sha256: input.source_asset.checksum_sha256,
        size_bytes: input.source_asset.size_bytes,
        member_count: input.source_asset.member_count,
    })
}

fn output_root_for_job(outputs: &[String]) -> Result<String> {
    if outputs.is_empty() {
        return Err(anyhow!("HPC scratch layout job must keep at least one output path"));
    }
    if outputs.len() == 1 {
        return Path::new(&outputs[0]).parent().map(path_to_string).ok_or_else(|| {
            anyhow!("HPC scratch layout output path `{}` has no parent", outputs[0])
        });
    }
    common_path_prefix(outputs)
}

fn log_root_for_job(stdout: &str, stderr: &str) -> Result<String> {
    let stdout_parent = Path::new(stdout)
        .parent()
        .ok_or_else(|| anyhow!("stdout path `{stdout}` has no parent"))?;
    let stderr_parent = Path::new(stderr)
        .parent()
        .ok_or_else(|| anyhow!("stderr path `{stderr}` has no parent"))?;
    if stdout_parent != stderr_parent {
        return Err(anyhow!(
            "stdout and stderr roots diverged for scratch layout: `{stdout}` vs `{stderr}`"
        ));
    }
    Ok(path_to_string(stdout_parent))
}

fn common_path_prefix(paths: &[String]) -> Result<String> {
    let mut prefix = PathBuf::from(&paths[0]);
    while !paths.iter().all(|path| Path::new(path).starts_with(&prefix)) {
        if !prefix.pop() {
            return Err(anyhow!(
                "scratch layout could not derive a shared output root from `{}`",
                paths.join("`, `")
            ));
        }
    }
    Ok(path_to_string(&prefix))
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn default_cleanup_policy() -> LocalHpcScratchCleanupPolicy {
    LocalHpcScratchCleanupPolicy {
        policy_id: CLEANUP_POLICY_ID.to_string(),
        remove_scratch_after_successful_sync: true,
        retain_scratch_on_failure: true,
        preserve_output_root: true,
        preserve_log_root: true,
        preserve_staged_inputs: true,
    }
}

fn ensure_scratch_layout_contract(manifest: &LocalHpcScratchLayout) -> Result<()> {
    if manifest.selected_job_count != manifest.jobs.len() {
        return Err(anyhow!("HPC scratch layout must keep selected_job_count aligned with jobs"));
    }
    if manifest.input_link_count
        != manifest.jobs.iter().map(|job| job.input_links.len()).sum::<usize>()
    {
        return Err(anyhow!(
            "HPC scratch layout must keep input_link_count aligned with input links"
        ));
    }
    if manifest.benchmark_job_count + manifest.essential_pipeline_job_count != manifest.jobs.len() {
        return Err(anyhow!(
            "HPC scratch layout must keep benchmark and essential pipeline counts aligned with jobs"
        ));
    }

    for job in &manifest.jobs {
        if job.input_links.is_empty() {
            return Err(anyhow!(
                "HPC scratch layout job `{}` must keep at least one input link",
                job.job_id_local
            ));
        }
        if job.scratch_root == manifest.scratch_root {
            return Err(anyhow!(
                "HPC scratch layout job `{}` must keep a job-scoped scratch root",
                job.job_id_local
            ));
        }
        if !job.input_links_root.starts_with(&job.scratch_root) {
            return Err(anyhow!(
                "HPC scratch layout job `{}` input_links_root must stay inside scratch_root",
                job.job_id_local
            ));
        }
        if job.log_root.is_empty() || job.output_root.is_empty() {
            return Err(anyhow!(
                "HPC scratch layout job `{}` must keep explicit log and output roots",
                job.job_id_local
            ));
        }
        for input in &job.input_links {
            if !input.link_path.starts_with(&job.input_links_root) {
                return Err(anyhow!(
                    "HPC scratch layout job `{}` input link `{}` must stay inside input_links_root",
                    job.job_id_local,
                    input.source_path
                ));
            }
            if input.source_path == input.link_path {
                return Err(anyhow!(
                    "HPC scratch layout job `{}` input link `{}` must not point at the original source path",
                    job.job_id_local,
                    input.source_path
                ));
            }
            if input.source_path == input.staged_path {
                return Err(anyhow!(
                    "HPC scratch layout job `{}` input `{}` must keep a staged path distinct from the source path",
                    job.job_id_local,
                    input.source_path
                ));
            }
        }
    }

    Ok(())
}

fn ensure_scratch_layout_matches_expected(
    manifest_path: &Path,
    observed: &LocalHpcScratchLayout,
    expected: &LocalHpcScratchLayout,
) -> Result<()> {
    if observed != expected {
        return Err(anyhow!(
            "HPC scratch layout `{}` drifted from the governed render",
            manifest_path.display()
        ));
    }
    Ok(())
}

#[cfg(all(test, feature = "bam_downstream"))]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{
        load_validated_hpc_scratch_layout_path, render_hpc_scratch_layout,
        validate_hpc_scratch_layout_path, LOCAL_HPC_SCRATCH_LAYOUT_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    fn manifest_path(repo_root: &Path, label: &str) -> (tempfile::TempDir, PathBuf) {
        let temp_dir = tempfile::Builder::new()
            .prefix(label)
            .tempdir_in(repo_root.join("runs/bench/hpc-dry-run"))
            .expect("temporary HPC dry-run directory");
        let manifest_path = temp_dir.path().join("scratch-layout.json");
        (temp_dir, manifest_path)
    }

    #[test]
    fn rendered_hpc_scratch_layout_covers_all_generated_jobs() {
        let root = repo_root();
        let (_temp_dir, manifest_path) = manifest_path(&root, "rendered-hpc-scratch-layout-");
        let manifest =
            render_hpc_scratch_layout(&root, manifest_path).expect("render HPC scratch layout");

        assert_eq!(manifest.schema_version, LOCAL_HPC_SCRATCH_LAYOUT_SCHEMA_VERSION);
        assert!(manifest.output_path.ends_with("/scratch-layout.json"));
        assert!(manifest.selected_job_count > 0);
        assert_eq!(
            manifest.selected_job_count,
            manifest.benchmark_job_count + manifest.essential_pipeline_job_count
        );
        assert!(manifest.input_link_count >= manifest.selected_job_count);
        assert!(manifest.jobs.iter().all(|job| !job.input_links.is_empty()));

        let benchmark_job = manifest
            .jobs
            .iter()
            .find(|job| {
                job.result_id.as_deref() == Some("bam:corpus-01-mini:bam.align:sample-set:bowtie2")
            })
            .expect("governed BAM benchmark job");
        assert!(benchmark_job
            .scratch_root
            .starts_with("runs/bench/hpc-dry-run/scratch/benchmark-results/"));
        assert_eq!(
            benchmark_job.cleanup_policy.policy_id,
            "copy_back_outputs_remove_successful_scratch"
        );
        assert!(benchmark_job.input_links.iter().any(|input| {
            input.source_path == "assets/reference/host/references/toy_host_reference"
                && input.staged_path
                    == "runs/bench/hpc-dry-run/staged/assets/reference/host/references/toy_host_reference"
                && input.link_path.starts_with(&benchmark_job.input_links_root)
        }));

        let pipeline_job = manifest
            .jobs
            .iter()
            .find(|job| job.pipeline_id.is_some() && job.node_id.is_some())
            .expect("essential pipeline job");
        assert!(pipeline_job
            .scratch_root
            .starts_with("runs/bench/hpc-dry-run/scratch/essential-pipelines/"));
        assert!(!pipeline_job.output_root.is_empty());
        assert!(!pipeline_job.log_root.is_empty());
        assert!(pipeline_job.resources.scratch_gb > 0);
    }

    #[test]
    fn loaded_hpc_scratch_layout_matches_governed_render() {
        let root = repo_root();
        let (_temp_dir, manifest_path) = manifest_path(&root, "loaded-hpc-scratch-layout-");
        let rendered = render_hpc_scratch_layout(&root, manifest_path.clone())
            .expect("render HPC scratch layout");
        let loaded = load_validated_hpc_scratch_layout_path(&root, &manifest_path)
            .expect("load validated HPC scratch layout");

        assert_eq!(loaded, rendered);
    }

    #[test]
    fn validated_hpc_scratch_layout_rejects_stale_input_link_count() {
        let root = repo_root();
        let (_temp_dir, manifest_path) = manifest_path(&root, "validated-hpc-scratch-layout-");
        let rendered = render_hpc_scratch_layout(&root, manifest_path.clone())
            .expect("render HPC scratch layout");
        let stale_body =
            std::fs::read_to_string(&manifest_path).expect("read manifest body").replacen(
                &format!("\"input_link_count\": {}", rendered.input_link_count),
                &format!("\"input_link_count\": {}", rendered.input_link_count.saturating_sub(1)),
                1,
            );
        std::fs::write(&manifest_path, stale_body).expect("write stale manifest body");

        let error = validate_hpc_scratch_layout_path(&root, &manifest_path)
            .expect_err("stale scratch layout must fail validation");
        assert!(
            error.to_string().contains("input_link_count aligned with input links"),
            "stale scratch layout error must report the input-link invariant: {error:#}"
        );
    }
}
