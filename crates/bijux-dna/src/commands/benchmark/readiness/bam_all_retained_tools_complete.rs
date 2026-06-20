use std::collections::BTreeSet;
use std::env;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};

use super::bam_active_row_consistency;
use super::bam_adapter_output_contract;
use super::bam_authenticity_complete;
use super::bam_command_adapter_coverage;
use super::bam_contamination_complete;
use super::bam_damage_complete;
use super::bam_endogenous_content_complete;
use super::bam_genotyping_complete;
use super::bam_haplogroups_complete;
use super::bam_kinship_complete;
use super::bam_local_container_smoke;
use super::bam_overlap_correction_complete;
use super::bam_parser_fixture_coverage;
use super::bam_recalibration_complete;
use super::bam_rendered_commands;
use super::bam_report_map;
use super::bam_science_thresholds_ready;
use super::bam_sex_complete;
use super::expected_benchmark_results;
use super::tool_serving_map;
use crate::commands::benchmark::local_bam_micro_smoke_subset;
use crate::commands::benchmark::readiness::bam_command_adapter_coverage::BamBenchmarkStatus;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_BAM_ALL_RETAINED_TOOLS_COMPLETE_PATH: &str =
    "benchmarks/readiness/bam/BAM_ALL_RETAINED_TOOLS_COMPLETE.json";
const BAM_ALL_RETAINED_TOOLS_COMPLETE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_all_retained_tools_complete.v1";
const EXPECTED_CHECKED_GOAL_COUNT: usize = 19;
const EXPECTED_RETAINED_ROW_COUNT: usize = 49;
const EXPECTED_RETAINED_STAGE_COUNT: usize = 24;
const EXPECTED_RETAINED_TOOL_COUNT: usize = 25;
#[cfg(feature = "bam_downstream")]
const EXPECTED_HOST_STAGE_SMOKE_ROW_COUNT: usize = 20;
#[cfg(not(feature = "bam_downstream"))]
const EXPECTED_HOST_STAGE_SMOKE_ROW_COUNT: usize = 18;
#[cfg(feature = "bam_downstream")]
const EXPECTED_CONTAINER_SMOKE_ROW_COUNT: usize = 29;
#[cfg(not(feature = "bam_downstream"))]
const EXPECTED_CONTAINER_SMOKE_ROW_COUNT: usize = 31;
const EXPECTED_ACTIVE_ROW_CONSISTENCY_SURFACE_COUNT: usize = 6;
const EXPECTED_MICRO_SMOKE_FAMILY_COUNT: usize = 12;
const EXPECTED_BAM_COMPARABLE_STAGE_COUNT: usize = 15;
const EXPECTED_BAM_GOVERNED_METRIC_COUNT: usize = 51;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BamStageCompletionGoal {
    goal_id: u32,
    stage_id: &'static str,
    output_path: &'static str,
    expected_active_row_count: usize,
    expected_checked_surface_count: usize,
}

const BAM_STAGE_COMPLETION_GOALS: &[BamStageCompletionGoal] = &[
    BamStageCompletionGoal {
        goal_id: 431,
        stage_id: "bam.overlap_correction",
        output_path: bam_overlap_correction_complete::DEFAULT_BAM_OVERLAP_CORRECTION_COMPLETE_PATH,
        expected_active_row_count: 1,
        expected_checked_surface_count: 11,
    },
    BamStageCompletionGoal {
        goal_id: 432,
        stage_id: "bam.endogenous_content",
        output_path: bam_endogenous_content_complete::DEFAULT_BAM_ENDOGENOUS_CONTENT_COMPLETE_PATH,
        expected_active_row_count: 1,
        expected_checked_surface_count: 11,
    },
    BamStageCompletionGoal {
        goal_id: 433,
        stage_id: "bam.damage",
        output_path: bam_damage_complete::DEFAULT_BAM_DAMAGE_COMPLETE_PATH,
        expected_active_row_count: 6,
        expected_checked_surface_count: 13,
    },
    BamStageCompletionGoal {
        goal_id: 434,
        stage_id: "bam.authenticity",
        output_path: bam_authenticity_complete::DEFAULT_BAM_AUTHENTICITY_COMPLETE_PATH,
        expected_active_row_count: 3,
        expected_checked_surface_count: 12,
    },
    BamStageCompletionGoal {
        goal_id: 435,
        stage_id: "bam.contamination",
        output_path: bam_contamination_complete::DEFAULT_BAM_CONTAMINATION_COMPLETE_PATH,
        expected_active_row_count: 3,
        expected_checked_surface_count: 14,
    },
    BamStageCompletionGoal {
        goal_id: 436,
        stage_id: "bam.sex",
        output_path: bam_sex_complete::DEFAULT_BAM_SEX_COMPLETE_PATH,
        expected_active_row_count: 3,
        expected_checked_surface_count: 15,
    },
    BamStageCompletionGoal {
        goal_id: 437,
        stage_id: "bam.haplogroups",
        output_path: bam_haplogroups_complete::DEFAULT_BAM_HAPLOGROUPS_COMPLETE_PATH,
        expected_active_row_count: 1,
        expected_checked_surface_count: 14,
    },
    BamStageCompletionGoal {
        goal_id: 438,
        stage_id: "bam.recalibration",
        output_path: bam_recalibration_complete::DEFAULT_BAM_RECALIBRATION_COMPLETE_PATH,
        expected_active_row_count: 1,
        expected_checked_surface_count: 13,
    },
    BamStageCompletionGoal {
        goal_id: 439,
        stage_id: "bam.genotyping",
        output_path: bam_genotyping_complete::DEFAULT_BAM_GENOTYPING_COMPLETE_PATH,
        expected_active_row_count: 1,
        expected_checked_surface_count: 15,
    },
    BamStageCompletionGoal {
        goal_id: 440,
        stage_id: "bam.kinship",
        output_path: bam_kinship_complete::DEFAULT_BAM_KINSHIP_COMPLETE_PATH,
        expected_active_row_count: 2,
        expected_checked_surface_count: 19,
    },
];

#[derive(Debug, Clone, Deserialize)]
struct StageCompletionFallbackReport {
    output_path: String,
    active_row_count: usize,
    complete_row_count: usize,
    incomplete_row_count: usize,
    checked_surface_count: usize,
    violation_count: usize,
    ok: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct BamBindingKey {
    stage_id: String,
    tool_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BamAllRetainedToolsCompleteGoalCheck {
    pub(crate) goal_id: u32,
    pub(crate) surface: String,
    pub(crate) output_path: Option<String>,
    pub(crate) ok: bool,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BamAllRetainedToolsCompleteReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) checked_goal_count: usize,
    pub(crate) passed_goal_count: usize,
    pub(crate) failed_goal_count: usize,
    pub(crate) failing_goal_ids: Vec<u32>,
    pub(crate) retained_row_count: usize,
    pub(crate) retained_stage_count: usize,
    pub(crate) retained_tool_count: usize,
    pub(crate) command_adapter_row_count: usize,
    pub(crate) output_declaration_row_count: usize,
    pub(crate) expected_result_row_count: usize,
    pub(crate) rendered_command_row_count: usize,
    pub(crate) parser_fixture_row_count: usize,
    pub(crate) local_smoke_row_count: usize,
    pub(crate) local_smoke_host_stage_row_count: usize,
    pub(crate) local_smoke_container_row_count: usize,
    pub(crate) report_map_row_count: usize,
    pub(crate) active_row_consistency_surface_count: usize,
    pub(crate) micro_smoke_family_count: usize,
    pub(crate) science_threshold_stage_count: usize,
    pub(crate) ok: bool,
    pub(crate) checks: Vec<BamAllRetainedToolsCompleteGoalCheck>,
}

pub(crate) fn run_render_bam_all_retained_tools_complete(
    args: &parse::BenchReadinessRenderBamAllRetainedToolsCompleteArgs,
) -> Result<()> {
    let repo_root = env::current_dir().context("resolve current directory")?;
    let report = render_bam_all_retained_tools_complete(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_ALL_RETAINED_TOOLS_COMPLETE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_bam_all_retained_tools_complete(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamAllRetainedToolsCompleteReport> {
    let _cwd_guard = CurrentDirGuard::change_to(repo_root);
    let absolute_output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = absolute_output_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let tool_map = tool_serving_map::render_bam_tool_serving_map(
        repo_root,
        PathBuf::from(tool_serving_map::DEFAULT_BAM_TOOL_SERVING_MAP_PATH),
    )?;
    let command_coverage = bam_command_adapter_coverage::render_bam_command_adapter_coverage(
        repo_root,
        PathBuf::from(bam_command_adapter_coverage::DEFAULT_BAM_COMMAND_ADAPTER_COVERAGE_PATH),
    )?;
    let parser_coverage = bam_parser_fixture_coverage::render_bam_parser_fixture_coverage(
        repo_root,
        PathBuf::from(bam_parser_fixture_coverage::DEFAULT_BAM_PARSER_FIXTURE_COVERAGE_PATH),
    )?;
    let local_smoke = bam_local_container_smoke::render_bam_local_container_smoke(
        repo_root,
        PathBuf::from(bam_local_container_smoke::DEFAULT_BAM_LOCAL_CONTAINER_SMOKE_PATH),
    )?;
    let output_declarations = bam_adapter_output_contract::render_bam_adapter_output_contract(
        repo_root,
        PathBuf::from(bam_adapter_output_contract::DEFAULT_BAM_ADAPTER_OUTPUT_CONTRACT_PATH),
    )?;
    let rendered_commands = bam_rendered_commands::render_bam_commands(
        repo_root,
        PathBuf::from(bam_rendered_commands::DEFAULT_BAM_RENDERED_COMMANDS_PATH),
    )?;
    let expected_results = expected_benchmark_results::render_expected_benchmark_results(
        repo_root,
        PathBuf::from(expected_benchmark_results::DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH),
    )?;
    let report_map = bam_report_map::render_bam_report_map(
        repo_root,
        PathBuf::from(bam_report_map::DEFAULT_BAM_REPORT_MAP_PATH),
    )?;
    let active_row_consistency = bam_active_row_consistency::render_bam_active_row_consistency(
        repo_root,
        PathBuf::from(bam_active_row_consistency::DEFAULT_BAM_ACTIVE_ROW_CONSISTENCY_PATH),
    )?;
    let micro_smoke_summary = local_bam_micro_smoke_subset::render_bam_micro_smoke_subset(
        repo_root,
        PathBuf::from(local_bam_micro_smoke_subset::DEFAULT_BAM_MICRO_SMOKE_SUMMARY_PATH),
    )?;
    let science_thresholds = bam_science_thresholds_ready::render_bam_science_thresholds_ready(
        repo_root,
        PathBuf::from(bam_science_thresholds_ready::DEFAULT_BAM_SCIENCE_THRESHOLDS_READY_PATH),
    )?;

    let retained_bindings =
        tool_map.rows.iter().map(binding_key_from_tool_serving_map_row).collect::<BTreeSet<_>>();
    let command_bindings = command_coverage
        .rows
        .iter()
        .filter(|row| row.benchmark_status == BamBenchmarkStatus::BenchmarkReady)
        .map(binding_key_from_command_coverage_row)
        .collect::<BTreeSet<_>>();
    let parser_bindings = parser_coverage
        .rows
        .iter()
        .map(binding_key_from_parser_fixture_row)
        .collect::<BTreeSet<_>>();
    let local_smoke_bindings =
        local_smoke.rows.iter().map(binding_key_from_local_smoke_row).collect::<BTreeSet<_>>();
    let output_declaration_bindings = output_declarations
        .rows
        .iter()
        .map(binding_key_from_output_declaration_row)
        .collect::<BTreeSet<_>>();
    let rendered_command_bindings = rendered_commands
        .rows
        .iter()
        .map(binding_key_from_rendered_command_row)
        .collect::<BTreeSet<_>>();
    let expected_result_bindings = expected_results
        .rows
        .iter()
        .filter(|row| row.domain == "bam")
        .map(binding_key_from_expected_result_row)
        .collect::<BTreeSet<_>>();
    let report_map_bindings =
        report_map.rows.iter().map(binding_key_from_report_map_row).collect::<BTreeSet<_>>();

    let mut checks = Vec::new();

    for goal in BAM_STAGE_COMPLETION_GOALS {
        record_goal_check(
            &mut checks,
            goal.goal_id,
            goal.stage_id.to_string(),
            Some(goal.output_path.to_string()),
            || validate_stage_completion_goal(repo_root, *goal),
        );
    }

    record_goal_check(
        &mut checks,
        441,
        "bam local and container smoke coverage",
        Some(bam_local_container_smoke::DEFAULT_BAM_LOCAL_CONTAINER_SMOKE_PATH.to_string()),
        || {
            if local_smoke.row_count != EXPECTED_RETAINED_ROW_COUNT
                || local_smoke.stage_count != EXPECTED_RETAINED_STAGE_COUNT
                || local_smoke.tool_count != EXPECTED_RETAINED_TOOL_COUNT
                || local_smoke.host_stage_smoke_row_count != EXPECTED_HOST_STAGE_SMOKE_ROW_COUNT
                || local_smoke.container_smoke_row_count != EXPECTED_CONTAINER_SMOKE_ROW_COUNT
            {
                bail!(
                    "BAM local/container smoke drifted: rows={}, stages={}, tools={}, host_stage_smokes={}, container_smokes={}",
                    local_smoke.row_count,
                    local_smoke.stage_count,
                    local_smoke.tool_count,
                    local_smoke.host_stage_smoke_row_count,
                    local_smoke.container_smoke_row_count
                );
            }
            ensure_binding_subset(
                "bam retained local/container smoke",
                "local/container smoke coverage",
                &retained_bindings,
                &local_smoke_bindings,
            )?;
            Ok(format!(
                "validated host-vs-container smoke coverage for every retained BAM binding with {} host smoke row(s) and {} container smoke row(s)",
                local_smoke.host_stage_smoke_row_count,
                local_smoke.container_smoke_row_count
            ))
        },
    );

    record_goal_check(
        &mut checks,
        442,
        "bam parser fixture coverage",
        Some(bam_parser_fixture_coverage::DEFAULT_BAM_PARSER_FIXTURE_COVERAGE_PATH.to_string()),
        || {
            if parser_coverage.row_count != EXPECTED_RETAINED_ROW_COUNT
                || parser_coverage.stage_count != EXPECTED_RETAINED_STAGE_COUNT
                || parser_coverage.tool_count != EXPECTED_RETAINED_TOOL_COUNT
                || parser_coverage.covered_row_count != EXPECTED_RETAINED_ROW_COUNT
                || parser_coverage.missing_row_count != 0
                || (parser_coverage.parser_fixture_coverage_percent - 100.0).abs() > f64::EPSILON
            {
                bail!(
                    "BAM parser fixture coverage drifted: rows={}, stages={}, tools={}, covered={}, missing={}, percent={}",
                    parser_coverage.row_count,
                    parser_coverage.stage_count,
                    parser_coverage.tool_count,
                    parser_coverage.covered_row_count,
                    parser_coverage.missing_row_count,
                    parser_coverage.parser_fixture_coverage_percent
                );
            }
            ensure_binding_subset(
                "bam retained parser fixture coverage",
                "parser fixture coverage",
                &retained_bindings,
                &parser_bindings,
            )?;
            Ok("validated 100% parser fixture coverage for the retained BAM binding slice"
                .to_string())
        },
    );

    record_goal_check(
        &mut checks,
        443,
        "bam rendered commands",
        Some(bam_rendered_commands::DEFAULT_BAM_RENDERED_COMMANDS_PATH.to_string()),
        || {
            if rendered_commands.row_count != EXPECTED_RETAINED_ROW_COUNT
                || rendered_commands.stage_count != EXPECTED_RETAINED_STAGE_COUNT
                || rendered_commands.tool_count != EXPECTED_RETAINED_TOOL_COUNT
                || !rendered_commands.bash_syntax_passed
            {
                bail!(
                    "BAM rendered commands drifted: rows={}, stages={}, tools={}, bash_syntax_passed={}",
                    rendered_commands.row_count,
                    rendered_commands.stage_count,
                    rendered_commands.tool_count,
                    rendered_commands.bash_syntax_passed
                );
            }
            ensure_binding_subset(
                "bam retained command coverage",
                "command coverage",
                &retained_bindings,
                &command_bindings,
            )?;
            ensure_binding_subset(
                "bam retained rendered commands",
                "rendered command coverage",
                &retained_bindings,
                &rendered_command_bindings,
            )?;
            Ok("validated one bash-syntax-checked rendered command bundle for every retained BAM binding".to_string())
        },
    );

    record_goal_check(
        &mut checks,
        444,
        "bam output declarations",
        Some(bam_adapter_output_contract::DEFAULT_BAM_ADAPTER_OUTPUT_CONTRACT_PATH.to_string()),
        || {
            if output_declarations.row_count != EXPECTED_RETAINED_ROW_COUNT
                || output_declarations.adapter_row_count != EXPECTED_RETAINED_ROW_COUNT
                || output_declarations.complete_adapter_row_count != EXPECTED_RETAINED_ROW_COUNT
                || output_declarations.incomplete_adapter_row_count != 0
                || output_declarations.missing_adapter_row_count != 0
            {
                bail!(
                    "BAM output declarations drifted: rows={}, adapter_rows={}, complete={}, incomplete={}, missing_adapter={}",
                    output_declarations.row_count,
                    output_declarations.adapter_row_count,
                    output_declarations.complete_adapter_row_count,
                    output_declarations.incomplete_adapter_row_count,
                    output_declarations.missing_adapter_row_count
                );
            }
            ensure_binding_subset(
                "bam retained output declarations",
                "output declaration coverage",
                &retained_bindings,
                &output_declaration_bindings,
            )?;
            Ok("validated declared raw outputs, normalized metrics, stdout/stderr, and manifests for every retained BAM binding".to_string())
        },
    );

    record_goal_check(
        &mut checks,
        445,
        "bam expected results",
        Some(expected_benchmark_results::DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH.to_string()),
        || {
            if expected_result_bindings.len() != EXPECTED_RETAINED_ROW_COUNT {
                bail!(
                    "BAM expected results drifted: bam_rows={} expected={}",
                    expected_result_bindings.len(),
                    EXPECTED_RETAINED_ROW_COUNT
                );
            }
            ensure_binding_subset(
                "bam retained expected results",
                "expected benchmark results",
                &retained_bindings,
                &expected_result_bindings,
            )?;
            Ok("validated one governed expected-result row for every retained BAM binding"
                .to_string())
        },
    );

    record_goal_check(
        &mut checks,
        446,
        "bam report map",
        Some(bam_report_map::DEFAULT_BAM_REPORT_MAP_PATH.to_string()),
        || {
            if report_map.expected_result_row_count != EXPECTED_RETAINED_ROW_COUNT
                || report_map.row_count != EXPECTED_RETAINED_ROW_COUNT
                || report_map.stage_count != EXPECTED_RETAINED_STAGE_COUNT
                || report_map.tool_count != EXPECTED_RETAINED_TOOL_COUNT
            {
                bail!(
                    "BAM report map drifted: expected_results={}, report_map={}, stages={}, tools={}",
                    report_map.expected_result_row_count,
                    report_map.row_count,
                    report_map.stage_count,
                    report_map.tool_count
                );
            }
            ensure_binding_subset(
                "bam retained report map",
                "report map coverage",
                &retained_bindings,
                &report_map_bindings,
            )?;
            Ok("validated report-map placement for every retained BAM expected-result row"
                .to_string())
        },
    );

    record_goal_check(
        &mut checks,
        447,
        "bam active row consistency",
        Some(bam_active_row_consistency::DEFAULT_BAM_ACTIVE_ROW_CONSISTENCY_PATH.to_string()),
        || {
            if active_row_consistency.active_row_count != EXPECTED_RETAINED_ROW_COUNT
                || active_row_consistency.active_stage_count != EXPECTED_RETAINED_STAGE_COUNT
                || active_row_consistency.active_tool_count != EXPECTED_RETAINED_TOOL_COUNT
                || active_row_consistency.checked_surface_count
                    != EXPECTED_ACTIVE_ROW_CONSISTENCY_SURFACE_COUNT
                || active_row_consistency.passed_surface_count
                    != EXPECTED_ACTIVE_ROW_CONSISTENCY_SURFACE_COUNT
                || active_row_consistency.failed_surface_count != 0
                || !active_row_consistency.ok
            {
                bail!(
                    "BAM active-row consistency drifted: rows={}, stages={}, tools={}, checked_surfaces={}, passed_surfaces={}, failed_surfaces={}, ok={}",
                    active_row_consistency.active_row_count,
                    active_row_consistency.active_stage_count,
                    active_row_consistency.active_tool_count,
                    active_row_consistency.checked_surface_count,
                    active_row_consistency.passed_surface_count,
                    active_row_consistency.failed_surface_count,
                    active_row_consistency.ok
                );
            }
            Ok("validated active BAM rows against rendered commands, output declarations, expected results, parser fixtures, local jobs, and report-map rows".to_string())
        },
    );

    record_goal_check(
        &mut checks,
        448,
        "bam real micro smoke subset",
        Some(local_bam_micro_smoke_subset::DEFAULT_BAM_MICRO_SMOKE_SUMMARY_PATH.to_string()),
        || {
            if micro_smoke_summary.family_count != EXPECTED_MICRO_SMOKE_FAMILY_COUNT
                || !micro_smoke_summary.passes_behavior_test
            {
                bail!(
                    "BAM micro smoke subset drifted: family_count={}, passes_behavior_test={}",
                    micro_smoke_summary.family_count,
                    micro_smoke_summary.passes_behavior_test
                );
            }
            Ok(format!(
                "validated one representative retained BAM micro smoke per stage family with {} host-local row(s), {} container-needed row(s), and {} unavailable row(s)",
                micro_smoke_summary.local_smoke_count,
                micro_smoke_summary.container_needed_count,
                micro_smoke_summary.unavailable_count
            ))
        },
    );

    record_goal_check(
        &mut checks,
        449,
        "bam scientific thresholds",
        Some(bam_science_thresholds_ready::DEFAULT_BAM_SCIENCE_THRESHOLDS_READY_PATH.to_string()),
        || {
            if science_thresholds.comparable_stage_count != EXPECTED_BAM_COMPARABLE_STAGE_COUNT
                || science_thresholds.stage_row_count != EXPECTED_BAM_COMPARABLE_STAGE_COUNT
                || science_thresholds.threshold_declared_stage_count
                    != EXPECTED_BAM_COMPARABLE_STAGE_COUNT
                || science_thresholds.missing_threshold_stage_count != 0
                || science_thresholds.threshold_declared_metric_count
                    != EXPECTED_BAM_GOVERNED_METRIC_COUNT
                || science_thresholds.missing_threshold_metric_count != 0
                || science_thresholds.governed_metric_count != EXPECTED_BAM_GOVERNED_METRIC_COUNT
            {
                bail!(
                    "BAM scientific thresholds drifted: comparable_stages={}, stage_rows={}, declared_stages={}, missing_stages={}, declared_metrics={}, missing_metrics={}, governed_metrics={}",
                    science_thresholds.comparable_stage_count,
                    science_thresholds.stage_row_count,
                    science_thresholds.threshold_declared_stage_count,
                    science_thresholds.missing_threshold_stage_count,
                    science_thresholds.threshold_declared_metric_count,
                    science_thresholds.missing_threshold_metric_count,
                    science_thresholds.governed_metric_count
                );
            }
            Ok("validated scientific pass/fail direction, tolerance, and insufficiency policy for every governed BAM comparable metric".to_string())
        },
    );

    let passed_goal_count = checks.iter().filter(|check| check.ok).count();
    let failing_goal_ids =
        checks.iter().filter(|check| !check.ok).map(|check| check.goal_id).collect::<Vec<_>>();
    let failed_goal_count = failing_goal_ids.len();
    let report = BamAllRetainedToolsCompleteReport {
        schema_version: BAM_ALL_RETAINED_TOOLS_COMPLETE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &absolute_output_path),
        checked_goal_count: checks.len(),
        passed_goal_count,
        failed_goal_count,
        failing_goal_ids,
        retained_row_count: tool_map.row_count,
        retained_stage_count: tool_map.stage_count,
        retained_tool_count: tool_map.tool_count,
        command_adapter_row_count: command_coverage.row_count,
        output_declaration_row_count: output_declarations.row_count,
        expected_result_row_count: expected_result_bindings.len(),
        rendered_command_row_count: rendered_commands.row_count,
        parser_fixture_row_count: parser_coverage.row_count,
        local_smoke_row_count: local_smoke.row_count,
        local_smoke_host_stage_row_count: local_smoke.host_stage_smoke_row_count,
        local_smoke_container_row_count: local_smoke.container_smoke_row_count,
        report_map_row_count: report_map.row_count,
        active_row_consistency_surface_count: active_row_consistency.checked_surface_count,
        micro_smoke_family_count: micro_smoke_summary.family_count,
        science_threshold_stage_count: science_thresholds.stage_row_count,
        ok: failed_goal_count == 0
            && checks.len() == EXPECTED_CHECKED_GOAL_COUNT
            && tool_map.row_count == EXPECTED_RETAINED_ROW_COUNT
            && command_coverage.row_count == EXPECTED_RETAINED_ROW_COUNT
            && output_declarations.row_count == EXPECTED_RETAINED_ROW_COUNT
            && expected_result_bindings.len() == EXPECTED_RETAINED_ROW_COUNT
            && rendered_commands.row_count == EXPECTED_RETAINED_ROW_COUNT
            && parser_coverage.row_count == EXPECTED_RETAINED_ROW_COUNT
            && local_smoke.row_count == EXPECTED_RETAINED_ROW_COUNT
            && report_map.row_count == EXPECTED_RETAINED_ROW_COUNT
            && active_row_consistency.checked_surface_count
                == EXPECTED_ACTIVE_ROW_CONSISTENCY_SURFACE_COUNT
            && micro_smoke_summary.family_count == EXPECTED_MICRO_SMOKE_FAMILY_COUNT
            && micro_smoke_summary.passes_behavior_test
            && science_thresholds.stage_row_count == EXPECTED_BAM_COMPARABLE_STAGE_COUNT
            && science_thresholds.missing_threshold_stage_count == 0
            && science_thresholds.missing_threshold_metric_count == 0,
        checks,
    };
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)?;
    if !report.ok {
        return Err(anyhow!(
            "every governed BAM retained-tool gate from goals 431 through 449 must remain complete"
        ));
    }
    Ok(report)
}

fn ensure_binding_subset(
    scope: &str,
    surface: &str,
    expected: &BTreeSet<BamBindingKey>,
    actual: &BTreeSet<BamBindingKey>,
) -> Result<()> {
    let expected_stage_ids =
        expected.iter().map(|binding| binding.stage_id.clone()).collect::<BTreeSet<_>>();
    let actual_subset = actual
        .iter()
        .filter(|binding| expected_stage_ids.contains(&binding.stage_id))
        .cloned()
        .collect::<BTreeSet<_>>();
    if actual_subset != *expected {
        bail!(
            "{scope} drifted on {surface}: missing={:?} extra={:?}",
            diff_bindings(expected, &actual_subset),
            diff_bindings(&actual_subset, expected)
        );
    }
    Ok(())
}

fn filter_bindings_by_stage_ids(
    bindings: &BTreeSet<BamBindingKey>,
    stage_ids: &[&str],
) -> BTreeSet<BamBindingKey> {
    bindings
        .iter()
        .filter(|binding| stage_ids.contains(&binding.stage_id.as_str()))
        .cloned()
        .collect::<BTreeSet<_>>()
}

fn diff_bindings(left: &BTreeSet<BamBindingKey>, right: &BTreeSet<BamBindingKey>) -> Vec<String> {
    left.difference(right)
        .map(|binding| format!("{}/{}", binding.stage_id, binding.tool_id))
        .collect::<Vec<_>>()
}

fn binding_key_from_tool_serving_map_row(
    row: &tool_serving_map::ToolServingMapRow,
) -> BamBindingKey {
    BamBindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() }
}

fn binding_key_from_command_coverage_row(
    row: &bam_command_adapter_coverage::BamCommandAdapterCoverageRow,
) -> BamBindingKey {
    BamBindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() }
}

fn binding_key_from_parser_fixture_row(
    row: &bam_parser_fixture_coverage::BamParserFixtureCoverageRow,
) -> BamBindingKey {
    BamBindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() }
}

fn binding_key_from_local_smoke_row(
    row: &bam_local_container_smoke::BamLocalContainerSmokeRow,
) -> BamBindingKey {
    BamBindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() }
}

fn binding_key_from_output_declaration_row(
    row: &bam_adapter_output_contract::BamAdapterOutputContractRow,
) -> BamBindingKey {
    BamBindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() }
}

fn binding_key_from_rendered_command_row(
    row: &bam_rendered_commands::BamRenderedCommandRow,
) -> BamBindingKey {
    BamBindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() }
}

fn binding_key_from_expected_result_row(
    row: &expected_benchmark_results::ExpectedBenchmarkResultRow,
) -> BamBindingKey {
    BamBindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() }
}

fn binding_key_from_report_map_row(row: &bam_report_map::BamReportMapRow) -> BamBindingKey {
    BamBindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() }
}

fn validate_stage_completion_goal(
    repo_root: &Path,
    goal: BamStageCompletionGoal,
) -> Result<String> {
    if !cfg!(feature = "bam_downstream")
        && matches!(goal.stage_id, "bam.haplogroups" | "bam.genotyping" | "bam.kinship")
    {
        return validate_checked_in_stage_completion_artifact(repo_root, goal);
    }

    let rendered: Result<String> = match goal.stage_id {
        "bam.overlap_correction" => {
            bam_overlap_correction_complete::render_bam_overlap_correction_complete(
                repo_root,
                PathBuf::from(goal.output_path),
            )
            .and_then(|report| {
                validate_stage_completion_counts(
                    goal,
                    report.complete_row_count,
                    report.active_row_count,
                    report.checked_surface_count,
                )?;
                Ok(format_stage_completion_detail(
                    goal.stage_id,
                    report.complete_row_count,
                    report.active_row_count,
                    report.checked_surface_count,
                ))
            })
        }
        "bam.endogenous_content" => {
            bam_endogenous_content_complete::render_bam_endogenous_content_complete(
                repo_root,
                PathBuf::from(goal.output_path),
            )
            .and_then(|report| {
                validate_stage_completion_counts(
                    goal,
                    report.complete_row_count,
                    report.active_row_count,
                    report.checked_surface_count,
                )?;
                Ok(format_stage_completion_detail(
                    goal.stage_id,
                    report.complete_row_count,
                    report.active_row_count,
                    report.checked_surface_count,
                ))
            })
        }
        "bam.damage" => bam_damage_complete::render_bam_damage_complete(
            repo_root,
            PathBuf::from(goal.output_path),
        )
        .and_then(|report| {
            validate_stage_completion_counts(
                goal,
                report.complete_row_count,
                report.active_row_count,
                report.checked_surface_count,
            )?;
            Ok(format_stage_completion_detail(
                goal.stage_id,
                report.complete_row_count,
                report.active_row_count,
                report.checked_surface_count,
            ))
        }),
        "bam.authenticity" => bam_authenticity_complete::render_bam_authenticity_complete(
            repo_root,
            PathBuf::from(goal.output_path),
        )
        .and_then(|report| {
            validate_stage_completion_counts(
                goal,
                report.complete_row_count,
                report.active_row_count,
                report.checked_surface_count,
            )?;
            Ok(format_stage_completion_detail(
                goal.stage_id,
                report.complete_row_count,
                report.active_row_count,
                report.checked_surface_count,
            ))
        }),
        "bam.contamination" => bam_contamination_complete::render_bam_contamination_complete(
            repo_root,
            PathBuf::from(goal.output_path),
        )
        .and_then(|report| {
            validate_stage_completion_counts(
                goal,
                report.complete_row_count,
                report.active_row_count,
                report.checked_surface_count,
            )?;
            Ok(format_stage_completion_detail(
                goal.stage_id,
                report.complete_row_count,
                report.active_row_count,
                report.checked_surface_count,
            ))
        }),
        "bam.sex" => {
            bam_sex_complete::render_bam_sex_complete(repo_root, PathBuf::from(goal.output_path))
                .and_then(|report| {
                    validate_stage_completion_counts(
                        goal,
                        report.complete_row_count,
                        report.active_row_count,
                        report.checked_surface_count,
                    )?;
                    Ok(format_stage_completion_detail(
                        goal.stage_id,
                        report.complete_row_count,
                        report.active_row_count,
                        report.checked_surface_count,
                    ))
                })
        }
        "bam.haplogroups" => bam_haplogroups_complete::render_bam_haplogroups_complete(
            repo_root,
            PathBuf::from(goal.output_path),
        )
        .and_then(|report| {
            validate_stage_completion_counts(
                goal,
                report.complete_row_count,
                report.active_row_count,
                report.checked_surface_count,
            )?;
            Ok(format_stage_completion_detail(
                goal.stage_id,
                report.complete_row_count,
                report.active_row_count,
                report.checked_surface_count,
            ))
        }),
        "bam.recalibration" => bam_recalibration_complete::render_bam_recalibration_complete(
            repo_root,
            PathBuf::from(goal.output_path),
        )
        .and_then(|report| {
            validate_stage_completion_counts(
                goal,
                report.complete_row_count,
                report.active_row_count,
                report.checked_surface_count,
            )?;
            Ok(format_stage_completion_detail(
                goal.stage_id,
                report.complete_row_count,
                report.active_row_count,
                report.checked_surface_count,
            ))
        }),
        "bam.genotyping" => bam_genotyping_complete::render_bam_genotyping_complete(
            repo_root,
            PathBuf::from(goal.output_path),
        )
        .and_then(|report| {
            validate_stage_completion_counts(
                goal,
                report.complete_row_count,
                report.active_row_count,
                report.checked_surface_count,
            )?;
            Ok(format_stage_completion_detail(
                goal.stage_id,
                report.complete_row_count,
                report.active_row_count,
                report.checked_surface_count,
            ))
        }),
        "bam.kinship" => bam_kinship_complete::render_bam_kinship_complete(
            repo_root,
            PathBuf::from(goal.output_path),
        )
        .and_then(|report| {
            validate_stage_completion_counts(
                goal,
                report.complete_row_count,
                report.active_row_count,
                report.checked_surface_count,
            )?;
            Ok(format_stage_completion_detail(
                goal.stage_id,
                report.complete_row_count,
                report.active_row_count,
                report.checked_surface_count,
            ))
        }),
        _ => bail!("unknown BAM stage completion goal `{}`", goal.stage_id),
    };

    match rendered {
        Ok(detail) => Ok(detail),
        Err(error) if error.to_string().contains("`bam_downstream` feature") => {
            validate_checked_in_stage_completion_artifact(repo_root, goal)
        }
        Err(error) => Err(error),
    }
}

fn validate_stage_completion_counts(
    goal: BamStageCompletionGoal,
    complete_row_count: usize,
    active_row_count: usize,
    checked_surface_count: usize,
) -> Result<()> {
    if active_row_count != goal.expected_active_row_count
        || complete_row_count != goal.expected_active_row_count
        || checked_surface_count != goal.expected_checked_surface_count
    {
        bail!(
            "stage completion counts for `{}` drifted: active_rows={}, complete_rows={}, checked_surfaces={}, expected_active_rows={}, expected_checked_surfaces={}",
            goal.stage_id,
            active_row_count,
            complete_row_count,
            checked_surface_count,
            goal.expected_active_row_count,
            goal.expected_checked_surface_count
        );
    }
    Ok(())
}

fn validate_checked_in_stage_completion_artifact(
    repo_root: &Path,
    goal: BamStageCompletionGoal,
) -> Result<String> {
    let absolute_path = repo_relative_path(repo_root, Path::new(goal.output_path));
    let payload = std::fs::read_to_string(&absolute_path)
        .with_context(|| format!("read {}", absolute_path.display()))?;
    let report: StageCompletionFallbackReport = serde_json::from_str(&payload)
        .with_context(|| format!("parse {}", absolute_path.display()))?;
    if report.output_path != goal.output_path {
        bail!(
            "checked-in stage completion artifact for `{}` drifted output_path: observed=`{}` expected=`{}`",
            goal.stage_id,
            report.output_path,
            goal.output_path
        );
    }
    if !report.ok || report.incomplete_row_count != 0 || report.violation_count != 0 {
        bail!(
            "checked-in stage completion artifact for `{}` is not complete: ok={}, incomplete_rows={}, violations={}",
            goal.stage_id,
            report.ok,
            report.incomplete_row_count,
            report.violation_count
        );
    }
    validate_stage_completion_counts(
        goal,
        report.complete_row_count,
        report.active_row_count,
        report.checked_surface_count,
    )?;
    Ok(format!(
        "validated checked-in `{}` completion artifact across {}/{} retained row(s) and {} governed surface(s) because the local binary was built without `bam_downstream`",
        goal.stage_id,
        report.complete_row_count,
        report.active_row_count,
        report.checked_surface_count
    ))
}

fn format_stage_completion_detail(
    stage_id: &str,
    complete_row_count: usize,
    active_row_count: usize,
    checked_surface_count: usize,
) -> String {
    format!(
        "validated `{stage_id}` completion across {complete_row_count}/{active_row_count} retained row(s) and {checked_surface_count} governed surface(s)"
    )
}

fn record_goal_check<F>(
    checks: &mut Vec<BamAllRetainedToolsCompleteGoalCheck>,
    goal_id: u32,
    surface: impl Into<String>,
    output_path: Option<String>,
    check: F,
) where
    F: FnOnce() -> Result<String>,
{
    match check() {
        Ok(detail) => checks.push(BamAllRetainedToolsCompleteGoalCheck {
            goal_id,
            surface: surface.into(),
            output_path,
            ok: true,
            detail,
        }),
        Err(error) => checks.push(BamAllRetainedToolsCompleteGoalCheck {
            goal_id,
            surface: surface.into(),
            output_path,
            ok: false,
            detail: error.to_string(),
        }),
    }
}

fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

struct CurrentDirGuard {
    original_dir: PathBuf,
}

impl CurrentDirGuard {
    fn change_to(path: &Path) -> Self {
        let original_dir = env::current_dir().expect("capture current dir");
        env::set_current_dir(path).expect("set current dir");
        Self { original_dir }
    }
}

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        env::set_current_dir(&self.original_dir).expect("restore current dir");
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_bam_all_retained_tools_complete, BamAllRetainedToolsCompleteReport,
        BAM_ALL_RETAINED_TOOLS_COMPLETE_SCHEMA_VERSION,
        DEFAULT_BAM_ALL_RETAINED_TOOLS_COMPLETE_PATH,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn render_bam_all_retained_tools_complete_reports_governed_pass_state() {
        let report = render_bam_all_retained_tools_complete(
            &repo_root(),
            PathBuf::from(DEFAULT_BAM_ALL_RETAINED_TOOLS_COMPLETE_PATH),
        )
        .expect("render BAM retained-tools completion report");

        assert_governed_pass_state(&report);
        assert!(
            report
                .checks
                .iter()
                .find(|check| check.goal_id == 439)
                .expect("goal 439 check")
                .detail
                .contains("bam.genotyping"),
            "goal 439 detail must keep BAM genotyping explicit"
        );
    }

    fn assert_governed_pass_state(report: &BamAllRetainedToolsCompleteReport) {
        assert_eq!(report.schema_version, BAM_ALL_RETAINED_TOOLS_COMPLETE_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_BAM_ALL_RETAINED_TOOLS_COMPLETE_PATH);
        assert_eq!(report.checked_goal_count, 19);
        assert_eq!(report.passed_goal_count, 19);
        assert_eq!(report.failed_goal_count, 0);
        assert!(report.failing_goal_ids.is_empty());
        assert_eq!(report.retained_row_count, 49);
        assert_eq!(report.retained_stage_count, 24);
        assert_eq!(report.retained_tool_count, 25);
        assert_eq!(report.command_adapter_row_count, 49);
        assert_eq!(report.output_declaration_row_count, 49);
        assert_eq!(report.expected_result_row_count, 49);
        assert_eq!(report.rendered_command_row_count, 49);
        assert_eq!(report.parser_fixture_row_count, 49);
        assert_eq!(report.local_smoke_row_count, 49);
        let expected_host_stage_smoke_row_count = if cfg!(feature = "bam_downstream") {
            20
        } else {
            18
        };
        let expected_container_smoke_row_count = if cfg!(feature = "bam_downstream") {
            29
        } else {
            31
        };
        assert_eq!(report.local_smoke_host_stage_row_count, expected_host_stage_smoke_row_count);
        assert_eq!(report.local_smoke_container_row_count, expected_container_smoke_row_count);
        assert_eq!(report.report_map_row_count, 49);
        assert_eq!(report.active_row_consistency_surface_count, 6);
        assert_eq!(report.micro_smoke_family_count, 12);
        assert_eq!(report.science_threshold_stage_count, 15);
        assert!(report.ok);
        assert_eq!(report.checks.len(), 19);
        assert!(report.checks.iter().all(|check| check.ok));
    }
}
