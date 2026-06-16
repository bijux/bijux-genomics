use std::collections::BTreeSet;
use std::env;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;

use super::bam_command_adapter_coverage;
use super::bam_local_container_smoke;
use super::bam_parser_fixture_coverage;
use super::bam_rendered_commands;
use super::bam_report_map;
use super::expected_benchmark_results;
use super::tool_serving_map;
use crate::commands::benchmark::readiness::bam_command_adapter_coverage::BamBenchmarkStatus;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_BAM_ALL_RETAINED_TOOLS_COMPLETE_PATH: &str =
    "benchmarks/readiness/bam/BAM_ALL_RETAINED_TOOLS_COMPLETE.json";
const BAM_ALL_RETAINED_TOOLS_COMPLETE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_all_retained_tools_complete.v1";
const EXPECTED_CHECKED_GOAL_COUNT: usize = 17;
const EXPECTED_RETAINED_ROW_COUNT: usize = 49;
const EXPECTED_RETAINED_STAGE_COUNT: usize = 24;
const EXPECTED_RETAINED_TOOL_COUNT: usize = 25;
const EXPECTED_HOST_STAGE_SMOKE_ROW_COUNT: usize = 18;
const EXPECTED_CONTAINER_SMOKE_ROW_COUNT: usize = 31;

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
    pub(crate) expected_result_row_count: usize,
    pub(crate) rendered_command_row_count: usize,
    pub(crate) parser_fixture_row_count: usize,
    pub(crate) local_smoke_row_count: usize,
    pub(crate) local_smoke_host_stage_row_count: usize,
    pub(crate) local_smoke_container_row_count: usize,
    pub(crate) report_map_row_count: usize,
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

    record_goal_check(
        &mut checks,
        379,
        "bam retained binding matrix",
        Some(tool_serving_map::DEFAULT_BAM_TOOL_SERVING_MAP_PATH.to_string()),
        || {
            if tool_map.row_count != EXPECTED_RETAINED_ROW_COUNT
                || tool_map.stage_count != EXPECTED_RETAINED_STAGE_COUNT
                || tool_map.tool_count != EXPECTED_RETAINED_TOOL_COUNT
            {
                bail!(
                    "BAM retained matrix drifted: rows={}, stages={}, tools={}",
                    tool_map.row_count,
                    tool_map.stage_count,
                    tool_map.tool_count
                );
            }
            let unsupported_rows = tool_map
                .rows
                .iter()
                .filter(|row| row.support_status != "supported")
                .map(|row| format!("{}/{} ({})", row.stage_id, row.tool_id, row.support_status))
                .collect::<Vec<_>>();
            if !unsupported_rows.is_empty() {
                bail!(
                    "BAM retained matrix still carries non-retained support rows: {}",
                    unsupported_rows.join(", ")
                );
            }
            let adapter_gaps = tool_map
                .rows
                .iter()
                .filter(|row| !matches!(row.adapter_status.as_str(), "runnable" | "plannable"))
                .map(|row| format!("{}/{} ({})", row.stage_id, row.tool_id, row.adapter_status))
                .collect::<Vec<_>>();
            if !adapter_gaps.is_empty() {
                bail!(
                    "BAM retained matrix still carries declared-only adapter rows: {}",
                    adapter_gaps.join(", ")
                );
            }
            if retained_bindings != command_bindings {
                bail!(
                    "BAM retained matrix drifted from command coverage: missing={:?} extra={:?}",
                    diff_bindings(&retained_bindings, &command_bindings),
                    diff_bindings(&command_bindings, &retained_bindings)
                );
            }
            Ok(
                "validated the governed 49-row BAM retained matrix with only supported, adapter-backed retained bindings"
                    .to_string(),
            )
        },
    );

    for (goal_id, surface, stage_ids) in [
        (380, "bam.align", vec!["bam.align"]),
        (
            381,
            "bam validation and core qc",
            vec!["bam.validate", "bam.qc_pre", "bam.mapping_summary"],
        ),
        (382, "bam filtering", vec!["bam.filter", "bam.mapq_filter", "bam.length_filter"]),
        (383, "bam duplicate handling", vec!["bam.markdup", "bam.duplication_metrics"]),
        (384, "bam complexity", vec!["bam.complexity"]),
        (385, "bam coverage", vec!["bam.coverage"]),
        (386, "bam insert-size and gc-bias", vec!["bam.insert_size", "bam.gc_bias"]),
        (
            387,
            "bam overlap and endogenous-content",
            vec!["bam.overlap_correction", "bam.endogenous_content"],
        ),
        (
            388,
            "bam damage and authenticity",
            vec!["bam.bias_mitigation", "bam.damage", "bam.authenticity"],
        ),
        (
            389,
            "bam contamination sex haplogroups",
            vec!["bam.contamination", "bam.sex", "bam.haplogroups"],
        ),
        (390, "bam recalibration and genotyping", vec!["bam.recalibration", "bam.genotyping"]),
        (391, "bam kinship", vec!["bam.kinship"]),
    ] {
        record_goal_check(
            &mut checks,
            goal_id,
            surface.to_string(),
            Some(
                bam_command_adapter_coverage::DEFAULT_BAM_COMMAND_ADAPTER_COVERAGE_PATH.to_string(),
            ),
            || {
                let expected = filter_bindings_by_stage_ids(&retained_bindings, &stage_ids);
                if expected.is_empty() {
                    bail!("no retained BAM bindings were found for stage slice `{surface}`");
                }
                ensure_binding_subset(surface, "command coverage", &expected, &command_bindings)?;
                ensure_binding_subset(
                    surface,
                    "parser fixture coverage",
                    &expected,
                    &parser_bindings,
                )?;
                ensure_binding_subset(
                    surface,
                    "local/container smoke coverage",
                    &expected,
                    &local_smoke_bindings,
                )?;
                ensure_binding_subset(
                    surface,
                    "rendered command coverage",
                    &expected,
                    &rendered_command_bindings,
                )?;
                ensure_binding_subset(
                    surface,
                    "expected benchmark results",
                    &expected,
                    &expected_result_bindings,
                )?;
                ensure_binding_subset(
                    surface,
                    "report map coverage",
                    &expected,
                    &report_map_bindings,
                )?;
                Ok(format!(
                    "validated `{surface}` across {} retained BAM binding(s) with command, parser, smoke, expected-result, rendered-command, and report-map coverage",
                    expected.len()
                ))
            },
        );
    }

    record_goal_check(
        &mut checks,
        392,
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
            Ok("validated governed host-vs-container smoke coverage for every retained BAM binding"
                .to_string())
        },
    );

    record_goal_check(
        &mut checks,
        393,
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
        394,
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
        395,
        "bam report map",
        Some(bam_report_map::DEFAULT_BAM_REPORT_MAP_PATH.to_string()),
        || {
            if report_map.expected_result_row_count != EXPECTED_RETAINED_ROW_COUNT
                || report_map.row_count != EXPECTED_RETAINED_ROW_COUNT
                || report_map.stage_count != EXPECTED_RETAINED_STAGE_COUNT
                || report_map.tool_count != EXPECTED_RETAINED_TOOL_COUNT
            {
                bail!(
                    "BAM expected-result/report-map counts drifted: expected_results={}, report_map={}, stages={}, tools={}",
                    report_map.expected_result_row_count,
                    report_map.row_count,
                    report_map.stage_count,
                    report_map.tool_count
                );
            }
            ensure_binding_subset(
                "bam retained expected results",
                "expected benchmark results",
                &retained_bindings,
                &expected_result_bindings,
            )?;
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
        expected_result_row_count: expected_result_bindings.len(),
        rendered_command_row_count: rendered_commands.row_count,
        parser_fixture_row_count: parser_coverage.row_count,
        local_smoke_row_count: local_smoke.row_count,
        local_smoke_host_stage_row_count: local_smoke.host_stage_smoke_row_count,
        local_smoke_container_row_count: local_smoke.container_smoke_row_count,
        report_map_row_count: report_map.row_count,
        ok: failed_goal_count == 0
            && checks.len() == EXPECTED_CHECKED_GOAL_COUNT
            && tool_map.row_count == EXPECTED_RETAINED_ROW_COUNT
            && command_coverage.row_count == EXPECTED_RETAINED_ROW_COUNT
            && expected_result_bindings.len() == EXPECTED_RETAINED_ROW_COUNT
            && rendered_commands.row_count == EXPECTED_RETAINED_ROW_COUNT
            && parser_coverage.row_count == EXPECTED_RETAINED_ROW_COUNT
            && local_smoke.row_count == EXPECTED_RETAINED_ROW_COUNT
            && report_map.row_count == EXPECTED_RETAINED_ROW_COUNT,
        checks,
    };
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)?;
    if !report.ok {
        return Err(anyhow!(
            "every governed BAM retained-tool gate from goals 379 through 395 must remain complete"
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
                .find(|check| check.goal_id == 390)
                .expect("goal 390 check")
                .detail
                .contains("recalibration and genotyping"),
            "goal 390 detail must keep the BAM recalibration/genotyping slice explicit"
        );
    }

    fn assert_governed_pass_state(report: &BamAllRetainedToolsCompleteReport) {
        assert_eq!(report.schema_version, BAM_ALL_RETAINED_TOOLS_COMPLETE_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_BAM_ALL_RETAINED_TOOLS_COMPLETE_PATH);
        assert_eq!(report.checked_goal_count, 17);
        assert_eq!(report.passed_goal_count, 17);
        assert_eq!(report.failed_goal_count, 0);
        assert!(report.failing_goal_ids.is_empty());
        assert_eq!(report.retained_row_count, 49);
        assert_eq!(report.retained_stage_count, 24);
        assert_eq!(report.retained_tool_count, 25);
        assert_eq!(report.command_adapter_row_count, 49);
        assert_eq!(report.expected_result_row_count, 49);
        assert_eq!(report.rendered_command_row_count, 49);
        assert_eq!(report.parser_fixture_row_count, 49);
        assert_eq!(report.local_smoke_row_count, 49);
        assert_eq!(report.local_smoke_host_stage_row_count, 18);
        assert_eq!(report.local_smoke_container_row_count, 31);
        assert_eq!(report.report_map_row_count, 49);
        assert!(report.ok);
        assert_eq!(report.checks.len(), 17);
        assert!(report.checks.iter().all(|check| check.ok));
    }
}
