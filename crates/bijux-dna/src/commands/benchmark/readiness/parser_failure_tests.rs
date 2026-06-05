use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_dna_domain_bam::metrics::{
    evaluate_bam_raw_parser_failure_contracts, BamRawParserFailureClass,
};
use bijux_dna_domain_fastq::{
    evaluate_fastq_raw_parser_failure_contracts, FastqRawParserFailureClass,
};
use serde::Serialize;

use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_PARSER_FAILURE_TESTS_PATH: &str =
    "target/bench-readiness/parser-failure-tests.json";
const PARSER_FAILURE_TESTS_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.parser_failure_tests.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ParserFailureTestRow {
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) parser_id: String,
    pub(crate) expected_failure_class: String,
    pub(crate) observed_failure_class: String,
    pub(crate) observed_error: String,
    pub(crate) passed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ParserFailureTestsReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) passed_row_count: usize,
    pub(crate) failed_row_count: usize,
    pub(crate) domain_row_counts: BTreeMap<String, usize>,
    pub(crate) expected_failure_class_counts: BTreeMap<String, usize>,
    pub(crate) observed_failure_class_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<ParserFailureTestRow>,
}

pub(crate) fn run_render_parser_failure_tests(
    args: &parse::BenchReadinessRenderParserFailureTestsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_parser_failure_tests(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_PARSER_FAILURE_TESTS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_parser_failure_tests(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<ParserFailureTestsReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_parser_failure_test_rows(repo_root)?;
    let passed_row_count = rows.iter().filter(|row| row.passed).count();
    let failed_row_count = rows.len().saturating_sub(passed_row_count);
    let mut domain_row_counts = BTreeMap::<String, usize>::new();
    let mut expected_failure_class_counts = BTreeMap::<String, usize>::new();
    let mut observed_failure_class_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_row_counts.entry(row.domain.clone()).or_default() += 1;
        *expected_failure_class_counts
            .entry(row.expected_failure_class.clone())
            .or_default() += 1;
        *observed_failure_class_counts
            .entry(row.observed_failure_class.clone())
            .or_default() += 1;
    }

    let report = ParserFailureTestsReport {
        schema_version: PARSER_FAILURE_TESTS_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        passed_row_count,
        failed_row_count,
        domain_row_counts,
        expected_failure_class_counts,
        observed_failure_class_counts,
        rows,
    };
    let payload =
        serde_json::to_string_pretty(&report).context("render parser failure tests to JSON")?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_bytes(&output_path, payload.as_bytes())?;
    Ok(report)
}

fn collect_parser_failure_test_rows(repo_root: &Path) -> Result<Vec<ParserFailureTestRow>> {
    let scratch_root = repo_root.join("artifacts/bench-readiness");
    let mut rows = Vec::new();

    rows.extend(
        evaluate_fastq_raw_parser_failure_contracts(repo_root, &scratch_root)?
            .into_iter()
            .map(|row| ParserFailureTestRow {
                domain: "fastq".to_string(),
                stage_id: row.stage_id,
                tool_id: row.tool_id,
                parser_id: row.parser_id,
                expected_failure_class: fastq_failure_class_label(row.expected_failure_class)
                    .to_string(),
                observed_failure_class: fastq_failure_class_label(row.observed_failure_class)
                    .to_string(),
                observed_error: row.observed_error,
                passed: row.passed,
            }),
    );
    rows.extend(
        evaluate_bam_raw_parser_failure_contracts(repo_root, &scratch_root)?
            .into_iter()
            .map(|row| ParserFailureTestRow {
                domain: "bam".to_string(),
                stage_id: row.stage_id,
                tool_id: row.tool_id,
                parser_id: row.parser_id,
                expected_failure_class: bam_failure_class_label(row.expected_failure_class)
                    .to_string(),
                observed_failure_class: bam_failure_class_label(row.observed_failure_class)
                    .to_string(),
                observed_error: row.observed_error,
                passed: row.passed,
            }),
    );

    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.parser_id.cmp(&right.parser_id))
            .then_with(|| left.expected_failure_class.cmp(&right.expected_failure_class))
    });
    Ok(rows)
}

fn fastq_failure_class_label(value: FastqRawParserFailureClass) -> &'static str {
    match value {
        FastqRawParserFailureClass::MissingRawOutput => "missing_raw_output",
        FastqRawParserFailureClass::EmptyRawOutput => "empty_raw_output",
        FastqRawParserFailureClass::MalformedRawOutput => "malformed_raw_output",
        FastqRawParserFailureClass::UnexpectedSuccess => "unexpected_success",
    }
}

fn bam_failure_class_label(value: BamRawParserFailureClass) -> &'static str {
    match value {
        BamRawParserFailureClass::MissingRawOutput => "missing_raw_output",
        BamRawParserFailureClass::EmptyRawOutput => "empty_raw_output",
        BamRawParserFailureClass::MalformedRawOutput => "malformed_raw_output",
        BamRawParserFailureClass::UnexpectedSuccess => "unexpected_success",
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_parser_failure_tests, DEFAULT_PARSER_FAILURE_TESTS_PATH,
        PARSER_FAILURE_TESTS_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn render_parser_failure_tests_reports_structured_missing_empty_and_malformed_rows() {
        let root = repo_root();
        let report =
            render_parser_failure_tests(&root, PathBuf::from(DEFAULT_PARSER_FAILURE_TESTS_PATH))
                .expect("render parser failure tests");

        assert_eq!(report.schema_version, PARSER_FAILURE_TESTS_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_PARSER_FAILURE_TESTS_PATH);
        assert_eq!(report.row_count, 96);
        assert_eq!(report.passed_row_count, 96);
        assert_eq!(report.failed_row_count, 0);
        assert_eq!(report.domain_row_counts.get("fastq"), Some(&45));
        assert_eq!(report.domain_row_counts.get("bam"), Some(&51));
        assert_eq!(
            report.expected_failure_class_counts.get("missing_raw_output"),
            Some(&32)
        );
        assert_eq!(
            report.expected_failure_class_counts.get("empty_raw_output"),
            Some(&32)
        );
        assert_eq!(
            report.expected_failure_class_counts.get("malformed_raw_output"),
            Some(&32)
        );
        assert!(!report.observed_failure_class_counts.contains_key("unexpected_success"));
        assert!(report.rows.iter().all(|row| row.passed));
        assert!(report.rows.iter().any(|row| {
            row.domain == "fastq"
                && row.stage_id == "fastq.detect_adapters"
                && row.tool_id == "fastqc"
                && row.expected_failure_class == "malformed_raw_output"
                && row.observed_error.contains("fastqc total sequences missing")
        }));
        assert!(report.rows.iter().any(|row| {
            row.domain == "bam"
                && row.stage_id == "bam.qc_pre"
                && row.tool_id == "samtools"
                && row.expected_failure_class == "malformed_raw_output"
                && row.observed_error.contains("flagstat summary missing `in total` line")
        }));
    }
}
