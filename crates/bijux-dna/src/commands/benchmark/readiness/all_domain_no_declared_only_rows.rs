use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::all_domain_active_stage_tool_matrix::{
    collect_all_domain_adapter_status_by_binding,
    collect_all_domain_executable_active_stage_tool_matrix_rows,
    collect_all_domain_lifecycle_active_stage_tool_matrix_rows, is_executable_adapter_status,
    AllDomainActiveStageToolMatrixRow,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ALL_DOMAIN_NO_DECLARED_ONLY_ROWS_PATH: &str =
    "benchmarks/readiness/all-domains/no-declared-only-rows.json";
const ALL_DOMAIN_NO_DECLARED_ONLY_ROWS_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_no_declared_only_rows.v1";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AllDomainNoDeclaredOnlyRow {
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) adapter_id: String,
    pub(crate) parser_id: String,
    pub(crate) schema_id: String,
    pub(crate) status: String,
    pub(crate) adapter_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AllDomainNoDeclaredOnlyRowsReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) lifecycle_active_row_count: usize,
    pub(crate) lifecycle_active_stage_count: usize,
    pub(crate) lifecycle_active_tool_count: usize,
    pub(crate) active_row_count: usize,
    pub(crate) active_stage_count: usize,
    pub(crate) active_tool_count: usize,
    pub(crate) removed_row_count: usize,
    pub(crate) removed_stage_count: usize,
    pub(crate) removed_tool_count: usize,
    pub(crate) removed_adapter_status_counts: BTreeMap<String, usize>,
    pub(crate) violation_count: usize,
    pub(crate) ok: bool,
    pub(crate) removed_rows: Vec<AllDomainNoDeclaredOnlyRow>,
    pub(crate) violations: Vec<AllDomainNoDeclaredOnlyRow>,
}

pub(crate) fn run_render_all_domain_no_declared_only_rows(
    args: &parse::BenchReadinessRenderAllDomainNoDeclaredOnlyRowsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_all_domain_no_declared_only_rows(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ALL_DOMAIN_NO_DECLARED_ONLY_ROWS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_all_domain_no_declared_only_rows(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AllDomainNoDeclaredOnlyRowsReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let report = build_all_domain_no_declared_only_rows_report(repo_root, &output_path)?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let payload =
        serde_json::to_vec_pretty(&report).context("serialize no-declared-only-rows report")?;
    fs::write(&output_path, payload).with_context(|| format!("write {}", output_path.display()))?;
    if !report.ok {
        return Err(anyhow!(
            "all-domain active scope still contains declaration-only rows without executable adapters"
        ));
    }
    Ok(report)
}

fn build_all_domain_no_declared_only_rows_report(
    repo_root: &Path,
    output_path: &Path,
) -> Result<AllDomainNoDeclaredOnlyRowsReport> {
    let lifecycle_active_rows =
        collect_all_domain_lifecycle_active_stage_tool_matrix_rows(repo_root)?;
    let active_rows = collect_all_domain_executable_active_stage_tool_matrix_rows(repo_root)?;
    let adapter_status_by_binding = collect_all_domain_adapter_status_by_binding(repo_root)?;

    let lifecycle_active_rows = lifecycle_active_rows
        .iter()
        .map(|row| attach_adapter_status(row, &adapter_status_by_binding))
        .collect::<Result<Vec<_>>>()?;
    let active_rows = active_rows
        .iter()
        .map(|row| attach_adapter_status(row, &adapter_status_by_binding))
        .collect::<Result<Vec<_>>>()?;

    let removed_rows = lifecycle_active_rows
        .iter()
        .filter(|row| !is_executable_adapter_status(&row.adapter_status))
        .cloned()
        .collect::<Vec<_>>();
    let violations = active_rows
        .iter()
        .filter(|row| !is_executable_adapter_status(&row.adapter_status))
        .cloned()
        .collect::<Vec<_>>();

    let mut removed_adapter_status_counts = BTreeMap::<String, usize>::new();
    for row in &removed_rows {
        *removed_adapter_status_counts.entry(row.adapter_status.clone()).or_default() += 1;
    }

    let report = AllDomainNoDeclaredOnlyRowsReport {
        schema_version: ALL_DOMAIN_NO_DECLARED_ONLY_ROWS_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        lifecycle_active_row_count: lifecycle_active_rows.len(),
        lifecycle_active_stage_count: count_stage_keys(&lifecycle_active_rows),
        lifecycle_active_tool_count: count_tool_keys(&lifecycle_active_rows),
        active_row_count: active_rows.len(),
        active_stage_count: count_stage_keys(&active_rows),
        active_tool_count: count_tool_keys(&active_rows),
        removed_row_count: removed_rows.len(),
        removed_stage_count: count_stage_keys(&removed_rows),
        removed_tool_count: count_tool_keys(&removed_rows),
        removed_adapter_status_counts,
        violation_count: violations.len(),
        ok: violations.is_empty(),
        removed_rows,
        violations,
    };
    ensure_all_domain_no_declared_only_rows_contract(&report)?;
    Ok(report)
}

fn ensure_all_domain_no_declared_only_rows_contract(
    report: &AllDomainNoDeclaredOnlyRowsReport,
) -> Result<()> {
    if report.removed_row_count != report.removed_rows.len() {
        return Err(anyhow!(
            "all-domain no-declared-only-rows report drifted from its removed rows"
        ));
    }
    if report.violation_count != report.violations.len() {
        return Err(anyhow!(
            "all-domain no-declared-only-rows report drifted from its violation rows"
        ));
    }
    if report.removed_rows.iter().any(|row| is_executable_adapter_status(&row.adapter_status)) {
        return Err(anyhow!(
            "all-domain no-declared-only-rows report kept an executable row in the removed set"
        ));
    }
    if report.violations.iter().any(|row| is_executable_adapter_status(&row.adapter_status)) {
        return Err(anyhow!(
            "all-domain no-declared-only-rows report kept an executable row in the violation set"
        ));
    }
    if report.ok && report.violation_count != 0 {
        return Err(anyhow!(
            "all-domain no-declared-only-rows report cannot be ok while declaration-only violations remain"
        ));
    }
    if !report.ok && report.violation_count == 0 {
        return Err(anyhow!(
            "all-domain no-declared-only-rows report must keep explicit violations when failing"
        ));
    }
    Ok(())
}

fn attach_adapter_status(
    row: &AllDomainActiveStageToolMatrixRow,
    adapter_status_by_binding: &BTreeMap<(String, String, String), String>,
) -> Result<AllDomainNoDeclaredOnlyRow> {
    let adapter_status = adapter_status_by_binding
        .get(&(row.domain.clone(), row.stage_id.clone(), row.tool_id.clone()))
        .cloned()
        .ok_or_else(|| {
            anyhow!(
                "all-domain no-declared-only-rows report is missing adapter status coverage for `{}` / `{}` / `{}`",
                row.domain,
                row.stage_id,
                row.tool_id
            )
        })?;
    Ok(AllDomainNoDeclaredOnlyRow {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
        adapter_id: row.adapter_id.clone(),
        parser_id: row.parser_id.clone(),
        schema_id: row.schema_id.clone(),
        status: row.status.clone(),
        adapter_status,
    })
}

fn count_stage_keys(rows: &[AllDomainNoDeclaredOnlyRow]) -> usize {
    rows.iter()
        .map(|row| (row.domain.as_str(), row.stage_id.as_str()))
        .collect::<BTreeSet<_>>()
        .len()
}

fn count_tool_keys(rows: &[AllDomainNoDeclaredOnlyRow]) -> usize {
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
