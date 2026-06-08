use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::all_domain_active_stage_tool_matrix::{
    collect_all_domain_active_stage_tool_matrix_rows, AllDomainActiveStageToolMatrixRow,
};
use super::all_domain_expected_benchmark_results::{
    collect_all_domain_expected_benchmark_result_rows, AllDomainExpectedBenchmarkResultRow,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ALL_DOMAIN_EXPECTED_RESULT_COVERAGE_PATH: &str =
    "benchmarks/readiness/all-domains/expected-result-coverage.tsv";
const ALL_DOMAIN_EXPECTED_RESULT_COVERAGE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_expected_result_coverage.v1";
const COVERAGE_STATUS_COVERED: &str = "covered";
const COVERAGE_STATUS_MISSING_EXPECTED_RESULT: &str = "missing_expected_result";
const NO_VALUE: &str = "none";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CoverageKey {
    domain: String,
    stage_id: String,
    tool_id: String,
    corpus_id: String,
    asset_profile_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AllDomainExpectedResultCoverageRow {
    pub(crate) result_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) adapter_id: String,
    pub(crate) parser_id: String,
    pub(crate) schema_id: String,
    pub(crate) expected_outputs: Vec<String>,
    pub(crate) expected_metrics: Vec<String>,
    pub(crate) report_section: String,
    pub(crate) coverage_status: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AllDomainExpectedResultCoverageReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) result_id_count: usize,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) expected_result_binding_count: usize,
    pub(crate) covered_row_count: usize,
    pub(crate) missing_row_count: usize,
    pub(crate) coverage_percent: f64,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) report_section_counts: BTreeMap<String, usize>,
    pub(crate) coverage_status_counts: BTreeMap<String, usize>,
    pub(crate) violation_count: usize,
    pub(crate) ok: bool,
    pub(crate) rows: Vec<AllDomainExpectedResultCoverageRow>,
    pub(crate) violations: Vec<AllDomainExpectedResultCoverageRow>,
}

pub(crate) fn run_render_all_domain_expected_result_coverage(
    args: &parse::BenchReadinessRenderAllDomainExpectedResultCoverageArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_all_domain_expected_result_coverage(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ALL_DOMAIN_EXPECTED_RESULT_COVERAGE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_all_domain_expected_result_coverage(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AllDomainExpectedResultCoverageReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let report = build_all_domain_expected_result_coverage_report(repo_root, &output_path)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_all_domain_expected_result_coverage_tsv(&report.rows))
        .with_context(|| format!("write {}", output_path.display()))?;
    if !report.ok {
        return Err(anyhow!("all-domain active rows must keep complete expected-result coverage"));
    }
    Ok(report)
}

fn build_all_domain_expected_result_coverage_report(
    repo_root: &Path,
    output_path: &Path,
) -> Result<AllDomainExpectedResultCoverageReport> {
    let active_rows = collect_all_domain_active_stage_tool_matrix_rows(repo_root)?;
    let expected_rows = collect_all_domain_expected_benchmark_result_rows(repo_root)?;
    let active_keys = active_rows.iter().map(coverage_key_from_active_row).collect::<BTreeSet<_>>();
    let expected_by_key = expected_rows
        .into_iter()
        .map(|row| (coverage_key_from_expected_row(&row), row))
        .collect::<BTreeMap<_, _>>();
    let expected_keys = expected_by_key.keys().cloned().collect::<BTreeSet<_>>();

    let mut rows = Vec::with_capacity(active_rows.len());
    for active_row in &active_rows {
        rows.push(render_row(
            active_row,
            expected_by_key.get(&coverage_key_from_active_row(active_row)),
        ));
    }
    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.result_id.cmp(&right.result_id))
    });

    let row_count = rows.len();
    let covered_row_count =
        rows.iter().filter(|row| row.coverage_status == COVERAGE_STATUS_COVERED).count();
    let missing_row_count = row_count.saturating_sub(covered_row_count);
    let coverage_percent =
        if row_count == 0 { 0.0 } else { covered_row_count as f64 * 100.0 / row_count as f64 };
    let result_id_count = rows
        .iter()
        .map(|row| row.result_id.as_str())
        .filter(|result_id| *result_id != NO_VALUE)
        .collect::<BTreeSet<_>>()
        .len();
    let stage_count = rows
        .iter()
        .map(|row| (row.domain.as_str(), row.stage_id.as_str()))
        .collect::<BTreeSet<_>>()
        .len();
    let tool_count = rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len();
    let mut domain_counts = BTreeMap::<String, usize>::new();
    let mut report_section_counts = BTreeMap::<String, usize>::new();
    let mut coverage_status_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
        *report_section_counts.entry(row.report_section.clone()).or_default() += 1;
        *coverage_status_counts.entry(row.coverage_status.clone()).or_default() += 1;
    }
    let violations = rows
        .iter()
        .filter(|row| row.coverage_status != COVERAGE_STATUS_COVERED)
        .cloned()
        .collect::<Vec<_>>();

    let report = AllDomainExpectedResultCoverageReport {
        schema_version: ALL_DOMAIN_EXPECTED_RESULT_COVERAGE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        row_count,
        result_id_count,
        stage_count,
        tool_count,
        expected_result_binding_count: expected_keys.len(),
        covered_row_count,
        missing_row_count,
        coverage_percent,
        domain_counts,
        report_section_counts,
        coverage_status_counts,
        violation_count: violations.len(),
        ok: violations.is_empty(),
        rows,
        violations,
    };
    ensure_all_domain_expected_result_coverage_contract(
        &active_rows,
        &active_keys,
        &expected_keys,
        &report,
    )?;
    Ok(report)
}

fn render_row(
    active_row: &AllDomainActiveStageToolMatrixRow,
    expected_row: Option<&AllDomainExpectedBenchmarkResultRow>,
) -> AllDomainExpectedResultCoverageRow {
    match expected_row {
        Some(expected_row) => AllDomainExpectedResultCoverageRow {
            result_id: expected_row.result_id.clone(),
            domain: active_row.domain.clone(),
            stage_id: active_row.stage_id.clone(),
            tool_id: active_row.tool_id.clone(),
            corpus_id: active_row.corpus_id.clone(),
            asset_profile_id: active_row.asset_profile_id.clone(),
            adapter_id: active_row.adapter_id.clone(),
            parser_id: active_row.parser_id.clone(),
            schema_id: active_row.schema_id.clone(),
            expected_outputs: expected_row.expected_outputs.clone(),
            expected_metrics: expected_row.expected_metrics.clone(),
            report_section: expected_row.report_section.clone(),
            coverage_status: COVERAGE_STATUS_COVERED.to_string(),
            reason: format!(
                "active row `{}` / `{}` / `{}` keeps governed expected-result coverage with result_id `{}` in report section `{}`",
                active_row.domain,
                active_row.stage_id,
                active_row.tool_id,
                expected_row.result_id,
                expected_row.report_section
            ),
        },
        None => AllDomainExpectedResultCoverageRow {
            result_id: NO_VALUE.to_string(),
            domain: active_row.domain.clone(),
            stage_id: active_row.stage_id.clone(),
            tool_id: active_row.tool_id.clone(),
            corpus_id: active_row.corpus_id.clone(),
            asset_profile_id: active_row.asset_profile_id.clone(),
            adapter_id: active_row.adapter_id.clone(),
            parser_id: active_row.parser_id.clone(),
            schema_id: active_row.schema_id.clone(),
            expected_outputs: Vec::new(),
            expected_metrics: Vec::new(),
            report_section: NO_VALUE.to_string(),
            coverage_status: COVERAGE_STATUS_MISSING_EXPECTED_RESULT.to_string(),
            reason: format!(
                "active row `{}` / `{}` / `{}` / `{}` / `{}` is missing an all-domain expected-result row",
                active_row.domain,
                active_row.stage_id,
                active_row.tool_id,
                active_row.corpus_id,
                active_row.asset_profile_id
            ),
        },
    }
}

fn ensure_all_domain_expected_result_coverage_contract(
    active_rows: &[AllDomainActiveStageToolMatrixRow],
    active_keys: &BTreeSet<CoverageKey>,
    expected_keys: &BTreeSet<CoverageKey>,
    report: &AllDomainExpectedResultCoverageReport,
) -> Result<()> {
    if report.row_count != active_rows.len() || report.row_count != report.rows.len() {
        return Err(anyhow!(
            "all-domain expected-result coverage must keep exactly one row per active binding"
        ));
    }
    if report.covered_row_count + report.missing_row_count != report.row_count {
        return Err(anyhow!(
            "all-domain expected-result coverage drifted from its covered/missing counts"
        ));
    }
    if active_keys != expected_keys {
        let missing_keys =
            active_keys.difference(expected_keys).map(display_coverage_key).collect::<Vec<_>>();
        let extra_keys =
            expected_keys.difference(active_keys).map(display_coverage_key).collect::<Vec<_>>();
        return Err(anyhow!(
            "all-domain expected-result coverage drifted from the governed active slice; missing={}, extra={}",
            if missing_keys.is_empty() { "none".to_string() } else { missing_keys.join(", ") },
            if extra_keys.is_empty() { "none".to_string() } else { extra_keys.join(", ") },
        ));
    }
    if report.violation_count != report.violations.len() {
        return Err(anyhow!(
            "all-domain expected-result coverage violation count drifted from the violation rows"
        ));
    }
    if report.covered_row_count != report.row_count || !report.ok {
        return Err(anyhow!(
            "all-domain expected-result coverage must keep governed expected-result coverage for every active row"
        ));
    }
    if report.row_count != 124 || report.result_id_count != 124 {
        return Err(anyhow!(
            "all-domain expected-result coverage must retain exactly 124 governed active result rows, found {} rows and {} result ids",
            report.row_count,
            report.result_id_count
        ));
    }
    Ok(())
}

fn coverage_key_from_active_row(active_row: &AllDomainActiveStageToolMatrixRow) -> CoverageKey {
    CoverageKey {
        domain: active_row.domain.clone(),
        stage_id: active_row.stage_id.clone(),
        tool_id: active_row.tool_id.clone(),
        corpus_id: active_row.corpus_id.clone(),
        asset_profile_id: active_row.asset_profile_id.clone(),
    }
}

fn coverage_key_from_expected_row(
    expected_row: &AllDomainExpectedBenchmarkResultRow,
) -> CoverageKey {
    CoverageKey {
        domain: expected_row.domain.clone(),
        stage_id: expected_row.stage_id.clone(),
        tool_id: expected_row.tool_id.clone(),
        corpus_id: expected_row.corpus_id.clone(),
        asset_profile_id: expected_row.asset_profile_id.clone(),
    }
}

fn display_coverage_key(key: &CoverageKey) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        key.domain, key.stage_id, key.tool_id, key.corpus_id, key.asset_profile_id
    )
}

fn render_all_domain_expected_result_coverage_tsv(
    rows: &[AllDomainExpectedResultCoverageRow],
) -> String {
    let mut output = String::from(
        "result_id\tdomain\tstage_id\ttool_id\tcorpus_id\tasset_profile_id\tadapter_id\tparser_id\tschema_id\texpected_outputs\texpected_metrics\treport_section\tcoverage_status\treason\n",
    );
    for row in rows {
        output.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            row.result_id,
            row.domain,
            row.stage_id,
            row.tool_id,
            row.corpus_id,
            row.asset_profile_id,
            row.adapter_id,
            row.parser_id,
            row.schema_id,
            join_values(&row.expected_outputs),
            join_values(&row.expected_metrics),
            row.report_section,
            row.coverage_status,
            row.reason
        ));
    }
    output
}

fn join_values(values: &[String]) -> String {
    if values.is_empty() {
        NO_VALUE.to_string()
    } else {
        values.join(",")
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}
