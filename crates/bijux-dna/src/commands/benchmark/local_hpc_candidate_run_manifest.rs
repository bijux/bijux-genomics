use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::local_all_domain_slurm_submit_manifest::BenchLocalAllDomainSlurmSubmitJob;
use super::local_hpc_asset_staging_manifest::{
    collect_hpc_asset_staging_manifest, LocalHpcAssetStagingInput, LocalHpcAssetStagingManifest,
};
use super::local_hpc_execution_resolver::{
    collect_hpc_execution_resolver_report, LocalHpcExecutionResolverReport,
    LocalHpcExecutionResolverRow,
};
use super::local_hpc_job_completion::resolve_local_hpc_job_result_paths;
use super::local_hpc_job_resources::{load_local_hpc_job_resource_hints, LocalHpcJobResourceHint};
use super::local_hpc_selected_jobs::load_local_hpc_selected_jobs;
use super::path_resolution::{
    ensure_path_stays_within_benchmark_readiness_hpc_root, BenchmarkPathResolver,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

const LOCAL_HPC_CANDIDATE_RUN_MANIFEST_SCHEMA_VERSION: &str =
    "bijux.bench.local_hpc_candidate_run_manifest.v1";
pub(crate) const DEFAULT_HPC_CANDIDATE_RUN_MANIFEST_PATH: &str =
    "benchmarks/readiness/hpc/FIRST_HPC_CANDIDATE_RUN.json";
const SMALL_CANDIDATE_PROFILE_ID: &str = "small_runtime_surface_representatives";
const SMALL_INPUT_FOOTPRINT_MAX_BYTES: u64 = 10 * 1024 * 1024;
const SMALL_CPUS_PER_TASK_MAX: u32 = 4;
const SMALL_MEMORY_MB_MAX: u32 = 4096;
const SMALL_WALLTIME_MINUTES_MAX: u32 = 15;
const SMALL_SCRATCH_GB_MAX: u32 = 2;

const EXCLUSION_NON_BENCHMARK_RESULT: &str = "non_benchmark_result";
const EXCLUSION_HAS_DEPENDENCIES: &str = "has_dependencies";
const EXCLUSION_MISSING_EXPECTED_OUTPUTS: &str = "missing_expected_outputs";
const EXCLUSION_MISSING_KNOWN_ASSETS: &str = "missing_known_assets";
const EXCLUSION_MISSING_RESOURCE_HINT: &str = "missing_resource_hint";
const EXCLUSION_UNKNOWN_EXECUTION_MODE: &str = "unknown_execution_mode";
const EXCLUSION_UNAVAILABLE_EXECUTION_RESOLUTION: &str = "unavailable_execution_resolution";
const EXCLUSION_RESOURCE_ENVELOPE_EXCEEDED: &str = "resource_envelope_exceeded";
const EXCLUSION_INPUT_FOOTPRINT_EXCEEDED: &str = "input_footprint_exceeded";
const EXCLUSION_SUPERSEDED_REPRESENTATIVE: &str = "superseded_by_smaller_representative";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcCandidateResourceCeiling {
    pub(crate) cpus_per_task_max: u32,
    pub(crate) memory_mb_max: u32,
    pub(crate) walltime_minutes_max: u32,
    pub(crate) scratch_gb_max: u32,
    pub(crate) staged_input_bytes_max: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcCandidateBehavior {
    pub(crate) only_benchmark_result_jobs: bool,
    pub(crate) only_dependency_free_jobs: bool,
    pub(crate) only_known_assets: bool,
    pub(crate) only_known_execution_modes: bool,
    pub(crate) only_available_execution_resolution: bool,
    pub(crate) only_small_resource_envelopes: bool,
    pub(crate) only_small_input_footprints: bool,
    pub(crate) selected_rows_keep_expected_outputs: bool,
    pub(crate) selected_rows_keep_stop_conditions: bool,
    pub(crate) vcf_rows_excluded_for_unknown_execution_modes: bool,
    pub(crate) proven: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcCandidateStopCondition {
    pub(crate) condition_id: String,
    pub(crate) source_surface: String,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcCandidateResourceHint {
    pub(crate) cpus_per_task: u32,
    pub(crate) memory_mb: u32,
    pub(crate) time_limit: String,
    pub(crate) walltime_minutes: u32,
    pub(crate) scratch_gb: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcCandidateJob {
    pub(crate) representative_group_id: String,
    pub(crate) selection_reason: String,
    pub(crate) job_id_local: String,
    pub(crate) result_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) execution_mode: String,
    pub(crate) execution_mode_summary: String,
    pub(crate) resolution_kind: String,
    pub(crate) resolution_target: String,
    pub(crate) command_entrypoint: Option<String>,
    pub(crate) expected_stage_result_manifest_path: String,
    pub(crate) expected_outputs: Vec<String>,
    pub(crate) resource_hint: LocalHpcCandidateResourceHint,
    pub(crate) staged_input_count: usize,
    pub(crate) staged_input_total_bytes: u64,
    pub(crate) staged_inputs: Vec<LocalHpcAssetStagingInput>,
    pub(crate) stop_conditions: Vec<LocalHpcCandidateStopCondition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcCandidateRunManifest {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) selection_profile_id: String,
    pub(crate) evaluated_job_count: usize,
    pub(crate) benchmark_job_count: usize,
    pub(crate) root_benchmark_job_count: usize,
    pub(crate) eligible_small_job_count: usize,
    pub(crate) representative_group_count: usize,
    pub(crate) selected_job_count: usize,
    pub(crate) selected_staged_input_count: usize,
    pub(crate) selected_staged_input_bytes: u64,
    pub(crate) selected_domain_counts: BTreeMap<String, usize>,
    pub(crate) selected_execution_mode_counts: BTreeMap<String, usize>,
    pub(crate) selected_resolution_kind_counts: BTreeMap<String, usize>,
    pub(crate) exclusion_reason_counts: BTreeMap<String, usize>,
    pub(crate) resource_ceiling: LocalHpcCandidateResourceCeiling,
    pub(crate) behavior: LocalHpcCandidateBehavior,
    pub(crate) rows: Vec<LocalHpcCandidateJob>,
}

#[derive(Debug, Clone)]
struct EligibleCandidateJob<'a> {
    representative_group_id: String,
    selection_reason: String,
    submit_job: &'a BenchLocalAllDomainSlurmSubmitJob,
    execution: &'a LocalHpcExecutionResolverRow,
    resource_hint: LocalHpcJobResourceHint,
    walltime_minutes: u32,
    stage_result_manifest_path: String,
    staged_inputs: Vec<LocalHpcAssetStagingInput>,
    staged_input_total_bytes: u64,
}

pub(crate) fn run_render_hpc_candidate_run_manifest(
    args: &parse::BenchLocalRenderHpcCandidateRunManifestArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let output_path = args.output.clone().unwrap_or_else(|| {
        benchmark_paths.benchmark_hpc_readiness_root().join("FIRST_HPC_CANDIDATE_RUN.json")
    });
    let report = render_hpc_candidate_run_manifest(&repo_root, output_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn run_validate_hpc_candidate_run_manifest(
    args: &parse::BenchLocalValidateHpcCandidateRunManifestArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let manifest_path = args.manifest.clone().unwrap_or_else(|| {
        benchmark_paths.benchmark_hpc_readiness_root().join("FIRST_HPC_CANDIDATE_RUN.json")
    });
    let report = validate_hpc_candidate_run_manifest_path(&repo_root, &manifest_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_hpc_candidate_run_manifest(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<LocalHpcCandidateRunManifest> {
    let absolute_output =
        if output_path.is_absolute() { output_path } else { repo_root.join(&output_path) };
    let report = build_hpc_candidate_run_manifest(repo_root, &absolute_output)?;
    if let Some(parent) = absolute_output.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(&absolute_output, &report)?;
    Ok(report)
}

pub(crate) fn validate_hpc_candidate_run_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<LocalHpcCandidateRunManifest> {
    let absolute_manifest_path = if manifest_path.is_absolute() {
        manifest_path.to_path_buf()
    } else {
        repo_root.join(manifest_path)
    };
    let report = build_hpc_candidate_run_manifest(repo_root, &absolute_manifest_path)?;
    let observed = fs::read(&absolute_manifest_path)
        .with_context(|| format!("read {}", absolute_manifest_path.display()))?;
    let expected =
        serde_json::to_vec_pretty(&report).context("serialize governed HPC candidate manifest")?;
    if observed != expected {
        return Err(anyhow!(
            "HPC candidate run manifest `{}` drifted from governed dry-run inputs; rerun `bijux-dna bench local render-hpc-candidate-run-manifest --output {}`",
            absolute_manifest_path.display(),
            report.output_path
        ));
    }
    Ok(report)
}

fn build_hpc_candidate_run_manifest(
    repo_root: &Path,
    absolute_output_path: &Path,
) -> Result<LocalHpcCandidateRunManifest> {
    ensure_path_stays_within_benchmark_readiness_hpc_root(
        repo_root,
        absolute_output_path,
        "HPC candidate run manifest output",
    )?;
    let asset_manifest = collect_hpc_asset_staging_manifest(repo_root)?;
    let execution_resolver = collect_hpc_execution_resolver_report(repo_root)?;
    let selected_jobs = load_local_hpc_selected_jobs(repo_root)?;
    let resource_hints = load_local_hpc_job_resource_hints(repo_root)?;

    let report = select_candidate_run_rows(
        repo_root,
        absolute_output_path,
        &selected_jobs,
        &asset_manifest,
        &execution_resolver,
        &resource_hints,
    )?;
    ensure_candidate_run_manifest_contract(&report)?;
    Ok(report)
}

fn select_candidate_run_rows(
    repo_root: &Path,
    absolute_output_path: &Path,
    selected_jobs: &[BenchLocalAllDomainSlurmSubmitJob],
    asset_manifest: &LocalHpcAssetStagingManifest,
    execution_resolver: &LocalHpcExecutionResolverReport,
    resource_hints: &BTreeMap<(String, String, String), LocalHpcJobResourceHint>,
) -> Result<LocalHpcCandidateRunManifest> {
    let assets_by_result_id = asset_manifest
        .jobs
        .iter()
        .map(|job| (job.result_id.clone(), job.staged_inputs.clone()))
        .collect::<BTreeMap<_, _>>();
    let execution_by_tool_id = execution_resolver
        .rows
        .iter()
        .map(|row| (row.tool_id.clone(), row))
        .collect::<BTreeMap<_, _>>();

    let mut exclusion_reason_counts = BTreeMap::<String, usize>::new();
    let mut eligible = Vec::<EligibleCandidateJob<'_>>::new();
    let mut benchmark_job_count = 0usize;
    let mut root_benchmark_job_count = 0usize;
    let mut vcf_unknown_execution_mode_count = 0usize;

    for submit_job in selected_jobs {
        let Some(result_id) = submit_job.result_id.as_ref() else {
            increment_count(&mut exclusion_reason_counts, EXCLUSION_NON_BENCHMARK_RESULT);
            continue;
        };
        benchmark_job_count += 1;
        if !submit_job.dependencies.is_empty() {
            increment_count(&mut exclusion_reason_counts, EXCLUSION_HAS_DEPENDENCIES);
            continue;
        }
        root_benchmark_job_count += 1;
        if submit_job.outputs.is_empty() {
            increment_count(&mut exclusion_reason_counts, EXCLUSION_MISSING_EXPECTED_OUTPUTS);
            continue;
        }

        let Some(staged_inputs) = assets_by_result_id.get(result_id) else {
            increment_count(&mut exclusion_reason_counts, EXCLUSION_MISSING_KNOWN_ASSETS);
            continue;
        };
        if staged_inputs.is_empty() {
            increment_count(&mut exclusion_reason_counts, EXCLUSION_MISSING_KNOWN_ASSETS);
            continue;
        }

        let Some(execution) = execution_by_tool_id.get(&submit_job.tool_id) else {
            increment_count(&mut exclusion_reason_counts, EXCLUSION_UNKNOWN_EXECUTION_MODE);
            continue;
        };
        if execution.execution_mode == "unclassified" {
            increment_count(&mut exclusion_reason_counts, EXCLUSION_UNKNOWN_EXECUTION_MODE);
            if submit_job.domain == "vcf" {
                vcf_unknown_execution_mode_count += 1;
            }
            continue;
        }
        if execution.resolution_kind.contains("unavailable") {
            increment_count(
                &mut exclusion_reason_counts,
                EXCLUSION_UNAVAILABLE_EXECUTION_RESOLUTION,
            );
            continue;
        }

        let Some(resource_hint) = resource_hints
            .get(&(
                submit_job.domain.clone(),
                submit_job.stage_id.clone(),
                submit_job.tool_id.clone(),
            ))
            .cloned()
        else {
            increment_count(&mut exclusion_reason_counts, EXCLUSION_MISSING_RESOURCE_HINT);
            continue;
        };
        let walltime_minutes = parse_time_limit_minutes(&resource_hint.time_limit)?;
        if resource_hint.cpus_per_task > SMALL_CPUS_PER_TASK_MAX
            || resource_hint.memory_mb > SMALL_MEMORY_MB_MAX
            || walltime_minutes > SMALL_WALLTIME_MINUTES_MAX
            || resource_hint.scratch_gb > SMALL_SCRATCH_GB_MAX
        {
            increment_count(&mut exclusion_reason_counts, EXCLUSION_RESOURCE_ENVELOPE_EXCEEDED);
            continue;
        }

        let staged_input_total_bytes =
            staged_inputs.iter().map(|entry| entry.size_bytes).sum::<u64>();
        if staged_input_total_bytes > SMALL_INPUT_FOOTPRINT_MAX_BYTES {
            increment_count(&mut exclusion_reason_counts, EXCLUSION_INPUT_FOOTPRINT_EXCEEDED);
            continue;
        }

        let result_paths = resolve_local_hpc_job_result_paths(repo_root, submit_job)?;
        let representative_group_id = format!("{}:{}", submit_job.domain, execution.execution_mode);
        let selection_reason = format!(
            "smallest eligible dependency-free benchmark row for {} / {}",
            submit_job.domain, execution.execution_mode
        );
        eligible.push(EligibleCandidateJob {
            representative_group_id,
            selection_reason,
            submit_job,
            execution,
            resource_hint,
            walltime_minutes,
            stage_result_manifest_path: path_relative_to_repo(
                repo_root,
                &result_paths.stage_result_manifest_path,
            ),
            staged_inputs: staged_inputs.clone(),
            staged_input_total_bytes,
        });
    }

    eligible.sort_by(|left, right| candidate_sort_key(left).cmp(&candidate_sort_key(right)));
    let eligible_small_job_count = eligible.len();
    let mut selected_rows = Vec::new();
    let mut selected_groups = BTreeSet::<String>::new();
    for row in eligible {
        if !selected_groups.insert(row.representative_group_id.clone()) {
            increment_count(&mut exclusion_reason_counts, EXCLUSION_SUPERSEDED_REPRESENTATIVE);
            continue;
        }
        selected_rows.push(to_candidate_job(repo_root, row));
    }
    selected_rows.sort_by(|left, right| {
        left.representative_group_id
            .cmp(&right.representative_group_id)
            .then_with(|| left.result_id.cmp(&right.result_id))
    });

    let mut selected_domain_counts = BTreeMap::<String, usize>::new();
    let mut selected_execution_mode_counts = BTreeMap::<String, usize>::new();
    let mut selected_resolution_kind_counts = BTreeMap::<String, usize>::new();
    let mut selected_staged_input_count = 0usize;
    let mut selected_staged_input_bytes = 0u64;
    for row in &selected_rows {
        *selected_domain_counts.entry(row.domain.clone()).or_default() += 1;
        *selected_execution_mode_counts.entry(row.execution_mode.clone()).or_default() += 1;
        *selected_resolution_kind_counts.entry(row.resolution_kind.clone()).or_default() += 1;
        selected_staged_input_count += row.staged_input_count;
        selected_staged_input_bytes += row.staged_input_total_bytes;
    }

    let behavior = LocalHpcCandidateBehavior {
        only_benchmark_result_jobs: selected_rows.iter().all(|row| !row.result_id.is_empty()),
        only_dependency_free_jobs: true,
        only_known_assets: selected_rows.iter().all(|row| !row.staged_inputs.is_empty()),
        only_known_execution_modes: selected_rows.iter().all(|row| {
            !row.execution_mode.trim().is_empty() && row.execution_mode != "unclassified"
        }),
        only_available_execution_resolution: selected_rows
            .iter()
            .all(|row| !row.resolution_kind.contains("unavailable")),
        only_small_resource_envelopes: selected_rows.iter().all(|row| {
            row.resource_hint.cpus_per_task <= SMALL_CPUS_PER_TASK_MAX
                && row.resource_hint.memory_mb <= SMALL_MEMORY_MB_MAX
                && row.resource_hint.walltime_minutes <= SMALL_WALLTIME_MINUTES_MAX
                && row.resource_hint.scratch_gb <= SMALL_SCRATCH_GB_MAX
        }),
        only_small_input_footprints: selected_rows
            .iter()
            .all(|row| row.staged_input_total_bytes <= SMALL_INPUT_FOOTPRINT_MAX_BYTES),
        selected_rows_keep_expected_outputs: selected_rows
            .iter()
            .all(|row| !row.expected_outputs.is_empty()),
        selected_rows_keep_stop_conditions: selected_rows
            .iter()
            .all(|row| !row.stop_conditions.is_empty()),
        vcf_rows_excluded_for_unknown_execution_modes: vcf_unknown_execution_mode_count > 0,
        proven: false,
    };
    let behavior = LocalHpcCandidateBehavior {
        proven: behavior.only_benchmark_result_jobs
            && behavior.only_dependency_free_jobs
            && behavior.only_known_assets
            && behavior.only_known_execution_modes
            && behavior.only_available_execution_resolution
            && behavior.only_small_resource_envelopes
            && behavior.only_small_input_footprints
            && behavior.selected_rows_keep_expected_outputs
            && behavior.selected_rows_keep_stop_conditions
            && behavior.vcf_rows_excluded_for_unknown_execution_modes,
        ..behavior
    };

    Ok(LocalHpcCandidateRunManifest {
        schema_version: LOCAL_HPC_CANDIDATE_RUN_MANIFEST_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, absolute_output_path),
        selection_profile_id: SMALL_CANDIDATE_PROFILE_ID.to_string(),
        evaluated_job_count: selected_jobs.len(),
        benchmark_job_count,
        root_benchmark_job_count,
        eligible_small_job_count,
        representative_group_count: selected_groups.len(),
        selected_job_count: selected_rows.len(),
        selected_staged_input_count,
        selected_staged_input_bytes,
        selected_domain_counts,
        selected_execution_mode_counts,
        selected_resolution_kind_counts,
        exclusion_reason_counts,
        resource_ceiling: LocalHpcCandidateResourceCeiling {
            cpus_per_task_max: SMALL_CPUS_PER_TASK_MAX,
            memory_mb_max: SMALL_MEMORY_MB_MAX,
            walltime_minutes_max: SMALL_WALLTIME_MINUTES_MAX,
            scratch_gb_max: SMALL_SCRATCH_GB_MAX,
            staged_input_bytes_max: SMALL_INPUT_FOOTPRINT_MAX_BYTES,
        },
        behavior,
        rows: selected_rows,
    })
}

fn candidate_sort_key(
    row: &EligibleCandidateJob<'_>,
) -> (String, u64, u32, u32, u32, String, String, String) {
    (
        row.representative_group_id.clone(),
        row.staged_input_total_bytes,
        row.resource_hint.cpus_per_task,
        row.resource_hint.memory_mb,
        row.walltime_minutes,
        row.submit_job.stage_id.clone(),
        row.submit_job.tool_id.clone(),
        row.submit_job.result_id.clone().expect("eligible candidate rows always have result ids"),
    )
}

fn to_candidate_job(repo_root: &Path, row: EligibleCandidateJob<'_>) -> LocalHpcCandidateJob {
    let submit_job = row.submit_job;
    LocalHpcCandidateJob {
        representative_group_id: row.representative_group_id,
        selection_reason: row.selection_reason,
        job_id_local: submit_job.job_id_local.clone(),
        result_id: submit_job
            .result_id
            .clone()
            .expect("eligible candidate rows always have result ids"),
        domain: submit_job.domain.clone(),
        stage_id: submit_job.stage_id.clone(),
        tool_id: submit_job.tool_id.clone(),
        corpus_id: submit_job.corpus_id.clone(),
        asset_profile_id: submit_job.asset_profile_id.clone(),
        execution_mode: row.execution.execution_mode.clone(),
        execution_mode_summary: row.execution.execution_mode_summary.clone(),
        resolution_kind: row.execution.resolution_kind.clone(),
        resolution_target: row.execution.resolution_target.clone(),
        command_entrypoint: row.execution.command_entrypoint.clone(),
        expected_stage_result_manifest_path: row.stage_result_manifest_path,
        expected_outputs: submit_job.outputs.clone(),
        resource_hint: LocalHpcCandidateResourceHint {
            cpus_per_task: row.resource_hint.cpus_per_task,
            memory_mb: row.resource_hint.memory_mb,
            time_limit: row.resource_hint.time_limit.clone(),
            walltime_minutes: row.walltime_minutes,
            scratch_gb: row.resource_hint.scratch_gb,
        },
        staged_input_count: row.staged_inputs.len(),
        staged_input_total_bytes: row.staged_input_total_bytes,
        staged_inputs: row.staged_inputs,
        stop_conditions: candidate_stop_conditions(repo_root, submit_job),
    }
}

fn candidate_stop_conditions(
    repo_root: &Path,
    submit_job: &BenchLocalAllDomainSlurmSubmitJob,
) -> Vec<LocalHpcCandidateStopCondition> {
    let result_paths = resolve_local_hpc_job_result_paths(repo_root, submit_job)
        .expect("candidate stop conditions require resolvable HPC result paths");
    let manifest_path = path_relative_to_repo(repo_root, &result_paths.stage_result_manifest_path);
    let result_root = path_relative_to_repo(repo_root, &result_paths.result_root);
    vec![
        LocalHpcCandidateStopCondition {
            condition_id: "missing_stage_result_manifest".to_string(),
            source_surface: "local_hpc_job_completion".to_string(),
            detail: format!(
                "stop the candidate run if `{manifest_path}` is missing after `{}` finishes under `{result_root}`",
                submit_job.job_id_local
            ),
        },
        LocalHpcCandidateStopCondition {
            condition_id: "failed_stage_result_manifest".to_string(),
            source_surface: "local_hpc_job_completion".to_string(),
            detail: format!(
                "stop the candidate run if `{manifest_path}` records runtime status `failed` for `{}`",
                submit_job.job_id_local
            ),
        },
        LocalHpcCandidateStopCondition {
            condition_id: "stale_partial_outputs".to_string(),
            source_surface: "local_hpc_job_completion".to_string(),
            detail: format!(
                "stop the candidate run if `{}` reports success but any declared output under `{result_root}` is missing or mismatched",
                submit_job.job_id_local
            ),
        },
        LocalHpcCandidateStopCondition {
            condition_id: "unexpected_collection_status".to_string(),
            source_surface: "local_hpc_result_collection_simulation".to_string(),
            detail: format!(
                "stop the candidate run if result collection would classify `{}` as anything other than `complete` once its declared outputs land",
                submit_job.job_id_local
            ),
        },
    ]
}

fn ensure_candidate_run_manifest_contract(report: &LocalHpcCandidateRunManifest) -> Result<()> {
    if report.selected_job_count != report.rows.len() {
        return Err(anyhow!(
            "HPC candidate run manifest must keep selected_job_count aligned with rows"
        ));
    }
    if report.selected_job_count == 0 {
        return Err(anyhow!(
            "HPC candidate run manifest must select at least one representative job"
        ));
    }
    if report.representative_group_count != report.rows.len() {
        return Err(anyhow!(
            "HPC candidate run manifest must keep one selected row per representative group"
        ));
    }
    let unique_groups =
        report.rows.iter().map(|row| row.representative_group_id.as_str()).collect::<BTreeSet<_>>();
    if unique_groups.len() != report.rows.len() {
        return Err(anyhow!("HPC candidate run manifest representative groups must stay unique"));
    }
    if report.selected_staged_input_count
        != report.rows.iter().map(|row| row.staged_input_count).sum::<usize>()
    {
        return Err(anyhow!(
            "HPC candidate run manifest must keep selected_staged_input_count aligned with rows"
        ));
    }
    if report.selected_staged_input_bytes
        != report.rows.iter().map(|row| row.staged_input_total_bytes).sum::<u64>()
    {
        return Err(anyhow!(
            "HPC candidate run manifest must keep selected_staged_input_bytes aligned with rows"
        ));
    }
    if !report.behavior.proven {
        return Err(anyhow!(
            "HPC candidate run manifest behavior must stay proven for the selected subset"
        ));
    }
    if !report.selected_domain_counts.contains_key("fastq")
        || !report.selected_domain_counts.contains_key("bam")
    {
        return Err(anyhow!(
            "HPC candidate run manifest must cover both fastq and bam runtime surfaces"
        ));
    }
    if report.rows.iter().any(|row| row.domain == "vcf") {
        return Err(anyhow!(
            "HPC candidate run manifest cannot admit VCF rows until execution modes are governed"
        ));
    }
    if report.rows.iter().any(|row| row.resource_hint.walltime_minutes > SMALL_WALLTIME_MINUTES_MAX)
    {
        return Err(anyhow!(
            "HPC candidate run manifest cannot admit rows outside the small walltime ceiling"
        ));
    }
    if report.rows.iter().any(|row| row.staged_input_total_bytes > SMALL_INPUT_FOOTPRINT_MAX_BYTES)
    {
        return Err(anyhow!(
            "HPC candidate run manifest cannot admit rows outside the small input-footprint ceiling"
        ));
    }
    Ok(())
}

fn parse_time_limit_minutes(time_limit: &str) -> Result<u32> {
    let mut parts = time_limit.split(':');
    let hours = parts
        .next()
        .ok_or_else(|| anyhow!("time limit `{time_limit}` is missing hours"))?
        .parse::<u32>()
        .with_context(|| format!("parse hours from `{time_limit}`"))?;
    let minutes = parts
        .next()
        .ok_or_else(|| anyhow!("time limit `{time_limit}` is missing minutes"))?
        .parse::<u32>()
        .with_context(|| format!("parse minutes from `{time_limit}`"))?;
    let seconds = parts
        .next()
        .ok_or_else(|| anyhow!("time limit `{time_limit}` is missing seconds"))?
        .parse::<u32>()
        .with_context(|| format!("parse seconds from `{time_limit}`"))?;
    if parts.next().is_some() {
        return Err(anyhow!("time limit `{time_limit}` has too many segments"));
    }
    if seconds != 0 {
        return Err(anyhow!(
            "time limit `{time_limit}` must use whole minutes for the governed candidate ceiling"
        ));
    }
    Ok(hours * 60 + minutes)
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

fn increment_count(counts: &mut BTreeMap<String, usize>, key: &str) {
    *counts.entry(key.to_string()).or_default() += 1;
}
