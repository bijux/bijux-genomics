use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::all_domain_active_stage_tool_matrix::{
    collect_all_domain_active_stage_tool_matrix_rows,
    collect_all_domain_executable_active_stage_tool_matrix_rows, AllDomainActiveStageToolMatrixRow,
    STATUS_NOT_BENCHMARK_READY,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ALL_DOMAIN_NO_NOT_BENCHMARK_READY_ROWS_PATH: &str =
    "benchmarks/readiness/all-domains/no-not-benchmark-ready-rows.json";
const ALL_DOMAIN_NO_NOT_BENCHMARK_READY_ROWS_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_no_not_benchmark_ready_rows.v1";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AllDomainNoNotBenchmarkReadyRowsReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) executable_active_row_count: usize,
    pub(crate) executable_active_stage_count: usize,
    pub(crate) executable_active_tool_count: usize,
    pub(crate) active_row_count: usize,
    pub(crate) active_stage_count: usize,
    pub(crate) active_tool_count: usize,
    pub(crate) removed_row_count: usize,
    pub(crate) removed_stage_count: usize,
    pub(crate) removed_tool_count: usize,
    pub(crate) removed_status_counts: BTreeMap<String, usize>,
    pub(crate) violation_count: usize,
    pub(crate) ok: bool,
    pub(crate) removed_rows: Vec<AllDomainActiveStageToolMatrixRow>,
    pub(crate) violations: Vec<AllDomainActiveStageToolMatrixRow>,
}

pub(crate) fn run_render_all_domain_no_not_benchmark_ready_rows(
    args: &parse::BenchReadinessRenderAllDomainNoNotBenchmarkReadyRowsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_all_domain_no_not_benchmark_ready_rows(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ALL_DOMAIN_NO_NOT_BENCHMARK_READY_ROWS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_all_domain_no_not_benchmark_ready_rows(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AllDomainNoNotBenchmarkReadyRowsReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let report = build_all_domain_no_not_benchmark_ready_rows_report(repo_root, &output_path)?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let payload = serde_json::to_vec_pretty(&report)
        .context("serialize no-not-benchmark-ready-rows report")?;
    fs::write(&output_path, payload).with_context(|| format!("write {}", output_path.display()))?;
    if !report.ok {
        return Err(anyhow!("all-domain active scope still contains not_benchmark_ready rows"));
    }
    Ok(report)
}

fn build_all_domain_no_not_benchmark_ready_rows_report(
    repo_root: &Path,
    output_path: &Path,
) -> Result<AllDomainNoNotBenchmarkReadyRowsReport> {
    let executable_active_rows =
        collect_all_domain_executable_active_stage_tool_matrix_rows(repo_root)?;
    let active_rows = collect_all_domain_active_stage_tool_matrix_rows(repo_root)?;

    let removed_rows = executable_active_rows
        .iter()
        .filter(|row| row.status == STATUS_NOT_BENCHMARK_READY)
        .cloned()
        .collect::<Vec<_>>();
    let violations = active_rows
        .iter()
        .filter(|row| row.status == STATUS_NOT_BENCHMARK_READY)
        .cloned()
        .collect::<Vec<_>>();
    let mut removed_status_counts = BTreeMap::<String, usize>::new();
    for row in &removed_rows {
        *removed_status_counts.entry(row.status.clone()).or_default() += 1;
    }

    let report = AllDomainNoNotBenchmarkReadyRowsReport {
        schema_version: ALL_DOMAIN_NO_NOT_BENCHMARK_READY_ROWS_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        executable_active_row_count: executable_active_rows.len(),
        executable_active_stage_count: count_stage_keys(&executable_active_rows),
        executable_active_tool_count: count_tool_keys(&executable_active_rows),
        active_row_count: active_rows.len(),
        active_stage_count: count_stage_keys(&active_rows),
        active_tool_count: count_tool_keys(&active_rows),
        removed_row_count: removed_rows.len(),
        removed_stage_count: count_stage_keys(&removed_rows),
        removed_tool_count: count_tool_keys(&removed_rows),
        removed_status_counts,
        violation_count: violations.len(),
        ok: violations.is_empty(),
        removed_rows,
        violations,
    };
    ensure_all_domain_no_not_benchmark_ready_rows_contract(&report)?;
    Ok(report)
}

fn ensure_all_domain_no_not_benchmark_ready_rows_contract(
    report: &AllDomainNoNotBenchmarkReadyRowsReport,
) -> Result<()> {
    if report.removed_row_count != report.removed_rows.len() {
        return Err(anyhow!(
            "all-domain no-not-benchmark-ready-rows report drifted from its removed rows"
        ));
    }
    if report.violation_count != report.violations.len() {
        return Err(anyhow!(
            "all-domain no-not-benchmark-ready-rows report drifted from its violation rows"
        ));
    }
    if report.removed_rows.iter().any(|row| row.status != STATUS_NOT_BENCHMARK_READY) {
        return Err(anyhow!(
            "all-domain no-not-benchmark-ready-rows report kept a removed row outside the not_benchmark_ready status set"
        ));
    }
    if report.violations.iter().any(|row| row.status != STATUS_NOT_BENCHMARK_READY) {
        return Err(anyhow!(
            "all-domain no-not-benchmark-ready-rows report kept a violation outside the not_benchmark_ready status set"
        ));
    }
    if report.ok && report.violation_count != 0 {
        return Err(anyhow!(
            "all-domain no-not-benchmark-ready-rows report cannot be ok while active-scope violations remain"
        ));
    }
    if !report.ok && report.violation_count == 0 {
        return Err(anyhow!(
            "all-domain no-not-benchmark-ready-rows report must keep explicit violations when failing"
        ));
    }
    Ok(())
}

fn count_stage_keys(rows: &[AllDomainActiveStageToolMatrixRow]) -> usize {
    rows.iter()
        .map(|row| (row.domain.as_str(), row.stage_id.as_str()))
        .collect::<BTreeSet<_>>()
        .len()
}

fn count_tool_keys(rows: &[AllDomainActiveStageToolMatrixRow]) -> usize {
    rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len()
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
