use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::removed_from_scope::{
    build_removed_from_scope_report, RemovedFromScopeRow, SCOPE_EXIT_KIND_BENCHMARK_NOT_READY,
    SCOPE_EXIT_KIND_LIFECYCLE_NOT_ACTIVE, SCOPE_EXIT_KIND_NON_EXECUTABLE_ADAPTER,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ALL_DOMAIN_ACTIVE_SCOPE_BLOCKERS_PATH: &str =
    "benchmarks/readiness/all-domains/active-scope-blockers.tsv";
const DEFAULT_REMOVED_FROM_SCOPE_SOURCE_PATH: &str = "benchmarks/readiness/removed-from-scope.tsv";
const DEFAULT_NO_PLANNED_ROWS_PATH: &str = "benchmarks/readiness/all-domains/no-planned-rows.json";
const DEFAULT_NO_NOT_BENCHMARK_READY_ROWS_PATH: &str =
    "benchmarks/readiness/all-domains/no-not-benchmark-ready-rows.json";
const DEFAULT_NO_DECLARED_ONLY_ROWS_PATH: &str =
    "benchmarks/readiness/all-domains/no-declared-only-rows.json";
const ALL_DOMAIN_ACTIVE_SCOPE_BLOCKERS_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_active_scope_blockers.v1";
const BLOCKER_TYPE_LIFECYCLE_NOT_ACTIVE: &str = "lifecycle_not_active";
const BLOCKER_TYPE_BENCHMARK_NOT_READY: &str = "benchmark_not_ready";
const BLOCKER_TYPE_NON_EXECUTABLE_ADAPTER: &str = "non_executable_adapter";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AllDomainActiveScopeBlockerRow {
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
    pub(crate) blocker_type: String,
    pub(crate) blocker_path: String,
    pub(crate) reason: String,
    pub(crate) stage_removed_from_active_scope: bool,
    pub(crate) tool_removed_from_active_scope: bool,
    pub(crate) absent_from_active_matrix: bool,
    pub(crate) absent_from_rendered_commands: bool,
    pub(crate) absent_from_expected_results: bool,
    pub(crate) absent_from_full_benchmark_report: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AllDomainActiveScopeBlockersReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) removed_from_scope_path: String,
    pub(crate) row_count: usize,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) blocker_type_counts: BTreeMap<String, usize>,
    pub(crate) blocker_path_counts: BTreeMap<String, usize>,
    pub(crate) violation_count: usize,
    pub(crate) ok: bool,
    pub(crate) rows: Vec<AllDomainActiveScopeBlockerRow>,
    pub(crate) violations: Vec<AllDomainActiveScopeBlockerRow>,
}

pub(crate) fn run_render_all_domain_active_scope_blockers(
    args: &parse::BenchReadinessRenderAllDomainActiveScopeBlockersArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_all_domain_active_scope_blockers(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ALL_DOMAIN_ACTIVE_SCOPE_BLOCKERS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_all_domain_active_scope_blockers(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AllDomainActiveScopeBlockersReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let report = build_all_domain_active_scope_blockers_report(repo_root, &output_path)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_all_domain_active_scope_blockers_tsv(&report.rows))
        .with_context(|| format!("write {}", output_path.display()))?;
    if !report.ok {
        return Err(anyhow!(
            "all-domain active-scope blockers must keep exact blocker rows for every removed binding"
        ));
    }
    Ok(report)
}

fn build_all_domain_active_scope_blockers_report(
    repo_root: &Path,
    output_path: &Path,
) -> Result<AllDomainActiveScopeBlockersReport> {
    let removed_from_scope_path =
        repo_relative_path(repo_root, Path::new(DEFAULT_REMOVED_FROM_SCOPE_SOURCE_PATH));
    let removed_report = build_removed_from_scope_report(repo_root, &removed_from_scope_path)?;

    let mut rows = removed_report.rows.iter().map(render_row).collect::<Result<Vec<_>>>()?;
    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.corpus_id.cmp(&right.corpus_id))
            .then_with(|| left.asset_profile_id.cmp(&right.asset_profile_id))
    });

    let mut blocker_type_counts = BTreeMap::<String, usize>::new();
    let mut blocker_path_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *blocker_type_counts.entry(row.blocker_type.clone()).or_default() += 1;
        *blocker_path_counts.entry(row.blocker_path.clone()).or_default() += 1;
    }
    let violations = rows
        .iter()
        .filter(|row| {
            row.blocker_type.trim().is_empty()
                || row.blocker_path.trim().is_empty()
                || !row.absent_from_active_matrix
                || !row.absent_from_rendered_commands
                || !row.absent_from_expected_results
                || !row.absent_from_full_benchmark_report
        })
        .cloned()
        .collect::<Vec<_>>();

    let report = AllDomainActiveScopeBlockersReport {
        schema_version: ALL_DOMAIN_ACTIVE_SCOPE_BLOCKERS_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        removed_from_scope_path: path_relative_to_repo(repo_root, &removed_from_scope_path),
        row_count: rows.len(),
        stage_count: rows
            .iter()
            .map(|row| (row.domain.as_str(), row.stage_id.as_str()))
            .collect::<BTreeSet<_>>()
            .len(),
        tool_count: rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len(),
        blocker_type_counts,
        blocker_path_counts,
        violation_count: violations.len(),
        ok: violations.is_empty(),
        rows,
        violations,
    };
    ensure_all_domain_active_scope_blockers_contract(&removed_report, &report)?;
    Ok(report)
}

fn render_row(source: &RemovedFromScopeRow) -> Result<AllDomainActiveScopeBlockerRow> {
    let (blocker_type, blocker_path, reason) =
        blocker_contract_for_scope_exit_kind(&source.scope_exit_kind)?;
    Ok(AllDomainActiveScopeBlockerRow {
        domain: source.domain.clone(),
        stage_id: source.stage_id.clone(),
        tool_id: source.tool_id.clone(),
        corpus_id: source.corpus_id.clone(),
        asset_profile_id: source.asset_profile_id.clone(),
        adapter_id: source.adapter_id.clone(),
        parser_id: source.parser_id.clone(),
        schema_id: source.schema_id.clone(),
        status: source.status.clone(),
        adapter_status: source.adapter_status.clone(),
        scope_exit_kind: source.scope_exit_kind.clone(),
        blocker_type: blocker_type.to_string(),
        blocker_path: blocker_path.to_string(),
        reason,
        stage_removed_from_active_scope: source.stage_removed_from_active_scope,
        tool_removed_from_active_scope: source.tool_removed_from_active_scope,
        absent_from_active_matrix: source.absent_from_active_matrix,
        absent_from_rendered_commands: source.absent_from_rendered_commands,
        absent_from_expected_results: source.absent_from_expected_results,
        absent_from_full_benchmark_report: source.absent_from_full_benchmark_report,
    })
}

fn blocker_contract_for_scope_exit_kind(
    scope_exit_kind: &str,
) -> Result<(&'static str, &'static str, String)> {
    match scope_exit_kind {
        SCOPE_EXIT_KIND_LIFECYCLE_NOT_ACTIVE => Ok((
            BLOCKER_TYPE_LIFECYCLE_NOT_ACTIVE,
            DEFAULT_NO_PLANNED_ROWS_PATH,
            "binding is intentionally outside final job-bearing active scope because its lifecycle status is not yet active".to_string(),
        )),
        SCOPE_EXIT_KIND_BENCHMARK_NOT_READY => Ok((
            BLOCKER_TYPE_BENCHMARK_NOT_READY,
            DEFAULT_NO_NOT_BENCHMARK_READY_ROWS_PATH,
            "binding is intentionally outside final job-bearing active scope because it is not benchmark ready".to_string(),
        )),
        SCOPE_EXIT_KIND_NON_EXECUTABLE_ADAPTER => Ok((
            BLOCKER_TYPE_NON_EXECUTABLE_ADAPTER,
            DEFAULT_NO_DECLARED_ONLY_ROWS_PATH,
            "binding is intentionally outside final job-bearing active scope because it lacks executable adapter coverage".to_string(),
        )),
        other => Err(anyhow!(
            "active-scope blocker table does not recognize removed scope exit kind `{other}`"
        )),
    }
}

fn ensure_all_domain_active_scope_blockers_contract(
    removed_report: &super::removed_from_scope::RemovedFromScopeReport,
    report: &AllDomainActiveScopeBlockersReport,
) -> Result<()> {
    if report.row_count != report.rows.len() {
        return Err(anyhow!("active-scope blocker report drifted from its row set"));
    }
    if report.violation_count != report.violations.len() {
        return Err(anyhow!("active-scope blocker report drifted from its violation set"));
    }
    if report.row_count != removed_report.removed_row_count {
        return Err(anyhow!(
            "active-scope blocker table must keep exactly one row per removed binding"
        ));
    }
    if report.ok && report.violation_count != 0 {
        return Err(anyhow!("active-scope blocker report cannot be ok while violations remain"));
    }
    if !report.ok && report.violation_count == 0 {
        return Err(anyhow!(
            "active-scope blocker report must keep explicit violating rows when failing"
        ));
    }
    if report
        .rows
        .iter()
        .any(|row| row.blocker_path.trim().is_empty() || row.blocker_type.trim().is_empty())
    {
        return Err(anyhow!(
            "active-scope blocker table must keep explicit blocker type and blocker path for every row"
        ));
    }
    Ok(())
}

fn render_all_domain_active_scope_blockers_tsv(rows: &[AllDomainActiveScopeBlockerRow]) -> String {
    let mut rendered = String::from(
        "domain\tstage_id\ttool_id\tcorpus_id\tasset_profile_id\tadapter_id\tparser_id\tschema_id\tstatus\tadapter_status\tscope_exit_kind\tblocker_type\tblocker_path\treason\tstage_removed_from_active_scope\ttool_removed_from_active_scope\tabsent_from_active_matrix\tabsent_from_rendered_commands\tabsent_from_expected_results\tabsent_from_full_benchmark_report\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
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
            sanitize_tsv(&row.blocker_type),
            sanitize_tsv(&row.blocker_path),
            sanitize_tsv(&row.reason),
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
