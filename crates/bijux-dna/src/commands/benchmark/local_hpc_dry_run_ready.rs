use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::local_hpc_asset_staging_manifest::{
    collect_hpc_asset_staging_manifest, LocalHpcAssetStagingManifest,
    DEFAULT_HPC_ASSET_STAGING_MANIFEST_PATH,
};
use super::local_hpc_candidate_run_manifest::{
    collect_hpc_candidate_run_manifest, LocalHpcCandidateRunManifest,
    DEFAULT_HPC_CANDIDATE_RUN_MANIFEST_PATH,
};
use super::local_hpc_dependency_simulation::{
    collect_hpc_dependency_simulation, LocalHpcDependencySimulationReport,
    DEFAULT_HPC_DEPENDENCY_SIMULATION_PATH,
};
use super::local_hpc_execution_resolver::{
    collect_hpc_execution_resolver_report, LocalHpcExecutionResolverReport,
    DEFAULT_HPC_EXECUTION_RESOLVER_PATH,
};
use super::local_hpc_input_discovery::path_relative_to_repo;
use super::local_hpc_pipeline_node_array::{
    collect_hpc_pipeline_node_array, LocalHpcPipelineNodeArrayReport,
    DEFAULT_HPC_PIPELINE_NODE_ARRAY_SCRIPT_PATH,
};
use super::local_hpc_result_collection_simulation::{
    collect_hpc_result_collection_simulation, LocalHpcResultCollectionSimulationReport,
    DEFAULT_HPC_RESULT_COLLECTION_SIMULATION_PATH,
};
use super::local_hpc_resume_simulation::{
    collect_hpc_resume_simulation, LocalHpcResumeSimulationReport,
    DEFAULT_HPC_RESUME_SIMULATION_PATH,
};
use super::local_hpc_scratch_layout::{
    collect_hpc_scratch_layout, LocalHpcScratchLayout, DEFAULT_HPC_SCRATCH_LAYOUT_PATH,
};
use super::local_hpc_stage_benchmark_array::{
    collect_hpc_stage_benchmark_array, LocalHpcStageBenchmarkArrayReport,
    DEFAULT_HPC_STAGE_BENCHMARK_ARRAY_SCRIPT_PATH,
};
use super::path_resolution::{
    ensure_path_stays_within_benchmark_readiness_hpc_root, BenchmarkPathResolver,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

const LOCAL_HPC_DRY_RUN_READY_SCHEMA_VERSION: &str = "bijux.bench.local_hpc_dry_run_ready.v1";
pub(crate) const DEFAULT_HPC_DRY_RUN_READY_PATH: &str =
    "benchmarks/readiness/hpc/HPC_DRY_RUN_LOCAL_READY.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcDryRunReadyGoalCheck {
    pub(crate) goal_id: u32,
    pub(crate) surface: String,
    pub(crate) output_path: Option<String>,
    pub(crate) ok: bool,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcDryRunReadySummary {
    pub(crate) benchmark_job_count: usize,
    pub(crate) essential_pipeline_job_count: usize,
    pub(crate) staged_input_count: usize,
    pub(crate) execution_resolver_row_count: usize,
    pub(crate) execution_unclassified_tool_count: usize,
    pub(crate) stage_benchmark_row_count: usize,
    pub(crate) pipeline_node_row_count: usize,
    pub(crate) dependency_case_count: usize,
    pub(crate) resume_rerun_job_count: usize,
    pub(crate) result_collection_row_count: usize,
    pub(crate) candidate_job_count: usize,
    pub(crate) candidate_representative_group_count: usize,
    pub(crate) candidate_selected_input_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcDryRunReadyBehavior {
    pub(crate) goals_481_to_489_validated: bool,
    pub(crate) benchmark_jobs_keep_known_assets: bool,
    pub(crate) benchmark_jobs_keep_scratch_layout: bool,
    pub(crate) benchmark_jobs_keep_explicit_execution_resolution: bool,
    pub(crate) pipeline_nodes_keep_dependency_manifests: bool,
    pub(crate) dependency_failures_block_descendants_only: bool,
    pub(crate) resume_rules_are_explicit: bool,
    pub(crate) result_collection_statuses_are_distinct: bool,
    pub(crate) first_run_candidate_is_small_and_governed: bool,
    pub(crate) no_manual_steps_or_ambiguity: bool,
    pub(crate) proven: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalHpcDryRunReadyReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) checked_goal_count: usize,
    pub(crate) passed_goal_count: usize,
    pub(crate) failed_goal_count: usize,
    pub(crate) failing_goal_ids: Vec<u32>,
    pub(crate) ready_for_first_hpc_run: bool,
    pub(crate) summary: LocalHpcDryRunReadySummary,
    pub(crate) behavior: LocalHpcDryRunReadyBehavior,
    pub(crate) checks: Vec<LocalHpcDryRunReadyGoalCheck>,
}

pub(crate) fn run_render_hpc_dry_run_ready(
    args: &parse::BenchLocalRenderHpcDryRunReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let output_path = args.output.clone().unwrap_or_else(|| {
        benchmark_paths.benchmark_hpc_readiness_root().join("HPC_DRY_RUN_LOCAL_READY.json")
    });
    let report = render_hpc_dry_run_ready(&repo_root, output_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn run_validate_hpc_dry_run_ready(
    args: &parse::BenchLocalValidateHpcDryRunReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let manifest_path = args.manifest.clone().unwrap_or_else(|| {
        benchmark_paths.benchmark_hpc_readiness_root().join("HPC_DRY_RUN_LOCAL_READY.json")
    });
    let report = validate_hpc_dry_run_ready_path(&repo_root, &manifest_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_hpc_dry_run_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<LocalHpcDryRunReadyReport> {
    let absolute_output =
        if output_path.is_absolute() { output_path } else { repo_root.join(&output_path) };
    let report = build_hpc_dry_run_ready(repo_root, &absolute_output)?;
    if let Some(parent) = absolute_output.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(&absolute_output, &report)
        .with_context(|| format!("write {}", absolute_output.display()))?;
    if report.ready_for_first_hpc_run {
        Ok(report)
    } else {
        Err(anyhow!("local HPC dry-run readiness failed; see {}", report.output_path))
    }
}

pub(crate) fn validate_hpc_dry_run_ready_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<LocalHpcDryRunReadyReport> {
    let absolute_manifest_path = if manifest_path.is_absolute() {
        manifest_path.to_path_buf()
    } else {
        repo_root.join(manifest_path)
    };
    let report = build_hpc_dry_run_ready(repo_root, &absolute_manifest_path)?;
    let observed = fs::read(&absolute_manifest_path)
        .with_context(|| format!("read {}", absolute_manifest_path.display()))?;
    let expected =
        serde_json::to_vec_pretty(&report).context("serialize governed HPC dry-run readiness")?;
    if observed != expected {
        return Err(anyhow!(
            "HPC dry-run readiness report `{}` drifted from governed dry-run inputs; rerun `bijux-dna bench local render-hpc-dry-run-ready --output {}`",
            absolute_manifest_path.display(),
            report.output_path
        ));
    }
    if report.ready_for_first_hpc_run {
        Ok(report)
    } else {
        Err(anyhow!("local HPC dry-run readiness failed; see {}", report.output_path))
    }
}

fn build_hpc_dry_run_ready(
    repo_root: &Path,
    absolute_output_path: &Path,
) -> Result<LocalHpcDryRunReadyReport> {
    ensure_path_stays_within_benchmark_readiness_hpc_root(
        repo_root,
        absolute_output_path,
        "HPC dry-run readiness output",
    )?;

    let asset_report = collect_hpc_asset_staging_manifest(repo_root);
    let scratch_report = collect_hpc_scratch_layout(repo_root);
    let execution_report = collect_hpc_execution_resolver_report(repo_root);
    let stage_benchmark_report = collect_hpc_stage_benchmark_array(repo_root);
    let pipeline_node_report = collect_hpc_pipeline_node_array(repo_root);
    let dependency_report = collect_hpc_dependency_simulation(repo_root);
    let resume_report = collect_hpc_resume_simulation(repo_root);
    let result_collection_report = collect_hpc_result_collection_simulation(repo_root);
    let candidate_report = collect_hpc_candidate_run_manifest(repo_root);

    let checks = vec![
        build_goal_481_check(
            asset_report.as_ref().ok(),
            stage_benchmark_report.as_ref().ok(),
            candidate_report.as_ref().ok(),
            asset_report.as_ref().err(),
            stage_benchmark_report.as_ref().err(),
            candidate_report.as_ref().err(),
        ),
        build_goal_482_check(
            scratch_report.as_ref().ok(),
            stage_benchmark_report.as_ref().ok(),
            pipeline_node_report.as_ref().ok(),
            dependency_report.as_ref().ok(),
            resume_report.as_ref().ok(),
            scratch_report.as_ref().err(),
            stage_benchmark_report.as_ref().err(),
            pipeline_node_report.as_ref().err(),
            dependency_report.as_ref().err(),
            resume_report.as_ref().err(),
        ),
        build_goal_483_check(
            execution_report.as_ref().ok(),
            candidate_report.as_ref().ok(),
            execution_report.as_ref().err(),
            candidate_report.as_ref().err(),
        ),
        build_goal_484_check(
            stage_benchmark_report.as_ref().ok(),
            asset_report.as_ref().ok(),
            candidate_report.as_ref().ok(),
            stage_benchmark_report.as_ref().err(),
            asset_report.as_ref().err(),
            candidate_report.as_ref().err(),
        ),
        build_goal_485_check(
            pipeline_node_report.as_ref().ok(),
            scratch_report.as_ref().ok(),
            pipeline_node_report.as_ref().err(),
            scratch_report.as_ref().err(),
        ),
        build_goal_486_check(
            dependency_report.as_ref().ok(),
            scratch_report.as_ref().ok(),
            dependency_report.as_ref().err(),
            scratch_report.as_ref().err(),
        ),
        build_goal_487_check(
            resume_report.as_ref().ok(),
            scratch_report.as_ref().ok(),
            resume_report.as_ref().err(),
            scratch_report.as_ref().err(),
        ),
        build_goal_488_check(
            result_collection_report.as_ref().ok(),
            result_collection_report.as_ref().err(),
        ),
        build_goal_489_check(
            candidate_report.as_ref().ok(),
            asset_report.as_ref().ok(),
            stage_benchmark_report.as_ref().ok(),
            execution_report.as_ref().ok(),
            result_collection_report.as_ref().ok(),
            candidate_report.as_ref().err(),
            asset_report.as_ref().err(),
            stage_benchmark_report.as_ref().err(),
            execution_report.as_ref().err(),
            result_collection_report.as_ref().err(),
        ),
    ];

    let failing_goal_ids = checks
        .iter()
        .filter(|check| !check.ok)
        .map(|check| check.goal_id)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let summary = build_summary(
        asset_report.as_ref().ok(),
        scratch_report.as_ref().ok(),
        execution_report.as_ref().ok(),
        stage_benchmark_report.as_ref().ok(),
        pipeline_node_report.as_ref().ok(),
        dependency_report.as_ref().ok(),
        resume_report.as_ref().ok(),
        result_collection_report.as_ref().ok(),
        candidate_report.as_ref().ok(),
    );
    let behavior = build_behavior(
        &checks,
        scratch_report.as_ref().ok(),
        execution_report.as_ref().ok(),
        pipeline_node_report.as_ref().ok(),
        dependency_report.as_ref().ok(),
        resume_report.as_ref().ok(),
        result_collection_report.as_ref().ok(),
        candidate_report.as_ref().ok(),
    );

    Ok(LocalHpcDryRunReadyReport {
        schema_version: LOCAL_HPC_DRY_RUN_READY_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, absolute_output_path),
        checked_goal_count: checks.len(),
        passed_goal_count: checks.iter().filter(|check| check.ok).count(),
        failed_goal_count: failing_goal_ids.len(),
        ready_for_first_hpc_run: failing_goal_ids.is_empty() && behavior.proven,
        failing_goal_ids,
        summary,
        behavior,
        checks,
    })
}

fn build_summary(
    asset_report: Option<&LocalHpcAssetStagingManifest>,
    scratch_report: Option<&LocalHpcScratchLayout>,
    execution_report: Option<&LocalHpcExecutionResolverReport>,
    stage_benchmark_report: Option<&LocalHpcStageBenchmarkArrayReport>,
    pipeline_node_report: Option<&LocalHpcPipelineNodeArrayReport>,
    dependency_report: Option<&LocalHpcDependencySimulationReport>,
    resume_report: Option<&LocalHpcResumeSimulationReport>,
    result_collection_report: Option<&LocalHpcResultCollectionSimulationReport>,
    candidate_report: Option<&LocalHpcCandidateRunManifest>,
) -> LocalHpcDryRunReadySummary {
    LocalHpcDryRunReadySummary {
        benchmark_job_count: stage_benchmark_report
            .map(|report| report.benchmark_job_count)
            .or_else(|| asset_report.map(|report| report.selected_job_count))
            .unwrap_or(0),
        essential_pipeline_job_count: pipeline_node_report
            .map(|report| report.pipeline_job_count)
            .or_else(|| scratch_report.map(|report| report.essential_pipeline_job_count))
            .unwrap_or(0),
        staged_input_count: asset_report.map_or(0, |report| report.staged_input_count),
        execution_resolver_row_count: execution_report.map_or(0, |report| report.row_count),
        execution_unclassified_tool_count: execution_report
            .map_or(0, |report| report.unclassified_tool_count),
        stage_benchmark_row_count: stage_benchmark_report.map_or(0, |report| report.rows.len()),
        pipeline_node_row_count: pipeline_node_report.map_or(0, |report| report.rows.len()),
        dependency_case_count: dependency_report.map_or(0, |report| report.case_count),
        resume_rerun_job_count: resume_report.map_or(0, |report| report.rerun_job_count),
        result_collection_row_count: result_collection_report.map_or(0, |report| report.row_count),
        candidate_job_count: candidate_report.map_or(0, |report| report.selected_job_count),
        candidate_representative_group_count: candidate_report
            .map_or(0, |report| report.representative_group_count),
        candidate_selected_input_bytes: candidate_report
            .map(|report| report.selected_staged_input_bytes)
            .unwrap_or(0),
    }
}

fn build_behavior(
    checks: &[LocalHpcDryRunReadyGoalCheck],
    scratch_report: Option<&LocalHpcScratchLayout>,
    execution_report: Option<&LocalHpcExecutionResolverReport>,
    pipeline_node_report: Option<&LocalHpcPipelineNodeArrayReport>,
    dependency_report: Option<&LocalHpcDependencySimulationReport>,
    resume_report: Option<&LocalHpcResumeSimulationReport>,
    result_collection_report: Option<&LocalHpcResultCollectionSimulationReport>,
    candidate_report: Option<&LocalHpcCandidateRunManifest>,
) -> LocalHpcDryRunReadyBehavior {
    let goals_481_to_489_validated = checks.iter().all(|check| check.ok);
    let benchmark_jobs_keep_known_assets =
        candidate_report.is_some_and(|report| report.behavior.only_known_assets);
    let benchmark_jobs_keep_scratch_layout = scratch_report
        .is_some_and(|report| report.benchmark_job_count > 0 && report.input_link_count > 0);
    let benchmark_jobs_keep_explicit_execution_resolution =
        candidate_report.is_some_and(|report| {
            report.behavior.only_known_execution_modes
                && report.behavior.only_available_execution_resolution
        }) && execution_report.is_some_and(|report| report.row_count > 0);
    let pipeline_nodes_keep_dependency_manifests = pipeline_node_report
        .is_some_and(|report| report.pipeline_job_count > 0 && report.dependency_count > 0);
    let dependency_failures_block_descendants_only =
        dependency_report.is_some_and(|report| report.all_cases_proven);
    let resume_rules_are_explicit = resume_report.is_some_and(|report| report.behavior.proven);
    let result_collection_statuses_are_distinct =
        result_collection_report.is_some_and(|report| report.behavior.proven);
    let first_run_candidate_is_small_and_governed = candidate_report
        .is_some_and(|report| report.behavior.proven && report.selected_job_count > 0);
    let no_manual_steps_or_ambiguity = goals_481_to_489_validated
        && benchmark_jobs_keep_known_assets
        && benchmark_jobs_keep_scratch_layout
        && benchmark_jobs_keep_explicit_execution_resolution
        && pipeline_nodes_keep_dependency_manifests
        && dependency_failures_block_descendants_only
        && resume_rules_are_explicit
        && result_collection_statuses_are_distinct
        && first_run_candidate_is_small_and_governed;

    LocalHpcDryRunReadyBehavior {
        goals_481_to_489_validated,
        benchmark_jobs_keep_known_assets,
        benchmark_jobs_keep_scratch_layout,
        benchmark_jobs_keep_explicit_execution_resolution,
        pipeline_nodes_keep_dependency_manifests,
        dependency_failures_block_descendants_only,
        resume_rules_are_explicit,
        result_collection_statuses_are_distinct,
        first_run_candidate_is_small_and_governed,
        no_manual_steps_or_ambiguity,
        proven: no_manual_steps_or_ambiguity,
    }
}

fn build_goal_481_check(
    asset_report: Option<&LocalHpcAssetStagingManifest>,
    stage_benchmark_report: Option<&LocalHpcStageBenchmarkArrayReport>,
    candidate_report: Option<&LocalHpcCandidateRunManifest>,
    asset_error: Option<&anyhow::Error>,
    stage_benchmark_error: Option<&anyhow::Error>,
    candidate_error: Option<&anyhow::Error>,
) -> LocalHpcDryRunReadyGoalCheck {
    let output_path = Some(
        asset_report
            .map(|report| report.output_path.clone())
            .unwrap_or_else(|| DEFAULT_HPC_ASSET_STAGING_MANIFEST_PATH.to_string()),
    );
    let Some(asset_report) = asset_report else {
        return fail_check(
            481,
            "HPC asset staging dry-run",
            output_path,
            error_detail(asset_error),
        );
    };
    let Some(stage_benchmark_report) = stage_benchmark_report else {
        return fail_check(
            481,
            "HPC asset staging dry-run",
            output_path,
            blocked_by("stage benchmark array", stage_benchmark_error),
        );
    };
    let Some(candidate_report) = candidate_report else {
        return fail_check(
            481,
            "HPC asset staging dry-run",
            output_path,
            blocked_by("first HPC candidate manifest", candidate_error),
        );
    };
    let asset_result_ids =
        asset_report.jobs.iter().map(|job| job.result_id.as_str()).collect::<BTreeSet<_>>();
    let missing_candidate_results = candidate_report
        .rows
        .iter()
        .filter_map(|row| {
            (!asset_result_ids.contains(row.result_id.as_str())).then_some(row.result_id.as_str())
        })
        .collect::<Vec<_>>();
    if asset_report.selected_job_count != stage_benchmark_report.benchmark_job_count {
        return fail_check(
            481,
            "HPC asset staging dry-run",
            output_path,
            format!(
                "asset staging manifest covers {} benchmark jobs but stage benchmark array covers {}",
                asset_report.selected_job_count, stage_benchmark_report.benchmark_job_count
            ),
        );
    }
    if !missing_candidate_results.is_empty() {
        return fail_check(
            481,
            "HPC asset staging dry-run",
            output_path,
            format!(
                "asset staging manifest is missing candidate benchmark results: {}",
                missing_candidate_results.join(", ")
            ),
        );
    }
    ok_check(
        481,
        "HPC asset staging dry-run",
        output_path,
        format!(
            "validated {} benchmark jobs with {} staged inputs and candidate coverage for all {} first-run rows",
            asset_report.selected_job_count,
            asset_report.staged_input_count,
            candidate_report.selected_job_count
        ),
    )
}

fn build_goal_482_check(
    scratch_report: Option<&LocalHpcScratchLayout>,
    stage_benchmark_report: Option<&LocalHpcStageBenchmarkArrayReport>,
    pipeline_node_report: Option<&LocalHpcPipelineNodeArrayReport>,
    dependency_report: Option<&LocalHpcDependencySimulationReport>,
    resume_report: Option<&LocalHpcResumeSimulationReport>,
    scratch_error: Option<&anyhow::Error>,
    stage_benchmark_error: Option<&anyhow::Error>,
    pipeline_node_error: Option<&anyhow::Error>,
    dependency_error: Option<&anyhow::Error>,
    resume_error: Option<&anyhow::Error>,
) -> LocalHpcDryRunReadyGoalCheck {
    let output_path = Some(
        scratch_report
            .map(|report| report.output_path.clone())
            .unwrap_or_else(|| DEFAULT_HPC_SCRATCH_LAYOUT_PATH.to_string()),
    );
    let Some(scratch_report) = scratch_report else {
        return fail_check(
            482,
            "HPC scratch layout dry-run",
            output_path,
            error_detail(scratch_error),
        );
    };
    let Some(stage_benchmark_report) = stage_benchmark_report else {
        return fail_check(
            482,
            "HPC scratch layout dry-run",
            output_path,
            blocked_by("stage benchmark array", stage_benchmark_error),
        );
    };
    let Some(pipeline_node_report) = pipeline_node_report else {
        return fail_check(
            482,
            "HPC scratch layout dry-run",
            output_path,
            blocked_by("pipeline node array", pipeline_node_error),
        );
    };
    let Some(dependency_report) = dependency_report else {
        return fail_check(
            482,
            "HPC scratch layout dry-run",
            output_path,
            blocked_by("dependency simulation", dependency_error),
        );
    };
    let Some(resume_report) = resume_report else {
        return fail_check(
            482,
            "HPC scratch layout dry-run",
            output_path,
            blocked_by("resume simulation", resume_error),
        );
    };
    if scratch_report.benchmark_job_count != stage_benchmark_report.benchmark_job_count {
        return fail_check(
            482,
            "HPC scratch layout dry-run",
            output_path,
            format!(
                "scratch layout covers {} benchmark jobs but stage benchmark array covers {}",
                scratch_report.benchmark_job_count, stage_benchmark_report.benchmark_job_count
            ),
        );
    }
    if scratch_report.essential_pipeline_job_count != pipeline_node_report.pipeline_job_count {
        return fail_check(
            482,
            "HPC scratch layout dry-run",
            output_path,
            format!(
                "scratch layout covers {} essential pipeline jobs but pipeline node array covers {}",
                scratch_report.essential_pipeline_job_count, pipeline_node_report.pipeline_job_count
            ),
        );
    }
    for (surface, count) in [
        ("dependency simulation", dependency_report.job_count),
        ("resume simulation", resume_report.job_count),
    ] {
        if scratch_report.selected_job_count != count {
            return fail_check(
                482,
                "HPC scratch layout dry-run",
                output_path,
                format!(
                    "scratch layout covers {} selected jobs but {} covers {}",
                    scratch_report.selected_job_count, surface, count
                ),
            );
        }
    }
    ok_check(
        482,
        "HPC scratch layout dry-run",
        output_path,
        format!(
            "validated {} total jobs with {} benchmark roots, {} pipeline roots, and {} staged input links",
            scratch_report.selected_job_count,
            scratch_report.benchmark_job_count,
            scratch_report.essential_pipeline_job_count,
            scratch_report.input_link_count
        ),
    )
}

fn build_goal_483_check(
    execution_report: Option<&LocalHpcExecutionResolverReport>,
    candidate_report: Option<&LocalHpcCandidateRunManifest>,
    execution_error: Option<&anyhow::Error>,
    candidate_error: Option<&anyhow::Error>,
) -> LocalHpcDryRunReadyGoalCheck {
    let output_path = Some(
        execution_report
            .map(|report| report.output_path.clone())
            .unwrap_or_else(|| DEFAULT_HPC_EXECUTION_RESOLVER_PATH.to_string()),
    );
    let Some(execution_report) = execution_report else {
        return fail_check(
            483,
            "HPC execution resolver",
            output_path,
            error_detail(execution_error),
        );
    };
    let Some(candidate_report) = candidate_report else {
        return fail_check(
            483,
            "HPC execution resolver",
            output_path,
            blocked_by("first HPC candidate manifest", candidate_error),
        );
    };
    let execution_by_tool = execution_report
        .rows
        .iter()
        .map(|row| (row.tool_id.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    for row in &candidate_report.rows {
        let Some(execution_row) = execution_by_tool.get(row.tool_id.as_str()) else {
            return fail_check(
                483,
                "HPC execution resolver",
                output_path,
                format!("execution resolver is missing candidate tool `{}`", row.tool_id),
            );
        };
        if execution_row.execution_mode == "unclassified"
            || execution_row.resolution_kind.contains("unavailable")
        {
            return fail_check(
                483,
                "HPC execution resolver",
                output_path,
                format!(
                    "candidate tool `{}` remains unresolved: execution_mode=`{}` resolution_kind=`{}`",
                    row.tool_id, execution_row.execution_mode, execution_row.resolution_kind
                ),
            );
        }
    }
    ok_check(
        483,
        "HPC execution resolver",
        output_path,
        format!(
            "validated {} selected-tool rows; first-run candidate resolves {} tools without ambiguity",
            execution_report.row_count, candidate_report.selected_job_count
        ),
    )
}

fn build_goal_484_check(
    stage_benchmark_report: Option<&LocalHpcStageBenchmarkArrayReport>,
    asset_report: Option<&LocalHpcAssetStagingManifest>,
    candidate_report: Option<&LocalHpcCandidateRunManifest>,
    stage_benchmark_error: Option<&anyhow::Error>,
    asset_error: Option<&anyhow::Error>,
    candidate_error: Option<&anyhow::Error>,
) -> LocalHpcDryRunReadyGoalCheck {
    let output_path = Some(
        stage_benchmark_report
            .map(|report| report.script_path.clone())
            .unwrap_or_else(|| DEFAULT_HPC_STAGE_BENCHMARK_ARRAY_SCRIPT_PATH.to_string()),
    );
    let Some(stage_benchmark_report) = stage_benchmark_report else {
        return fail_check(
            484,
            "SLURM array script for stage benchmarks",
            output_path,
            error_detail(stage_benchmark_error),
        );
    };
    let Some(asset_report) = asset_report else {
        return fail_check(
            484,
            "SLURM array script for stage benchmarks",
            output_path,
            blocked_by("asset staging manifest", asset_error),
        );
    };
    let Some(candidate_report) = candidate_report else {
        return fail_check(
            484,
            "SLURM array script for stage benchmarks",
            output_path,
            blocked_by("first HPC candidate manifest", candidate_error),
        );
    };
    let stage_results = stage_benchmark_report
        .rows
        .iter()
        .map(|row| row.result_id.as_str())
        .collect::<BTreeSet<_>>();
    let missing_candidate_results = candidate_report
        .rows
        .iter()
        .filter_map(|row| {
            (!stage_results.contains(row.result_id.as_str())).then_some(row.result_id.as_str())
        })
        .collect::<Vec<_>>();
    if stage_benchmark_report.benchmark_job_count != asset_report.selected_job_count {
        return fail_check(
            484,
            "SLURM array script for stage benchmarks",
            output_path,
            format!(
                "stage benchmark array covers {} rows but asset staging manifest covers {} benchmark jobs",
                stage_benchmark_report.benchmark_job_count, asset_report.selected_job_count
            ),
        );
    }
    if !missing_candidate_results.is_empty() {
        return fail_check(
            484,
            "SLURM array script for stage benchmarks",
            output_path,
            format!(
                "stage benchmark array is missing candidate result IDs: {}",
                missing_candidate_results.join(", ")
            ),
        );
    }
    ok_check(
        484,
        "SLURM array script for stage benchmarks",
        output_path,
        format!(
            "validated contiguous array mapping for {} benchmark rows with governed manifest {}",
            stage_benchmark_report.benchmark_job_count, stage_benchmark_report.manifest_path
        ),
    )
}

fn build_goal_485_check(
    pipeline_node_report: Option<&LocalHpcPipelineNodeArrayReport>,
    scratch_report: Option<&LocalHpcScratchLayout>,
    pipeline_node_error: Option<&anyhow::Error>,
    scratch_error: Option<&anyhow::Error>,
) -> LocalHpcDryRunReadyGoalCheck {
    let output_path = Some(
        pipeline_node_report
            .map(|report| report.script_path.clone())
            .unwrap_or_else(|| DEFAULT_HPC_PIPELINE_NODE_ARRAY_SCRIPT_PATH.to_string()),
    );
    let Some(pipeline_node_report) = pipeline_node_report else {
        return fail_check(
            485,
            "SLURM array script for pipeline nodes",
            output_path,
            error_detail(pipeline_node_error),
        );
    };
    let Some(scratch_report) = scratch_report else {
        return fail_check(
            485,
            "SLURM array script for pipeline nodes",
            output_path,
            blocked_by("scratch layout", scratch_error),
        );
    };
    if pipeline_node_report.pipeline_job_count != scratch_report.essential_pipeline_job_count {
        return fail_check(
            485,
            "SLURM array script for pipeline nodes",
            output_path,
            format!(
                "pipeline node array covers {} rows but scratch layout covers {} essential pipeline jobs",
                pipeline_node_report.pipeline_job_count, scratch_report.essential_pipeline_job_count
            ),
        );
    }
    if pipeline_node_report.dependency_count == 0 {
        return fail_check(
            485,
            "SLURM array script for pipeline nodes",
            output_path,
            "pipeline node array did not preserve any dependency entries".to_string(),
        );
    }
    ok_check(
        485,
        "SLURM array script for pipeline nodes",
        output_path,
        format!(
            "validated {} pipeline-node rows with {} explicit dependency edges",
            pipeline_node_report.pipeline_job_count, pipeline_node_report.dependency_count
        ),
    )
}

fn build_goal_486_check(
    dependency_report: Option<&LocalHpcDependencySimulationReport>,
    scratch_report: Option<&LocalHpcScratchLayout>,
    dependency_error: Option<&anyhow::Error>,
    scratch_error: Option<&anyhow::Error>,
) -> LocalHpcDryRunReadyGoalCheck {
    let output_path = Some(
        dependency_report
            .map(|report| report.output_path.clone())
            .unwrap_or_else(|| DEFAULT_HPC_DEPENDENCY_SIMULATION_PATH.to_string()),
    );
    let Some(dependency_report) = dependency_report else {
        return fail_check(
            486,
            "SLURM dependency simulator",
            output_path,
            error_detail(dependency_error),
        );
    };
    let Some(scratch_report) = scratch_report else {
        return fail_check(
            486,
            "SLURM dependency simulator",
            output_path,
            blocked_by("scratch layout", scratch_error),
        );
    };
    if !dependency_report.all_cases_proven {
        return fail_check(
            486,
            "SLURM dependency simulator",
            output_path,
            "dependency simulation did not prove descendant-only blocking".to_string(),
        );
    }
    if dependency_report.job_count != scratch_report.selected_job_count {
        return fail_check(
            486,
            "SLURM dependency simulator",
            output_path,
            format!(
                "dependency simulation covers {} jobs but scratch layout covers {}",
                dependency_report.job_count, scratch_report.selected_job_count
            ),
        );
    }
    ok_check(
        486,
        "SLURM dependency simulator",
        output_path,
        format!(
            "validated {} failure cases across {} governed HPC jobs",
            dependency_report.case_count, dependency_report.job_count
        ),
    )
}

fn build_goal_487_check(
    resume_report: Option<&LocalHpcResumeSimulationReport>,
    scratch_report: Option<&LocalHpcScratchLayout>,
    resume_error: Option<&anyhow::Error>,
    scratch_error: Option<&anyhow::Error>,
) -> LocalHpcDryRunReadyGoalCheck {
    let output_path = Some(
        resume_report
            .map(|report| report.output_path.clone())
            .unwrap_or_else(|| DEFAULT_HPC_RESUME_SIMULATION_PATH.to_string()),
    );
    let Some(resume_report) = resume_report else {
        return fail_check(487, "HPC resume simulator", output_path, error_detail(resume_error));
    };
    let Some(scratch_report) = scratch_report else {
        return fail_check(
            487,
            "HPC resume simulator",
            output_path,
            blocked_by("scratch layout", scratch_error),
        );
    };
    if !resume_report.behavior.proven {
        return fail_check(
            487,
            "HPC resume simulator",
            output_path,
            "resume simulation did not prove skip/rerun behavior".to_string(),
        );
    }
    if resume_report.job_count != scratch_report.selected_job_count {
        return fail_check(
            487,
            "HPC resume simulator",
            output_path,
            format!(
                "resume simulation covers {} jobs but scratch layout covers {}",
                resume_report.job_count, scratch_report.selected_job_count
            ),
        );
    }
    ok_check(
        487,
        "HPC resume simulator",
        output_path,
        format!(
            "validated explicit resume behavior across {} jobs with {} reruns and {} skips",
            resume_report.job_count, resume_report.rerun_job_count, resume_report.skip_job_count
        ),
    )
}

fn build_goal_488_check(
    result_collection_report: Option<&LocalHpcResultCollectionSimulationReport>,
    result_collection_error: Option<&anyhow::Error>,
) -> LocalHpcDryRunReadyGoalCheck {
    let output_path = Some(
        result_collection_report
            .map(|report| report.output_path.clone())
            .unwrap_or_else(|| DEFAULT_HPC_RESULT_COLLECTION_SIMULATION_PATH.to_string()),
    );
    let Some(result_collection_report) = result_collection_report else {
        return fail_check(
            488,
            "HPC result collection simulator",
            output_path,
            error_detail(result_collection_error),
        );
    };
    if !result_collection_report.behavior.proven {
        return fail_check(
            488,
            "HPC result collection simulator",
            output_path,
            "result-collection simulation did not prove distinct completion states".to_string(),
        );
    }
    let required_status_counts = [
        result_collection_report.complete_row_count,
        result_collection_report.failed_row_count,
        result_collection_report.missing_row_count,
        result_collection_report.insufficient_row_count,
        result_collection_report.unavailable_row_count,
    ];
    if required_status_counts.into_iter().any(|count| count == 0) {
        return fail_check(
            488,
            "HPC result collection simulator",
            output_path,
            "result-collection simulation no longer covers all five required collection states"
                .to_string(),
        );
    }
    ok_check(
        488,
        "HPC result collection simulator",
        output_path,
        format!(
            "validated {} collection rows spanning complete/failed/missing/insufficient/unavailable cases",
            result_collection_report.row_count
        ),
    )
}

fn build_goal_489_check(
    candidate_report: Option<&LocalHpcCandidateRunManifest>,
    asset_report: Option<&LocalHpcAssetStagingManifest>,
    stage_benchmark_report: Option<&LocalHpcStageBenchmarkArrayReport>,
    execution_report: Option<&LocalHpcExecutionResolverReport>,
    result_collection_report: Option<&LocalHpcResultCollectionSimulationReport>,
    candidate_error: Option<&anyhow::Error>,
    asset_error: Option<&anyhow::Error>,
    stage_benchmark_error: Option<&anyhow::Error>,
    execution_error: Option<&anyhow::Error>,
    result_collection_error: Option<&anyhow::Error>,
) -> LocalHpcDryRunReadyGoalCheck {
    let output_path = Some(
        candidate_report
            .map(|report| report.output_path.clone())
            .unwrap_or_else(|| DEFAULT_HPC_CANDIDATE_RUN_MANIFEST_PATH.to_string()),
    );
    let Some(candidate_report) = candidate_report else {
        return fail_check(
            489,
            "First small HPC candidate manifest",
            output_path,
            error_detail(candidate_error),
        );
    };
    let Some(asset_report) = asset_report else {
        return fail_check(
            489,
            "First small HPC candidate manifest",
            output_path,
            blocked_by("asset staging manifest", asset_error),
        );
    };
    let Some(stage_benchmark_report) = stage_benchmark_report else {
        return fail_check(
            489,
            "First small HPC candidate manifest",
            output_path,
            blocked_by("stage benchmark array", stage_benchmark_error),
        );
    };
    let Some(execution_report) = execution_report else {
        return fail_check(
            489,
            "First small HPC candidate manifest",
            output_path,
            blocked_by("execution resolver", execution_error),
        );
    };
    let Some(result_collection_report) = result_collection_report else {
        return fail_check(
            489,
            "First small HPC candidate manifest",
            output_path,
            blocked_by("result-collection simulation", result_collection_error),
        );
    };
    if !candidate_report.behavior.proven {
        return fail_check(
            489,
            "First small HPC candidate manifest",
            output_path,
            "candidate manifest no longer proves the small governed first-run contract".to_string(),
        );
    }
    let asset_result_ids =
        asset_report.jobs.iter().map(|job| job.result_id.as_str()).collect::<BTreeSet<_>>();
    let stage_result_ids = stage_benchmark_report
        .rows
        .iter()
        .map(|row| row.result_id.as_str())
        .collect::<BTreeSet<_>>();
    let resolver_by_tool = execution_report
        .rows
        .iter()
        .map(|row| (row.tool_id.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    for row in &candidate_report.rows {
        if !asset_result_ids.contains(row.result_id.as_str()) {
            return fail_check(
                489,
                "First small HPC candidate manifest",
                output_path,
                format!(
                    "candidate result `{}` is missing from asset staging manifest",
                    row.result_id
                ),
            );
        }
        if !stage_result_ids.contains(row.result_id.as_str()) {
            return fail_check(
                489,
                "First small HPC candidate manifest",
                output_path,
                format!(
                    "candidate result `{}` is missing from stage benchmark array",
                    row.result_id
                ),
            );
        }
        let Some(execution_row) = resolver_by_tool.get(row.tool_id.as_str()) else {
            return fail_check(
                489,
                "First small HPC candidate manifest",
                output_path,
                format!("candidate tool `{}` is missing from execution resolver", row.tool_id),
            );
        };
        if execution_row.execution_mode == "unclassified"
            || execution_row.resolution_kind.contains("unavailable")
        {
            return fail_check(
                489,
                "First small HPC candidate manifest",
                output_path,
                format!(
                    "candidate tool `{}` remains ambiguous: execution_mode=`{}` resolution_kind=`{}`",
                    row.tool_id, execution_row.execution_mode, execution_row.resolution_kind
                ),
            );
        }
    }
    if !result_collection_report.behavior.proven {
        return fail_check(
            489,
            "First small HPC candidate manifest",
            output_path,
            "candidate stop conditions are no longer backed by a proven result-collection simulator"
                .to_string(),
        );
    }
    ok_check(
        489,
        "First small HPC candidate manifest",
        output_path,
        format!(
            "validated {} first-run rows across {} representative groups with {} staged input bytes",
            candidate_report.selected_job_count,
            candidate_report.representative_group_count,
            candidate_report.selected_staged_input_bytes
        ),
    )
}

fn ok_check(
    goal_id: u32,
    surface: &str,
    output_path: Option<String>,
    detail: impl Into<String>,
) -> LocalHpcDryRunReadyGoalCheck {
    LocalHpcDryRunReadyGoalCheck {
        goal_id,
        surface: surface.to_string(),
        output_path,
        ok: true,
        detail: detail.into(),
    }
}

fn fail_check(
    goal_id: u32,
    surface: &str,
    output_path: Option<String>,
    detail: impl Into<String>,
) -> LocalHpcDryRunReadyGoalCheck {
    LocalHpcDryRunReadyGoalCheck {
        goal_id,
        surface: surface.to_string(),
        output_path,
        ok: false,
        detail: detail.into(),
    }
}

fn error_detail(error: Option<&anyhow::Error>) -> String {
    error.map(|error| format!("{error:#}")).unwrap_or_else(|| "validation failed".to_string())
}

fn blocked_by(surface: &str, error: Option<&anyhow::Error>) -> String {
    match error {
        Some(error) => format!("{surface} validation failed earlier: {error:#}"),
        None => format!("{surface} validation failed earlier"),
    }
}
