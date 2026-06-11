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
use super::bam_report_map::collect_bam_report_map_rows;
use super::fastq_report_map::collect_fastq_report_map_rows;
use super::vcf_report_map::collect_vcf_report_map_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ALL_DOMAIN_REPORT_MAP_COVERAGE_PATH: &str =
    "benchmarks/readiness/all-domains/report-map-coverage.tsv";
const ALL_DOMAIN_REPORT_MAP_COVERAGE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_report_map_coverage.v1";
const COVERAGE_STATUS_COVERED: &str = "covered";
const COVERAGE_STATUS_MISSING_EXPECTED_RESULT: &str = "missing_expected_result";
const COVERAGE_STATUS_MISSING_REPORT_MAP: &str = "missing_report_map";
const COVERAGE_STATUS_REPORT_SECTION_MISMATCH: &str = "report_section_mismatch";
const PROOF_SOURCE_FASTQ: &str = "fastq_report_map";
const PROOF_SOURCE_BAM: &str = "bam_report_map";
const PROOF_SOURCE_VCF: &str = "vcf_report_map";
const NO_VALUE: &str = "none";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CoverageKey {
    domain: String,
    stage_id: String,
    tool_id: String,
    corpus_id: String,
    asset_profile_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SourceKey {
    domain: String,
    stage_id: String,
    tool_id: String,
}

#[derive(Debug, Clone)]
struct ReportMapSourceRow {
    proof_source: String,
    report_section_id: String,
    summary_table_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AllDomainReportMapCoverageRow {
    pub(crate) result_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) adapter_id: String,
    pub(crate) parser_id: String,
    pub(crate) schema_id: String,
    pub(crate) expected_report_section: String,
    pub(crate) report_section_id: String,
    pub(crate) summary_table_id: String,
    pub(crate) proof_source: String,
    pub(crate) coverage_status: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AllDomainReportMapCoverageReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) result_id_count: usize,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) report_map_binding_count: usize,
    pub(crate) covered_row_count: usize,
    pub(crate) missing_row_count: usize,
    pub(crate) coverage_percent: f64,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) proof_source_counts: BTreeMap<String, usize>,
    pub(crate) report_section_counts: BTreeMap<String, usize>,
    pub(crate) summary_table_counts: BTreeMap<String, usize>,
    pub(crate) coverage_status_counts: BTreeMap<String, usize>,
    pub(crate) violation_count: usize,
    pub(crate) ok: bool,
    pub(crate) rows: Vec<AllDomainReportMapCoverageRow>,
    pub(crate) violations: Vec<AllDomainReportMapCoverageRow>,
}

pub(crate) fn run_render_all_domain_report_map_coverage(
    args: &parse::BenchReadinessRenderAllDomainReportMapCoverageArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_all_domain_report_map_coverage(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ALL_DOMAIN_REPORT_MAP_COVERAGE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_all_domain_report_map_coverage(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AllDomainReportMapCoverageReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let report = build_all_domain_report_map_coverage_report(repo_root, &output_path)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_all_domain_report_map_coverage_tsv(&report.rows))
        .with_context(|| format!("write {}", output_path.display()))?;
    if !report.ok {
        return Err(anyhow!("all-domain active rows must keep complete report-map coverage"));
    }
    Ok(report)
}

fn build_all_domain_report_map_coverage_report(
    repo_root: &Path,
    output_path: &Path,
) -> Result<AllDomainReportMapCoverageReport> {
    let active_rows = collect_all_domain_active_stage_tool_matrix_rows(repo_root)?;
    let expected_rows = collect_all_domain_expected_benchmark_result_rows(repo_root)?;
    let report_map_rows = collect_report_map_source_rows(repo_root)?;
    let expected_by_key = expected_rows
        .into_iter()
        .map(|row| (coverage_key_from_expected_row(&row), row))
        .collect::<BTreeMap<_, _>>();
    let report_map_binding_count = active_rows
        .iter()
        .filter_map(|row| {
            let key = source_key_from_active_row(row);
            report_map_rows.contains_key(&key).then_some(key)
        })
        .collect::<BTreeSet<_>>()
        .len();

    let mut rows = Vec::with_capacity(active_rows.len());
    for active_row in &active_rows {
        rows.push(render_row(
            active_row,
            expected_by_key.get(&coverage_key_from_active_row(active_row)),
            report_map_rows.get(&source_key_from_active_row(active_row)),
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
    let mut proof_source_counts = BTreeMap::<String, usize>::new();
    let mut report_section_counts = BTreeMap::<String, usize>::new();
    let mut summary_table_counts = BTreeMap::<String, usize>::new();
    let mut coverage_status_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
        *proof_source_counts.entry(row.proof_source.clone()).or_default() += 1;
        *report_section_counts.entry(row.report_section_id.clone()).or_default() += 1;
        *summary_table_counts.entry(row.summary_table_id.clone()).or_default() += 1;
        *coverage_status_counts.entry(row.coverage_status.clone()).or_default() += 1;
    }
    let violations = rows
        .iter()
        .filter(|row| row.coverage_status != COVERAGE_STATUS_COVERED)
        .cloned()
        .collect::<Vec<_>>();

    let report = AllDomainReportMapCoverageReport {
        schema_version: ALL_DOMAIN_REPORT_MAP_COVERAGE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        row_count,
        result_id_count,
        stage_count,
        tool_count,
        report_map_binding_count,
        covered_row_count,
        missing_row_count,
        coverage_percent,
        domain_counts,
        proof_source_counts,
        report_section_counts,
        summary_table_counts,
        coverage_status_counts,
        violation_count: violations.len(),
        ok: violations.is_empty(),
        rows,
        violations,
    };
    ensure_all_domain_report_map_coverage_contract(&active_rows, &report)?;
    Ok(report)
}

fn render_row(
    active_row: &AllDomainActiveStageToolMatrixRow,
    expected_row: Option<&AllDomainExpectedBenchmarkResultRow>,
    report_map_row: Option<&ReportMapSourceRow>,
) -> AllDomainReportMapCoverageRow {
    let expected_report_section =
        expected_row.map(|row| row.report_section.clone()).unwrap_or_else(|| NO_VALUE.to_string());
    let result_id =
        expected_row.map(|row| row.result_id.clone()).unwrap_or_else(|| NO_VALUE.to_string());
    match (expected_row, report_map_row) {
        (None, Some(report_map_row)) => AllDomainReportMapCoverageRow {
            result_id,
            domain: active_row.domain.clone(),
            stage_id: active_row.stage_id.clone(),
            tool_id: active_row.tool_id.clone(),
            corpus_id: active_row.corpus_id.clone(),
            asset_profile_id: active_row.asset_profile_id.clone(),
            adapter_id: active_row.adapter_id.clone(),
            parser_id: active_row.parser_id.clone(),
            schema_id: active_row.schema_id.clone(),
            expected_report_section,
            report_section_id: report_map_row.report_section_id.clone(),
            summary_table_id: report_map_row.summary_table_id.clone(),
            proof_source: report_map_row.proof_source.clone(),
            coverage_status: COVERAGE_STATUS_MISSING_EXPECTED_RESULT.to_string(),
            reason: format!(
                "active row `{}` / `{}` / `{}` is missing its canonical expected-result row, so report-map coverage cannot prove a stable result binding",
                active_row.domain, active_row.stage_id, active_row.tool_id
            ),
        },
        (Some(expected_row), None) => AllDomainReportMapCoverageRow {
            result_id,
            domain: active_row.domain.clone(),
            stage_id: active_row.stage_id.clone(),
            tool_id: active_row.tool_id.clone(),
            corpus_id: active_row.corpus_id.clone(),
            asset_profile_id: active_row.asset_profile_id.clone(),
            adapter_id: active_row.adapter_id.clone(),
            parser_id: active_row.parser_id.clone(),
            schema_id: active_row.schema_id.clone(),
            expected_report_section,
            report_section_id: NO_VALUE.to_string(),
            summary_table_id: NO_VALUE.to_string(),
            proof_source: proof_source_for_domain(&active_row.domain).to_string(),
            coverage_status: COVERAGE_STATUS_MISSING_REPORT_MAP.to_string(),
            reason: format!(
                "active row `{}` / `{}` / `{}` with result_id `{}` is missing governed report-map coverage in `{}`",
                active_row.domain,
                active_row.stage_id,
                active_row.tool_id,
                expected_row.result_id,
                proof_source_for_domain(&active_row.domain)
            ),
        },
        (Some(expected_row), Some(report_map_row)) => {
            let coverage_status =
                if expected_row.report_section == report_map_row.report_section_id {
                    COVERAGE_STATUS_COVERED
                } else {
                    COVERAGE_STATUS_REPORT_SECTION_MISMATCH
                };
            let reason = if coverage_status == COVERAGE_STATUS_COVERED {
                format!(
                    "active row `{}` / `{}` / `{}` appears in governed report section `{}` with summary table `{}` through `{}`",
                    active_row.domain,
                    active_row.stage_id,
                    active_row.tool_id,
                    report_map_row.report_section_id,
                    report_map_row.summary_table_id,
                    report_map_row.proof_source
                )
            } else {
                format!(
                    "active row `{}` / `{}` / `{}` expected report section `{}` but `{}` mapped it to `{}`",
                    active_row.domain,
                    active_row.stage_id,
                    active_row.tool_id,
                    expected_row.report_section,
                    report_map_row.proof_source,
                    report_map_row.report_section_id
                )
            };
            AllDomainReportMapCoverageRow {
                result_id,
                domain: active_row.domain.clone(),
                stage_id: active_row.stage_id.clone(),
                tool_id: active_row.tool_id.clone(),
                corpus_id: active_row.corpus_id.clone(),
                asset_profile_id: active_row.asset_profile_id.clone(),
                adapter_id: active_row.adapter_id.clone(),
                parser_id: active_row.parser_id.clone(),
                schema_id: active_row.schema_id.clone(),
                expected_report_section,
                report_section_id: report_map_row.report_section_id.clone(),
                summary_table_id: report_map_row.summary_table_id.clone(),
                proof_source: report_map_row.proof_source.clone(),
                coverage_status: coverage_status.to_string(),
                reason,
            }
        }
        (None, None) => AllDomainReportMapCoverageRow {
            result_id,
            domain: active_row.domain.clone(),
            stage_id: active_row.stage_id.clone(),
            tool_id: active_row.tool_id.clone(),
            corpus_id: active_row.corpus_id.clone(),
            asset_profile_id: active_row.asset_profile_id.clone(),
            adapter_id: active_row.adapter_id.clone(),
            parser_id: active_row.parser_id.clone(),
            schema_id: active_row.schema_id.clone(),
            expected_report_section,
            report_section_id: NO_VALUE.to_string(),
            summary_table_id: NO_VALUE.to_string(),
            proof_source: proof_source_for_domain(&active_row.domain).to_string(),
            coverage_status: COVERAGE_STATUS_MISSING_EXPECTED_RESULT.to_string(),
            reason: format!(
                "active row `{}` / `{}` / `{}` is missing both expected-result and report-map coverage",
                active_row.domain, active_row.stage_id, active_row.tool_id
            ),
        },
    }
}

fn collect_report_map_source_rows(
    repo_root: &Path,
) -> Result<BTreeMap<SourceKey, ReportMapSourceRow>> {
    let mut rows = BTreeMap::<SourceKey, ReportMapSourceRow>::new();

    for row in collect_fastq_report_map_rows(repo_root)? {
        insert_report_map_row(
            &mut rows,
            SourceKey {
                domain: "fastq".to_string(),
                stage_id: row.stage_id.clone(),
                tool_id: NO_VALUE.to_string(),
            },
            ReportMapSourceRow {
                proof_source: PROOF_SOURCE_FASTQ.to_string(),
                report_section_id: row.report_section_id,
                summary_table_id: row.summary_table_id,
            },
        )?;
    }

    for row in collect_bam_report_map_rows(repo_root)? {
        insert_report_map_row(
            &mut rows,
            SourceKey {
                domain: "bam".to_string(),
                stage_id: row.stage_id.clone(),
                tool_id: NO_VALUE.to_string(),
            },
            ReportMapSourceRow {
                proof_source: PROOF_SOURCE_BAM.to_string(),
                report_section_id: row.report_section_id,
                summary_table_id: row.summary_table_id,
            },
        )?;
    }

    for row in collect_vcf_report_map_rows(repo_root)? {
        insert_report_map_row(
            &mut rows,
            SourceKey {
                domain: "vcf".to_string(),
                stage_id: row.stage_id.clone(),
                tool_id: row.tool_id.clone(),
            },
            ReportMapSourceRow {
                proof_source: PROOF_SOURCE_VCF.to_string(),
                report_section_id: row.section_id,
                summary_table_id: row.summary_table,
            },
        )?;
    }

    Ok(rows)
}

fn insert_report_map_row(
    rows: &mut BTreeMap<SourceKey, ReportMapSourceRow>,
    key: SourceKey,
    row: ReportMapSourceRow,
) -> Result<()> {
    if rows.insert(key.clone(), row).is_some() {
        return Err(anyhow!(
            "all-domain report-map coverage found duplicate report-map source for `{}` / `{}` / `{}`",
            key.domain,
            key.stage_id,
            key.tool_id
        ));
    }
    Ok(())
}

fn ensure_all_domain_report_map_coverage_contract(
    active_rows: &[AllDomainActiveStageToolMatrixRow],
    report: &AllDomainReportMapCoverageReport,
) -> Result<()> {
    if report.row_count != active_rows.len() || report.row_count != report.rows.len() {
        return Err(anyhow!(
            "all-domain report-map coverage must keep exactly one row per active binding"
        ));
    }
    if report.covered_row_count + report.missing_row_count != report.row_count {
        return Err(anyhow!(
            "all-domain report-map coverage drifted from its covered/missing counts"
        ));
    }
    if report.violation_count != report.violations.len() {
        return Err(anyhow!(
            "all-domain report-map coverage violation count drifted from the violation rows"
        ));
    }
    if report.covered_row_count != report.row_count || !report.ok {
        return Err(anyhow!(
            "all-domain report-map coverage must keep governed report sections for every active row"
        ));
    }
    if report.result_id_count != report.row_count
        || report.report_map_binding_count > report.row_count
    {
        return Err(anyhow!(
            "all-domain report-map coverage must keep one unique result id per active row and report-map bindings cannot exceed active rows; found rows={}, result_ids={}, bindings={}",
            report.row_count,
            report.result_id_count,
            report.report_map_binding_count
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

fn source_key_from_active_row(active_row: &AllDomainActiveStageToolMatrixRow) -> SourceKey {
    SourceKey {
        domain: active_row.domain.clone(),
        stage_id: active_row.stage_id.clone(),
        tool_id: if active_row.domain == "vcf" {
            active_row.tool_id.clone()
        } else {
            NO_VALUE.to_string()
        },
    }
}

fn proof_source_for_domain(domain: &str) -> &'static str {
    match domain {
        "fastq" => PROOF_SOURCE_FASTQ,
        "bam" => PROOF_SOURCE_BAM,
        "vcf" => PROOF_SOURCE_VCF,
        _ => NO_VALUE,
    }
}

fn render_all_domain_report_map_coverage_tsv(rows: &[AllDomainReportMapCoverageRow]) -> String {
    let mut output = String::from(
        "result_id\tdomain\tstage_id\ttool_id\tcorpus_id\tasset_profile_id\tadapter_id\tparser_id\tschema_id\texpected_report_section\treport_section_id\tsummary_table_id\tproof_source\tcoverage_status\treason\n",
    );
    for row in rows {
        output.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            row.result_id,
            row.domain,
            row.stage_id,
            row.tool_id,
            row.corpus_id,
            row.asset_profile_id,
            row.adapter_id,
            row.parser_id,
            row.schema_id,
            row.expected_report_section,
            row.report_section_id,
            row.summary_table_id,
            row.proof_source,
            row.coverage_status,
            row.reason
        ));
    }
    output
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
