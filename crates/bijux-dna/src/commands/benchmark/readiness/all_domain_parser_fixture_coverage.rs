use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::all_domain_active_stage_tool_matrix::{
    collect_all_domain_active_stage_tool_matrix_rows, AllDomainActiveStageToolMatrixRow,
};
use super::bam_parser_fixture_coverage::{
    collect_bam_parser_fixture_coverage_rows, BamParserFixtureCoverageRow,
    BamParserFixtureCoverageStatus,
};
use super::fastq_parser_fixture_coverage::{
    collect_fastq_parser_fixture_coverage_rows, FastqParserFixtureCoverageRow,
    FastqParserFixtureCoverageStatus,
};
use super::vcf_parser_fixture_coverage::{
    collect_vcf_parser_fixture_coverage_rows, VcfParserFixtureCoverageRow,
    VcfParserFixtureCoverageStatus,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ALL_DOMAIN_PARSER_FIXTURE_COVERAGE_PATH: &str =
    "benchmarks/readiness/all-domains/parser-fixture-coverage.tsv";
const ALL_DOMAIN_PARSER_FIXTURE_COVERAGE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_parser_fixture_coverage.v1";
const COVERAGE_STATUS_COVERED: &str = "covered";
const COVERAGE_STATUS_MISSING_ACTIVE_BINDING: &str = "missing_active_binding";
const FIXTURE_REFERENCE_KIND_ASSET_SCOPE: &str = "asset_scope";
const FIXTURE_REFERENCE_KIND_FASTQ_CASE: &str = "fixture_case";
const FIXTURE_REFERENCE_KIND_VCF_FIXTURE_DIRECTORY: &str = "fixture_directory";
const FIXTURE_REFERENCE_KIND_NONE: &str = "none";
const PROOF_SOURCE_FASTQ: &str = "fastq_parser_fixture_coverage";
const PROOF_SOURCE_BAM: &str = "bam_parser_fixture_coverage";
const PROOF_SOURCE_VCF: &str = "vcf_parser_fixture_coverage";
const NO_VALUE: &str = "none";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CoverageKey {
    domain: String,
    stage_id: String,
    tool_id: String,
}

#[derive(Debug, Clone)]
struct ParserProofRow {
    parser_fixture_parser_id: String,
    parser_fixture_schema_id: String,
    parser_fixture_reference: String,
    parser_fixture_reference_kind: String,
    proof_source: String,
    coverage_status: String,
    reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AllDomainParserFixtureCoverageRow {
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) adapter_id: String,
    pub(crate) parser_id: String,
    pub(crate) schema_id: String,
    pub(crate) parser_fixture_parser_id: String,
    pub(crate) parser_fixture_schema_id: String,
    pub(crate) parser_fixture_reference_kind: String,
    pub(crate) parser_fixture_reference: String,
    pub(crate) proof_source: String,
    pub(crate) coverage_status: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AllDomainParserFixtureCoverageReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) parser_proof_binding_count: usize,
    pub(crate) covered_row_count: usize,
    pub(crate) missing_row_count: usize,
    pub(crate) coverage_percent: f64,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) proof_source_counts: BTreeMap<String, usize>,
    pub(crate) coverage_status_counts: BTreeMap<String, usize>,
    pub(crate) violation_count: usize,
    pub(crate) ok: bool,
    pub(crate) rows: Vec<AllDomainParserFixtureCoverageRow>,
    pub(crate) violations: Vec<AllDomainParserFixtureCoverageRow>,
}

pub(crate) fn run_render_all_domain_parser_fixture_coverage(
    args: &parse::BenchReadinessRenderAllDomainParserFixtureCoverageArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_all_domain_parser_fixture_coverage(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ALL_DOMAIN_PARSER_FIXTURE_COVERAGE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_all_domain_parser_fixture_coverage(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AllDomainParserFixtureCoverageReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let report = build_all_domain_parser_fixture_coverage_report(repo_root, &output_path)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_all_domain_parser_fixture_coverage_tsv(&report.rows))
        .with_context(|| format!("write {}", output_path.display()))?;
    if !report.ok {
        return Err(anyhow!("all-domain active rows must keep complete parser-fixture proof"));
    }
    Ok(report)
}

fn build_all_domain_parser_fixture_coverage_report(
    repo_root: &Path,
    output_path: &Path,
) -> Result<AllDomainParserFixtureCoverageReport> {
    let active_rows = collect_all_domain_active_stage_tool_matrix_rows(repo_root)?;
    let parser_proof_by_key = collect_parser_proof_by_key(repo_root)?;
    let active_coverage_keys =
        active_rows.iter().map(coverage_key_from_active_row).collect::<BTreeSet<_>>();
    let proof_coverage_keys = parser_proof_by_key.keys().cloned().collect::<BTreeSet<_>>();

    let mut rows = Vec::with_capacity(active_rows.len());
    for active_row in &active_rows {
        rows.push(render_row(
            active_row,
            parser_proof_by_key.get(&coverage_key_from_active_row(active_row)),
        )?);
    }
    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.corpus_id.cmp(&right.corpus_id))
            .then_with(|| left.asset_profile_id.cmp(&right.asset_profile_id))
    });

    let covered_row_count =
        rows.iter().filter(|row| row.coverage_status == COVERAGE_STATUS_COVERED).count();
    let missing_row_count = rows.len().saturating_sub(covered_row_count);
    let coverage_percent =
        if rows.is_empty() { 0.0 } else { covered_row_count as f64 * 100.0 / rows.len() as f64 };
    let stage_count = active_rows
        .iter()
        .map(|row| (row.domain.as_str(), row.stage_id.as_str()))
        .collect::<BTreeSet<_>>()
        .len();
    let tool_count =
        active_rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len();
    let mut domain_counts = BTreeMap::<String, usize>::new();
    let mut proof_source_counts = BTreeMap::<String, usize>::new();
    let mut coverage_status_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
        *proof_source_counts.entry(row.proof_source.clone()).or_default() += 1;
        *coverage_status_counts.entry(row.coverage_status.clone()).or_default() += 1;
    }
    let violations = rows
        .iter()
        .filter(|row| row.coverage_status != COVERAGE_STATUS_COVERED)
        .cloned()
        .collect::<Vec<_>>();

    let report = AllDomainParserFixtureCoverageReport {
        schema_version: ALL_DOMAIN_PARSER_FIXTURE_COVERAGE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        row_count: rows.len(),
        stage_count,
        tool_count,
        parser_proof_binding_count: proof_coverage_keys.len(),
        covered_row_count,
        missing_row_count,
        coverage_percent,
        domain_counts,
        proof_source_counts,
        coverage_status_counts,
        violation_count: violations.len(),
        ok: violations.is_empty(),
        rows,
        violations,
    };
    ensure_all_domain_parser_fixture_coverage_contract(
        &active_rows,
        &active_coverage_keys,
        &proof_coverage_keys,
        &report,
    )?;
    Ok(report)
}

fn render_row(
    active_row: &AllDomainActiveStageToolMatrixRow,
    proof_row: Option<&ParserProofRow>,
) -> Result<AllDomainParserFixtureCoverageRow> {
    match proof_row {
        Some(proof_row) => {
            Ok(AllDomainParserFixtureCoverageRow {
                domain: active_row.domain.clone(),
                stage_id: active_row.stage_id.clone(),
                tool_id: active_row.tool_id.clone(),
                corpus_id: active_row.corpus_id.clone(),
                asset_profile_id: active_row.asset_profile_id.clone(),
                adapter_id: active_row.adapter_id.clone(),
                parser_id: active_row.parser_id.clone(),
                schema_id: active_row.schema_id.clone(),
                parser_fixture_parser_id: proof_row.parser_fixture_parser_id.clone(),
                parser_fixture_schema_id: proof_row.parser_fixture_schema_id.clone(),
                parser_fixture_reference_kind: proof_row.parser_fixture_reference_kind.clone(),
                parser_fixture_reference: proof_row.parser_fixture_reference.clone(),
                proof_source: proof_row.proof_source.clone(),
                coverage_status: proof_row.coverage_status.clone(),
                reason: proof_row.reason.clone(),
            })
        }
        None => Ok(AllDomainParserFixtureCoverageRow {
            domain: active_row.domain.clone(),
            stage_id: active_row.stage_id.clone(),
            tool_id: active_row.tool_id.clone(),
            corpus_id: active_row.corpus_id.clone(),
            asset_profile_id: active_row.asset_profile_id.clone(),
            adapter_id: active_row.adapter_id.clone(),
            parser_id: active_row.parser_id.clone(),
            schema_id: active_row.schema_id.clone(),
            parser_fixture_parser_id: NO_VALUE.to_string(),
            parser_fixture_schema_id: NO_VALUE.to_string(),
            parser_fixture_reference_kind: FIXTURE_REFERENCE_KIND_NONE.to_string(),
            parser_fixture_reference: NO_VALUE.to_string(),
            proof_source: parser_proof_source_for_domain(&active_row.domain)?.to_string(),
            coverage_status: COVERAGE_STATUS_MISSING_ACTIVE_BINDING.to_string(),
            reason: format!(
                "active row `{}` / `{}` / `{}` is missing a parser-proof binding in the governed {} surface",
                active_row.domain,
                active_row.stage_id,
                active_row.tool_id,
                parser_proof_source_for_domain(&active_row.domain)?,
            ),
        }),
    }
}

fn collect_parser_proof_by_key(repo_root: &Path) -> Result<BTreeMap<CoverageKey, ParserProofRow>> {
    let mut rows = BTreeMap::<CoverageKey, ParserProofRow>::new();

    let (_, _, fastq_rows) = collect_fastq_parser_fixture_coverage_rows(repo_root)?;
    for row in fastq_rows {
        insert_parser_proof_row(
            &mut rows,
            CoverageKey {
                domain: "fastq".to_string(),
                stage_id: row.stage_id.clone(),
                tool_id: row.tool_id.clone(),
            },
            render_fastq_parser_proof_row(row),
        )?;
    }

    let (_, _, bam_rows, _) = collect_bam_parser_fixture_coverage_rows(repo_root)?;
    for row in bam_rows {
        insert_parser_proof_row(
            &mut rows,
            CoverageKey {
                domain: "bam".to_string(),
                stage_id: row.stage_id.clone(),
                tool_id: row.tool_id.clone(),
            },
            render_bam_parser_proof_row(row),
        )?;
    }

    let (_, _, vcf_rows) = collect_vcf_parser_fixture_coverage_rows(repo_root)?;
    for row in vcf_rows {
        insert_parser_proof_row(
            &mut rows,
            CoverageKey {
                domain: "vcf".to_string(),
                stage_id: row.stage_id.clone(),
                tool_id: row.tool_id.clone(),
            },
            render_vcf_parser_proof_row(row),
        )?;
    }

    Ok(rows)
}

fn insert_parser_proof_row(
    rows: &mut BTreeMap<CoverageKey, ParserProofRow>,
    key: CoverageKey,
    row: ParserProofRow,
) -> Result<()> {
    if rows.insert(key.clone(), row).is_some() {
        return Err(anyhow!(
            "all-domain parser fixture coverage encountered duplicate proof rows for `{}` / `{}` / `{}`",
            key.domain,
            key.stage_id,
            key.tool_id
        ));
    }
    Ok(())
}

fn render_fastq_parser_proof_row(row: FastqParserFixtureCoverageRow) -> ParserProofRow {
    ParserProofRow {
        parser_fixture_parser_id: row.parser_fixture_parser_id,
        parser_fixture_schema_id: row.parser_fixture_schema_id,
        parser_fixture_reference_kind: if row.parser_fixture_reference.trim().is_empty() {
            FIXTURE_REFERENCE_KIND_NONE.to_string()
        } else {
            FIXTURE_REFERENCE_KIND_FASTQ_CASE.to_string()
        },
        parser_fixture_reference: if row.parser_fixture_reference.trim().is_empty() {
            NO_VALUE.to_string()
        } else {
            row.parser_fixture_reference
        },
        proof_source: PROOF_SOURCE_FASTQ.to_string(),
        coverage_status: match row.coverage_status {
            FastqParserFixtureCoverageStatus::Covered => COVERAGE_STATUS_COVERED.to_string(),
            FastqParserFixtureCoverageStatus::MissingFixtureBinding => {
                "missing_fixture_binding".to_string()
            }
            FastqParserFixtureCoverageStatus::MissingFixtureCase => {
                "missing_fixture_case".to_string()
            }
            FastqParserFixtureCoverageStatus::InvalidFixtureCase => {
                "invalid_fixture_case".to_string()
            }
        },
        reason: row.reason,
    }
}

fn render_bam_parser_proof_row(row: BamParserFixtureCoverageRow) -> ParserProofRow {
    ParserProofRow {
        parser_fixture_parser_id: NO_VALUE.to_string(),
        parser_fixture_schema_id: NO_VALUE.to_string(),
        parser_fixture_reference: row.parser_fixture_reference,
        parser_fixture_reference_kind: row.parser_fixture_reference_kind,
        proof_source: PROOF_SOURCE_BAM.to_string(),
        coverage_status: match row.coverage_status {
            BamParserFixtureCoverageStatus::Covered => COVERAGE_STATUS_COVERED.to_string(),
            BamParserFixtureCoverageStatus::Missing => "missing".to_string(),
        },
        reason: row.reason,
    }
}

fn render_vcf_parser_proof_row(row: VcfParserFixtureCoverageRow) -> ParserProofRow {
    let coverage_status = match row.coverage_status {
        VcfParserFixtureCoverageStatus::Covered => COVERAGE_STATUS_COVERED.to_string(),
        VcfParserFixtureCoverageStatus::MissingFixtureInventory => {
            "missing_fixture_inventory".to_string()
        }
        VcfParserFixtureCoverageStatus::MissingFixtureDirectory => {
            "missing_fixture_directory".to_string()
        }
        VcfParserFixtureCoverageStatus::MissingExpectedNormalizedJson => {
            "missing_expected_normalized_json".to_string()
        }
        VcfParserFixtureCoverageStatus::MissingRawFixtures => "missing_raw_fixtures".to_string(),
        VcfParserFixtureCoverageStatus::InvalidExpectedNormalizedJson => {
            "invalid_expected_normalized_json".to_string()
        }
    };
    let parser_fixture_reference = if row.parser_fixture_root_path.trim().is_empty() {
        NO_VALUE.to_string()
    } else {
        row.parser_fixture_root_path.clone()
    };
    let parser_fixture_reference_kind = if row.parser_fixture_root_path.trim().is_empty() {
        FIXTURE_REFERENCE_KIND_NONE.to_string()
    } else {
        FIXTURE_REFERENCE_KIND_VCF_FIXTURE_DIRECTORY.to_string()
    };

    ParserProofRow {
        parser_fixture_parser_id: if row.parser_fixture_parser_id.trim().is_empty() {
            NO_VALUE.to_string()
        } else {
            row.parser_fixture_parser_id
        },
        parser_fixture_schema_id: if row.parser_fixture_schema_id.trim().is_empty() {
            NO_VALUE.to_string()
        } else {
            row.parser_fixture_schema_id
        },
        parser_fixture_reference,
        parser_fixture_reference_kind,
        proof_source: PROOF_SOURCE_VCF.to_string(),
        coverage_status,
        reason: row.reason,
    }
}

fn ensure_all_domain_parser_fixture_coverage_contract(
    active_rows: &[AllDomainActiveStageToolMatrixRow],
    active_coverage_keys: &BTreeSet<CoverageKey>,
    proof_coverage_keys: &BTreeSet<CoverageKey>,
    report: &AllDomainParserFixtureCoverageReport,
) -> Result<()> {
    if report.row_count != active_rows.len() || report.row_count != report.rows.len() {
        return Err(anyhow!(
            "all-domain parser fixture coverage must keep exactly one row per active binding"
        ));
    }
    if report.covered_row_count + report.missing_row_count != report.row_count {
        return Err(anyhow!(
            "all-domain parser fixture coverage drifted from its covered/missing counts"
        ));
    }
    if report.violation_count != report.violations.len() {
        return Err(anyhow!("all-domain parser fixture coverage drifted from its violation set"));
    }
    if active_coverage_keys != proof_coverage_keys {
        let missing_keys = active_coverage_keys
            .difference(proof_coverage_keys)
            .map(format_coverage_key)
            .collect::<Vec<_>>();
        let extra_keys = proof_coverage_keys
            .difference(active_coverage_keys)
            .map(format_coverage_key)
            .collect::<Vec<_>>();
        return Err(anyhow!(
            "all-domain parser fixture coverage drifted from active scope, missing proof keys: [{}], extra proof keys: [{}]",
            missing_keys.join(", "),
            extra_keys.join(", ")
        ));
    }

    let reported_keys =
        report.rows.iter().map(coverage_key_from_report_row).collect::<BTreeSet<_>>();
    if reported_keys != *active_coverage_keys {
        return Err(anyhow!(
            "all-domain parser fixture coverage TSV must match the governed active coverage-key set"
        ));
    }
    if report.ok && report.violation_count != 0 {
        return Err(anyhow!(
            "all-domain parser fixture coverage cannot be ok while violations remain"
        ));
    }
    if !report.ok && report.violation_count == 0 {
        return Err(anyhow!(
            "all-domain parser fixture coverage must keep explicit violations when failing"
        ));
    }
    if report.ok
        && (report.covered_row_count != report.row_count || report.coverage_percent != 100.0)
    {
        return Err(anyhow!(
            "all-domain parser fixture coverage must reach 100% when no violations remain"
        ));
    }
    for row in &report.rows {
        if row.parser_id.trim().is_empty()
            || row.schema_id.trim().is_empty()
            || row.parser_fixture_parser_id.trim().is_empty()
            || row.parser_fixture_schema_id.trim().is_empty()
            || row.proof_source.trim().is_empty()
            || row.coverage_status.trim().is_empty()
        {
            return Err(anyhow!(
                "all-domain parser fixture coverage row `{}` / `{}` / `{}` is missing a required field",
                row.domain,
                row.stage_id,
                row.tool_id
            ));
        }
    }
    Ok(())
}

fn render_all_domain_parser_fixture_coverage_tsv(
    rows: &[AllDomainParserFixtureCoverageRow],
) -> String {
    let mut rendered = String::from(
        "domain\tstage_id\ttool_id\tcorpus_id\tasset_profile_id\tadapter_id\tparser_id\tschema_id\tparser_fixture_parser_id\tparser_fixture_schema_id\tparser_fixture_reference_kind\tparser_fixture_reference\tproof_source\tcoverage_status\treason\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.domain),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.corpus_id),
            sanitize_tsv(&row.asset_profile_id),
            sanitize_tsv(&row.adapter_id),
            sanitize_tsv(&row.parser_id),
            sanitize_tsv(&row.schema_id),
            sanitize_tsv(&row.parser_fixture_parser_id),
            sanitize_tsv(&row.parser_fixture_schema_id),
            sanitize_tsv(&row.parser_fixture_reference_kind),
            sanitize_tsv(&row.parser_fixture_reference),
            sanitize_tsv(&row.proof_source),
            sanitize_tsv(&row.coverage_status),
            sanitize_tsv(&row.reason),
        ));
    }
    rendered
}

fn coverage_key_from_active_row(row: &AllDomainActiveStageToolMatrixRow) -> CoverageKey {
    CoverageKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
    }
}

fn coverage_key_from_report_row(row: &AllDomainParserFixtureCoverageRow) -> CoverageKey {
    CoverageKey {
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
    }
}

fn format_coverage_key(key: &CoverageKey) -> String {
    format!("{}/{}/{}", key.domain, key.stage_id, key.tool_id)
}

fn parser_proof_source_for_domain(domain: &str) -> Result<&'static str> {
    match domain {
        "fastq" => Ok(PROOF_SOURCE_FASTQ),
        "bam" => Ok(PROOF_SOURCE_BAM),
        "vcf" => Ok(PROOF_SOURCE_VCF),
        _ => {
            Err(anyhow!("all-domain parser fixture coverage does not recognize domain `{domain}`"))
        }
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

fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_all_domain_parser_fixture_coverage,
        ALL_DOMAIN_PARSER_FIXTURE_COVERAGE_SCHEMA_VERSION,
        DEFAULT_ALL_DOMAIN_PARSER_FIXTURE_COVERAGE_PATH,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn render_all_domain_parser_fixture_coverage_reports_complete_active_scope_proof() {
        let root = repo_root();
        let report = render_all_domain_parser_fixture_coverage(
            &root,
            PathBuf::from(DEFAULT_ALL_DOMAIN_PARSER_FIXTURE_COVERAGE_PATH),
        )
        .expect("render all-domain parser fixture coverage");

        assert_eq!(report.schema_version, ALL_DOMAIN_PARSER_FIXTURE_COVERAGE_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_ALL_DOMAIN_PARSER_FIXTURE_COVERAGE_PATH);
        assert_eq!(report.stage_count, 67);
        assert_eq!(report.tool_count, 71);
        assert_eq!(report.parser_proof_binding_count, report.row_count);
        assert_eq!(report.covered_row_count, report.row_count);
        assert_eq!(report.missing_row_count, 0);
        assert_eq!(report.coverage_percent, 100.0);
        assert_eq!(report.domain_counts.get("fastq"), Some(&69));
        assert_eq!(report.domain_counts.get("bam"), Some(&49));
        assert_eq!(report.domain_counts.get("vcf"), Some(&20));
        assert_eq!(report.proof_source_counts.get("fastq_parser_fixture_coverage"), Some(&69));
        assert_eq!(report.proof_source_counts.get("bam_parser_fixture_coverage"), Some(&49));
        assert_eq!(report.proof_source_counts.get("vcf_parser_fixture_coverage"), Some(&20));
        assert_eq!(report.proof_source_counts.values().copied().sum::<usize>(), report.row_count);
        assert_eq!(report.coverage_status_counts.get("covered"), Some(&report.row_count));
        assert_eq!(report.violation_count, 0);
        assert!(report.ok);
        assert!(report.violations.is_empty());
        assert!(report.rows.iter().all(|row| row.coverage_status == "covered"));
        assert!(report.rows.iter().all(|row| row.parser_fixture_reference_kind != "none"));

        assert!(report.rows.iter().any(|row| {
            row.domain == "fastq"
                && row.stage_id == "fastq.trim_reads"
                && row.tool_id == "trimmomatic"
                && row.parser_fixture_reference_kind == "fixture_case"
                && row.parser_fixture_reference == "fastq.trim_reads.report_json"
                && row.parser_fixture_parser_id == "parse_trim_reads_report"
                && row.proof_source == "fastq_parser_fixture_coverage"
        }));
        assert!(report.rows.iter().any(|row| {
            row.domain == "bam"
                && row.stage_id == "bam.contamination"
                && row.tool_id == "schmutzi"
                && row.parser_fixture_reference_kind == "fixture_corpus"
                && row.parser_fixture_reference == "fixture:corpus-01-adna-bam-mini"
                && row.proof_source == "bam_parser_fixture_coverage"
        }));
        assert!(report.rows.iter().any(|row| {
            row.domain == "vcf"
                && row.stage_id == "vcf.postprocess"
                && row.tool_id == "bcftools"
                && row.parser_fixture_parser_id == "parse_bcftools_postprocess_metrics"
                && row.parser_fixture_schema_id == "bijux.vcf.postprocess.v1"
                && row.parser_fixture_reference_kind == "fixture_directory"
                && row.parser_fixture_reference.starts_with(
                    "benchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.postprocess",
                )
                && row.proof_source == "vcf_parser_fixture_coverage"
        }));
        assert!(report.rows.iter().any(|row| {
            row.domain == "fastq"
                && row.stage_id == "fastq.index_reference"
                && row.tool_id == "bowtie2_build"
                && row.parser_fixture_reference_kind == "fixture_case"
                && row.parser_fixture_reference == "fastq.index_reference.report_json"
                && row.parser_fixture_parser_id == "parse_index_reference_report"
                && row.proof_source == "fastq_parser_fixture_coverage"
        }));
    }
}
