use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use serde_json::Value;

use super::all_domain_active_stage_tool_matrix::{
    collect_all_domain_active_stage_tool_matrix_candidate_rows,
    collect_all_domain_active_stage_tool_matrix_rows, collect_all_domain_adapter_status_by_binding,
    AllDomainActiveStageToolMatrixRow, ADAPTER_STATUS_PLANNABLE, ADAPTER_STATUS_RUNNABLE,
    STATUS_FUTURE, STATUS_NOT_BENCHMARK_READY, STATUS_PLANNED,
};
use super::all_domain_expected_benchmark_results::collect_all_domain_expected_benchmark_result_rows;
use super::all_domain_rendered_commands::collect_all_domain_rendered_command_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_REMOVED_FROM_SCOPE_PATH: &str =
    "benchmarks/readiness/removed-from-scope.tsv";
const REMOVED_FROM_SCOPE_SCHEMA_VERSION: &str = "bijux.bench.readiness.removed_from_scope.v1";
const TRACKED_FULL_BENCHMARK_REPORT_JSON_PATH: &str =
    "benchmarks/readiness/FASTQ_BAM_VCF_BENCHMARK_REPORT.json";
const REPORT_SURFACE_SOURCE_TRACKED_JSON: &str = "tracked_report_json";
const REPORT_SURFACE_SOURCE_EXPECTED_RESULT_CONTRACT: &str = "expected_result_contract";
pub(crate) const SCOPE_EXIT_KIND_LIFECYCLE_NOT_ACTIVE: &str = "lifecycle_not_active";
pub(crate) const SCOPE_EXIT_KIND_BENCHMARK_NOT_READY: &str = "benchmark_not_ready";
pub(crate) const SCOPE_EXIT_KIND_NON_EXECUTABLE_ADAPTER: &str = "non_executable_adapter";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct BindingKey {
    domain: String,
    stage_id: String,
    tool_id: String,
    corpus_id: String,
    asset_profile_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct RemovedFromScopeRow {
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
    pub(crate) scope_exit_kind: String,
    pub(crate) stage_removed_from_active_scope: bool,
    pub(crate) tool_removed_from_active_scope: bool,
    pub(crate) absent_from_active_matrix: bool,
    pub(crate) absent_from_rendered_commands: bool,
    pub(crate) absent_from_expected_results: bool,
    pub(crate) absent_from_full_benchmark_report: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RemovedFromScopeReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) full_benchmark_report_check_source: String,
    pub(crate) candidate_row_count: usize,
    pub(crate) candidate_stage_count: usize,
    pub(crate) candidate_tool_count: usize,
    pub(crate) active_row_count: usize,
    pub(crate) active_stage_count: usize,
    pub(crate) active_tool_count: usize,
    pub(crate) removed_row_count: usize,
    pub(crate) removed_stage_count: usize,
    pub(crate) removed_tool_count: usize,
    pub(crate) fully_removed_stage_count: usize,
    pub(crate) fully_removed_tool_count: usize,
    pub(crate) scope_exit_kind_counts: BTreeMap<String, usize>,
    pub(crate) violation_count: usize,
    pub(crate) ok: bool,
    pub(crate) rows: Vec<RemovedFromScopeRow>,
    pub(crate) violations: Vec<RemovedFromScopeRow>,
}

pub(crate) fn run_render_removed_from_scope(
    args: &parse::BenchReadinessRenderRemovedFromScopeArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_removed_from_scope(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_REMOVED_FROM_SCOPE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_removed_from_scope(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<RemovedFromScopeReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let report = build_removed_from_scope_report(repo_root, &output_path)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_removed_from_scope_tsv(&report.rows))
        .with_context(|| format!("write {}", output_path.display()))?;
    if !report.ok {
        return Err(anyhow!("removed-from-scope rows leaked into governed active surfaces"));
    }
    Ok(report)
}

pub(crate) fn build_removed_from_scope_report(
    repo_root: &Path,
    output_path: &Path,
) -> Result<RemovedFromScopeReport> {
    let candidate_rows = collect_all_domain_active_stage_tool_matrix_candidate_rows(repo_root)?;
    let active_rows = collect_all_domain_active_stage_tool_matrix_rows(repo_root)?;
    let adapter_status_by_binding = collect_all_domain_adapter_status_by_binding(repo_root)?;

    let active_keys = active_rows.iter().map(binding_key_from_active_row).collect::<BTreeSet<_>>();
    let active_stage_keys = active_rows
        .iter()
        .map(|row| (row.domain.as_str(), row.stage_id.as_str()))
        .collect::<BTreeSet<_>>();
    let active_tool_ids =
        active_rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>();

    let rendered_command_keys = collect_all_domain_rendered_command_rows(repo_root)?
        .iter()
        .map(binding_key_from_command_row)
        .collect::<BTreeSet<_>>();
    let expected_result_keys = collect_all_domain_expected_benchmark_result_rows(repo_root)?
        .iter()
        .map(binding_key_from_expected_row)
        .collect::<BTreeSet<_>>();
    let (full_benchmark_report_check_source, full_report_keys) =
        collect_full_benchmark_report_keys(repo_root, &expected_result_keys)?;

    let mut rows = Vec::new();
    for row in
        candidate_rows.iter().filter(|row| !active_keys.contains(&binding_key_from_active_row(row)))
    {
        let adapter_status = adapter_status_by_binding
            .get(&(row.domain.clone(), row.stage_id.clone(), row.tool_id.clone()))
            .cloned()
            .ok_or_else(|| {
                anyhow!(
                    "removed-from-scope table is missing adapter status coverage for `{}` / `{}` / `{}`",
                    row.domain,
                    row.stage_id,
                    row.tool_id
                )
            })?;
        let key = binding_key_from_active_row(row);
        let scope_exit_kind = classify_scope_exit_kind(row, &adapter_status)?;
        rows.push(RemovedFromScopeRow {
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
            stage_removed_from_active_scope: !active_stage_keys
                .contains(&(row.domain.as_str(), row.stage_id.as_str())),
            tool_removed_from_active_scope: !active_tool_ids.contains(row.tool_id.as_str()),
            absent_from_active_matrix: !active_keys.contains(&key),
            absent_from_rendered_commands: !rendered_command_keys.contains(&key),
            absent_from_expected_results: !expected_result_keys.contains(&key),
            absent_from_full_benchmark_report: !full_report_keys.contains(&key),
            scope_exit_kind,
        });
    }

    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.corpus_id.cmp(&right.corpus_id))
            .then_with(|| left.asset_profile_id.cmp(&right.asset_profile_id))
    });

    let mut scope_exit_kind_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *scope_exit_kind_counts.entry(row.scope_exit_kind.clone()).or_default() += 1;
    }
    let violations = rows
        .iter()
        .filter(|row| {
            !row.absent_from_active_matrix
                || !row.absent_from_rendered_commands
                || !row.absent_from_expected_results
                || !row.absent_from_full_benchmark_report
        })
        .cloned()
        .collect::<Vec<_>>();

    let report = RemovedFromScopeReport {
        schema_version: REMOVED_FROM_SCOPE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        full_benchmark_report_check_source,
        candidate_row_count: candidate_rows.len(),
        candidate_stage_count: count_stage_keys_from_active_rows(&candidate_rows),
        candidate_tool_count: count_tool_keys_from_active_rows(&candidate_rows),
        active_row_count: active_rows.len(),
        active_stage_count: count_stage_keys_from_active_rows(&active_rows),
        active_tool_count: count_tool_keys_from_active_rows(&active_rows),
        removed_row_count: rows.len(),
        removed_stage_count: rows
            .iter()
            .map(|row| (row.domain.as_str(), row.stage_id.as_str()))
            .collect::<BTreeSet<_>>()
            .len(),
        removed_tool_count: rows
            .iter()
            .map(|row| row.tool_id.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        fully_removed_stage_count: rows
            .iter()
            .filter(|row| row.stage_removed_from_active_scope)
            .map(|row| (row.domain.as_str(), row.stage_id.as_str()))
            .collect::<BTreeSet<_>>()
            .len(),
        fully_removed_tool_count: rows
            .iter()
            .filter(|row| row.tool_removed_from_active_scope)
            .map(|row| row.tool_id.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        scope_exit_kind_counts,
        violation_count: violations.len(),
        ok: violations.is_empty(),
        rows,
        violations,
    };
    ensure_removed_from_scope_contract(&candidate_rows, &active_rows, &report)?;
    Ok(report)
}

fn classify_scope_exit_kind(
    row: &AllDomainActiveStageToolMatrixRow,
    adapter_status: &str,
) -> Result<String> {
    if matches!(row.status.as_str(), STATUS_PLANNED | STATUS_FUTURE) {
        return Ok(SCOPE_EXIT_KIND_LIFECYCLE_NOT_ACTIVE.to_string());
    }
    if !is_executable_adapter_status(adapter_status) {
        return Ok(SCOPE_EXIT_KIND_NON_EXECUTABLE_ADAPTER.to_string());
    }
    if row.status == STATUS_NOT_BENCHMARK_READY {
        return Ok(SCOPE_EXIT_KIND_BENCHMARK_NOT_READY.to_string());
    }
    Err(anyhow!(
        "removed-from-scope row `{}` / `{}` / `{}` is outside active scope without a governed exit kind",
        row.domain,
        row.stage_id,
        row.tool_id
    ))
}

fn ensure_removed_from_scope_contract(
    candidate_rows: &[AllDomainActiveStageToolMatrixRow],
    active_rows: &[AllDomainActiveStageToolMatrixRow],
    report: &RemovedFromScopeReport,
) -> Result<()> {
    if report.removed_row_count != report.rows.len() {
        return Err(anyhow!("removed-from-scope report drifted from its row set"));
    }
    if report.violation_count != report.violations.len() {
        return Err(anyhow!("removed-from-scope report drifted from its violation set"));
    }

    let candidate_keys =
        candidate_rows.iter().map(binding_key_from_active_row).collect::<BTreeSet<_>>();
    let active_keys = active_rows.iter().map(binding_key_from_active_row).collect::<BTreeSet<_>>();
    let expected_removed_keys =
        candidate_keys.difference(&active_keys).cloned().collect::<BTreeSet<_>>();
    let reported_removed_keys =
        report.rows.iter().map(binding_key_from_removed_row).collect::<BTreeSet<_>>();
    if expected_removed_keys != reported_removed_keys {
        return Err(anyhow!(
            "removed-from-scope report must keep exactly the candidate bindings that are outside active scope"
        ));
    }

    for row in &report.rows {
        if row.scope_exit_kind.trim().is_empty()
            || row.status.trim().is_empty()
            || row.adapter_status.trim().is_empty()
        {
            return Err(anyhow!(
                "removed-from-scope row `{}` / `{}` / `{}` is missing a required scope-exit field",
                row.domain,
                row.stage_id,
                row.tool_id
            ));
        }
    }
    if report.ok && report.violation_count != 0 {
        return Err(anyhow!("removed-from-scope report cannot be ok while leaked bindings remain"));
    }
    if !report.ok && report.violation_count == 0 {
        return Err(anyhow!(
            "removed-from-scope report must keep explicit leaked bindings when failing"
        ));
    }
    Ok(())
}

fn collect_full_benchmark_report_keys(
    repo_root: &Path,
    expected_result_keys: &BTreeSet<BindingKey>,
) -> Result<(String, BTreeSet<BindingKey>)> {
    let report_path = repo_root.join(TRACKED_FULL_BENCHMARK_REPORT_JSON_PATH);
    if !report_path.exists() {
        return Ok((
            REPORT_SURFACE_SOURCE_EXPECTED_RESULT_CONTRACT.to_string(),
            expected_result_keys.clone(),
        ));
    }
    let payload =
        fs::read(&report_path).with_context(|| format!("read {}", report_path.display()))?;
    let value: Value = serde_json::from_slice(&payload)
        .with_context(|| format!("parse {}", report_path.display()))?;
    let rows = value
        .get("rows")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("full benchmark report JSON is missing `rows`"))?;
    let mut keys = BTreeSet::new();
    for row in rows {
        let domain = required_str_field(row, "domain")?;
        let stage_id = required_str_field(row, "stage_id")?;
        let tool_id = required_str_field(row, "tool_id")?;
        let corpus_id = required_str_field(row, "corpus_id")?;
        let asset_profile_id = required_str_field(row, "asset_profile_id")?;
        if corpus_id == "not_applicable" && asset_profile_id == "not_applicable" {
            continue;
        }
        keys.insert(BindingKey { domain, stage_id, tool_id, corpus_id, asset_profile_id });
    }
    Ok((REPORT_SURFACE_SOURCE_TRACKED_JSON.to_string(), keys))
}

fn required_str_field(row: &Value, field: &str) -> Result<String> {
    row.get(field)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| anyhow!("full benchmark report JSON row is missing `{field}`"))
}

fn render_removed_from_scope_tsv(rows: &[RemovedFromScopeRow]) -> String {
    let mut rendered = String::from(
        "domain\tstage_id\ttool_id\tcorpus_id\tasset_profile_id\tadapter_id\tparser_id\tschema_id\tstatus\tadapter_status\tscope_exit_kind\tstage_removed_from_active_scope\ttool_removed_from_active_scope\tabsent_from_active_matrix\tabsent_from_rendered_commands\tabsent_from_expected_results\tabsent_from_full_benchmark_report\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.domain),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.corpus_id),
            sanitize_tsv(&row.asset_profile_id),
            sanitize_tsv(&row.adapter_id),
            sanitize_tsv(&row.parser_id),
            sanitize_tsv(&row.schema_id),
            sanitize_tsv(&row.status),
            sanitize_tsv(&row.adapter_status),
            sanitize_tsv(&row.scope_exit_kind),
            row.stage_removed_from_active_scope,
            row.tool_removed_from_active_scope,
            row.absent_from_active_matrix,
            row.absent_from_rendered_commands,
            row.absent_from_expected_results,
            row.absent_from_full_benchmark_report,
        ));
    }
    rendered
}

fn sanitize_tsv(value: &str) -> String {
    value.replace('\t', " ").replace('\n', " ")
}

fn binding_key_from_active_row(row: &AllDomainActiveStageToolMatrixRow) -> BindingKey {
    BindingKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

fn binding_key_from_removed_row(row: &RemovedFromScopeRow) -> BindingKey {
    BindingKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

fn binding_key_from_command_row(
    row: &super::all_domain_rendered_commands::AllDomainRenderedCommandRow,
) -> BindingKey {
    BindingKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

fn binding_key_from_expected_row(
    row: &super::all_domain_expected_benchmark_results::AllDomainExpectedBenchmarkResultRow,
) -> BindingKey {
    BindingKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
    }
}

fn count_stage_keys_from_active_rows(rows: &[AllDomainActiveStageToolMatrixRow]) -> usize {
    rows.iter()
        .map(|row| (row.domain.as_str(), row.stage_id.as_str()))
        .collect::<BTreeSet<_>>()
        .len()
}

fn count_tool_keys_from_active_rows(rows: &[AllDomainActiveStageToolMatrixRow]) -> usize {
    rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len()
}

fn is_executable_adapter_status(status: &str) -> bool {
    matches!(status, ADAPTER_STATUS_RUNNABLE | ADAPTER_STATUS_PLANNABLE)
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
