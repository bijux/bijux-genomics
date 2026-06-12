use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use super::fastq_command_adapter_coverage::collect_fastq_command_adapter_coverage_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_FASTQ_PARSER_COVERAGE_PATH: &str =
    "benchmarks/readiness/fastq-parser-coverage.tsv";
const FASTQ_PARSER_COVERAGE_SCHEMA_VERSION: &str = "bijux.bench.readiness.fastq_parser_coverage.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FastqParserCoverageKind {
    Covered,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct FastqParserCoverageRow {
    pub(crate) tool_id: String,
    pub(crate) stage_id: String,
    pub(crate) parser_coverage: FastqParserCoverageKind,
    pub(crate) parser_status: String,
    pub(crate) support_status: String,
    pub(crate) adapter_status: String,
    pub(crate) corpus_status: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FastqParserCoverageReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) row_count: usize,
    pub(crate) parser_covered_row_count: usize,
    pub(crate) parser_missing_row_count: usize,
    pub(crate) parser_coverage_percent: f64,
    pub(crate) rows: Vec<FastqParserCoverageRow>,
}

pub(crate) fn run_render_fastq_parser_coverage(
    args: &parse::BenchReadinessRenderFastqParserCoverageArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_fastq_parser_coverage(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_FASTQ_PARSER_COVERAGE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_fastq_parser_coverage(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<FastqParserCoverageReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let (stage_count, tool_count, rows) = collect_fastq_parser_coverage_rows(repo_root)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_fastq_parser_coverage_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let parser_covered_row_count =
        rows.iter().filter(|row| row.parser_coverage == FastqParserCoverageKind::Covered).count();
    let parser_missing_row_count =
        rows.iter().filter(|row| row.parser_coverage == FastqParserCoverageKind::Missing).count();
    let parser_coverage_percent = if rows.is_empty() {
        0.0
    } else {
        parser_covered_row_count as f64 * 100.0 / rows.len() as f64
    };

    Ok(FastqParserCoverageReport {
        schema_version: FASTQ_PARSER_COVERAGE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        stage_count,
        tool_count,
        row_count: rows.len(),
        parser_covered_row_count,
        parser_missing_row_count,
        parser_coverage_percent,
        rows,
    })
}

pub(crate) fn collect_fastq_parser_coverage_rows(
    repo_root: &Path,
) -> Result<(usize, usize, Vec<FastqParserCoverageRow>)> {
    let (stage_count, tool_count, rows) = collect_fastq_command_adapter_coverage_rows(repo_root)?;
    let rows = rows
        .into_iter()
        .filter(row_requires_parser)
        .map(render_parser_coverage_row)
        .collect::<Vec<_>>();
    Ok((stage_count, tool_count, rows))
}

fn row_requires_parser(
    row: &super::fastq_command_adapter_coverage::FastqCommandAdapterCoverageRow,
) -> bool {
    row_has_governed_support(&row.support_status)
        && row_has_adapter(&row.adapter_status)
        && row_has_fixture_corpus(&row.corpus_status)
}

fn render_parser_coverage_row(
    row: super::fastq_command_adapter_coverage::FastqCommandAdapterCoverageRow,
) -> FastqParserCoverageRow {
    let parser_coverage = if row_has_normalized_parser(&row.parser_status) {
        FastqParserCoverageKind::Covered
    } else {
        FastqParserCoverageKind::Missing
    };
    let reason = match parser_coverage {
        FastqParserCoverageKind::Covered => format!(
            "row `{}` / `{}` has governed support, adapter-backed command rendering, fixture-backed corpus coverage, and normalized parser output",
            row.stage_id, row.tool_id
        ),
        FastqParserCoverageKind::Missing => format!(
            "row `{}` / `{}` has governed support, adapter-backed command rendering, and fixture-backed corpus coverage but no normalized parser contract (`{}`)",
            row.stage_id, row.tool_id, row.parser_status
        ),
    };

    FastqParserCoverageRow {
        tool_id: row.tool_id,
        stage_id: row.stage_id,
        parser_coverage,
        parser_status: row.parser_status,
        support_status: row.support_status,
        adapter_status: row.adapter_status,
        corpus_status: row.corpus_status,
        reason,
    }
}

fn row_has_governed_support(value: &str) -> bool {
    matches!(
        value,
        "governed_execution" | "governed_benchmark_cohort" | "observer_specialized_benchmark"
    )
}

fn row_has_adapter(value: &str) -> bool {
    matches!(value, "runnable" | "plannable")
}

fn row_has_normalized_parser(value: &str) -> bool {
    value != "not_normalized"
}

fn row_has_fixture_corpus(value: &str) -> bool {
    value.starts_with("fixture:")
}

fn render_fastq_parser_coverage_tsv(rows: &[FastqParserCoverageRow]) -> String {
    let mut rendered = String::from(
        "tool_id\tstage_id\tparser_coverage\tparser_status\tsupport_status\tadapter_status\tcorpus_status\treason\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(parser_coverage_label(row.parser_coverage)),
            sanitize_tsv(&row.parser_status),
            sanitize_tsv(&row.support_status),
            sanitize_tsv(&row.adapter_status),
            sanitize_tsv(&row.corpus_status),
            sanitize_tsv(&row.reason),
        ));
    }
    rendered
}

fn parser_coverage_label(status: FastqParserCoverageKind) -> &'static str {
    match status {
        FastqParserCoverageKind::Covered => "covered",
        FastqParserCoverageKind::Missing => "missing",
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
        render_fastq_parser_coverage, DEFAULT_FASTQ_PARSER_COVERAGE_PATH,
        FASTQ_PARSER_COVERAGE_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn render_fastq_parser_coverage_reports_governed_parser_rows() {
        let root = repo_root();
        let report =
            render_fastq_parser_coverage(&root, PathBuf::from(DEFAULT_FASTQ_PARSER_COVERAGE_PATH))
                .expect("render FASTQ parser coverage");

        assert_eq!(report.schema_version, FASTQ_PARSER_COVERAGE_SCHEMA_VERSION);
        assert_eq!(report.stage_count, 27);
        assert_eq!(report.tool_count, 44);
        assert_eq!(report.row_count, 66);
        assert_eq!(report.parser_covered_row_count, 66);
        assert_eq!(report.parser_missing_row_count, 0);
        assert_eq!(report.parser_coverage_percent, 100.0);
        assert!(report.rows.iter().all(|row| {
            super::parser_coverage_label(row.parser_coverage) == "covered"
                && row.corpus_status.starts_with("fixture:")
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "fastqc"
                && row.stage_id == "fastq.validate_reads"
                && super::parser_coverage_label(row.parser_coverage) == "covered"
                && row.parser_status == "comparable"
                && row.corpus_status == "fixture:corpus-01-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "bijux_dna"
                && row.stage_id == "fastq.detect_duplicates_premerge"
                && super::parser_coverage_label(row.parser_coverage) == "covered"
                && row.parser_status == "parse_normalized"
                && row.corpus_status == "fixture:corpus-01-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "kraken2"
                && row.stage_id == "fastq.screen_taxonomy"
                && super::parser_coverage_label(row.parser_coverage) == "covered"
                && row.parser_status == "benchmark_normalized"
                && row.corpus_status == "fixture:corpus-02-edna-mini"
        }));
    }

    #[test]
    fn render_fastq_parser_coverage_writes_governed_tsv_columns() {
        let root = repo_root();
        let output_path = PathBuf::from(DEFAULT_FASTQ_PARSER_COVERAGE_PATH);
        let report =
            render_fastq_parser_coverage(&root, output_path).expect("render parser coverage");
        let rendered = std::fs::read_to_string(root.join(&report.output_path))
            .expect("read rendered fastq parser coverage tsv");
        let rows = rendered.lines().collect::<Vec<_>>();

        assert_eq!(
            rows.first().copied(),
            Some(
                "tool_id\tstage_id\tparser_coverage\tparser_status\tsupport_status\tadapter_status\tcorpus_status\treason"
            )
        );
        assert!(
            rows.iter().any(|row| {
                row.starts_with(
                    "fastqc\tfastq.validate_reads\tcovered\tcomparable\tobserver_specialized_benchmark\trunnable\tfixture:corpus-01-mini\t"
                )
            }),
            "the governed FASTQ validation row must retain parser coverage"
        );
        assert!(
            rows.iter().any(|row| {
                row.starts_with(
                    "bijux_dna\tfastq.detect_duplicates_premerge\tcovered\tparse_normalized\tgoverned_execution\trunnable\tfixture:corpus-01-mini\t"
                )
            }),
            "the governed duplicate-signal row must retain parser coverage"
        );
        assert!(
            rows.iter().any(|row| {
                row.starts_with(
                    "kraken2\tfastq.screen_taxonomy\tcovered\tbenchmark_normalized\tgoverned_benchmark_cohort\trunnable\tfixture:corpus-02-edna-mini\t"
                )
            }),
            "the governed taxonomy row must retain parser coverage"
        );
    }
}
