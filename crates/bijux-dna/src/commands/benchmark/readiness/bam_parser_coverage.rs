use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use super::bam_command_adapter_coverage::{
    collect_bam_command_adapter_coverage_rows, BamBenchmarkStatus, BamReadinessGapKind,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_BAM_PARSER_COVERAGE_PATH: &str =
    "benchmarks/readiness/bam-parser-coverage.tsv";
const BAM_PARSER_COVERAGE_SCHEMA_VERSION: &str = "bijux.bench.readiness.bam_parser_coverage.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum BamParserCoverageKind {
    Covered,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BamParserCoverageRow {
    pub(crate) tool_id: String,
    pub(crate) stage_id: String,
    pub(crate) parser_coverage: BamParserCoverageKind,
    pub(crate) parser_status: String,
    pub(crate) support_status: String,
    pub(crate) adapter_status: String,
    pub(crate) corpus_status: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamParserCoverageReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) row_count: usize,
    pub(crate) parser_covered_row_count: usize,
    pub(crate) parser_missing_row_count: usize,
    pub(crate) parser_coverage_percent: f64,
    pub(crate) excluded_non_benchmark_ready_row_count: usize,
    pub(crate) excluded_readiness_gap_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<BamParserCoverageRow>,
}

pub(crate) fn run_render_bam_parser_coverage(
    args: &parse::BenchReadinessRenderBamParserCoverageArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_parser_coverage(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_PARSER_COVERAGE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_bam_parser_coverage(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamParserCoverageReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let (stage_count, tool_count, rows, excluded_readiness_gap_counts) =
        collect_bam_parser_coverage_rows(repo_root)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_bam_parser_coverage_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let parser_covered_row_count =
        rows.iter().filter(|row| row.parser_coverage == BamParserCoverageKind::Covered).count();
    let parser_missing_row_count =
        rows.iter().filter(|row| row.parser_coverage == BamParserCoverageKind::Missing).count();
    let parser_coverage_percent = if rows.is_empty() {
        0.0
    } else {
        parser_covered_row_count as f64 * 100.0 / rows.len() as f64
    };
    let excluded_non_benchmark_ready_row_count = excluded_readiness_gap_counts.values().sum();

    Ok(BamParserCoverageReport {
        schema_version: BAM_PARSER_COVERAGE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        stage_count,
        tool_count,
        row_count: rows.len(),
        parser_covered_row_count,
        parser_missing_row_count,
        parser_coverage_percent,
        excluded_non_benchmark_ready_row_count,
        excluded_readiness_gap_counts,
        rows,
    })
}

fn collect_bam_parser_coverage_rows(
    repo_root: &Path,
) -> Result<(usize, usize, Vec<BamParserCoverageRow>, BTreeMap<String, usize>)> {
    let (stage_count, tool_count, rows) = collect_bam_command_adapter_coverage_rows(repo_root)?;
    let mut excluded_readiness_gap_counts = BTreeMap::<String, usize>::new();
    let mut parser_rows = Vec::new();

    for row in rows {
        if row.benchmark_status == BamBenchmarkStatus::BenchmarkReady {
            parser_rows.push(render_parser_coverage_row(row));
        } else {
            *excluded_readiness_gap_counts
                .entry(readiness_gap_label(row.readiness_gap).to_string())
                .or_default() += 1;
        }
    }

    Ok((stage_count, tool_count, parser_rows, excluded_readiness_gap_counts))
}

fn render_parser_coverage_row(
    row: super::bam_command_adapter_coverage::BamCommandAdapterCoverageRow,
) -> BamParserCoverageRow {
    let parser_coverage = if row.parser_status == "parser_fixture_validated" {
        BamParserCoverageKind::Covered
    } else {
        BamParserCoverageKind::Missing
    };
    let reason = match parser_coverage {
        BamParserCoverageKind::Covered => format!(
            "row `{}` / `{}` is benchmark_ready with governed support, adapter-backed command rendering, fixture-backed corpus coverage, and parser-fixture-validated output",
            row.stage_id, row.tool_id
        ),
        BamParserCoverageKind::Missing => format!(
            "row `{}` / `{}` is benchmark_ready but lacks parser-fixture-validated BAM normalization (`{}`)",
            row.stage_id, row.tool_id, row.parser_status
        ),
    };

    BamParserCoverageRow {
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

fn render_bam_parser_coverage_tsv(rows: &[BamParserCoverageRow]) -> String {
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

fn parser_coverage_label(value: BamParserCoverageKind) -> &'static str {
    match value {
        BamParserCoverageKind::Covered => "covered",
        BamParserCoverageKind::Missing => "missing",
    }
}

fn readiness_gap_label(value: BamReadinessGapKind) -> &'static str {
    match value {
        BamReadinessGapKind::None => "none",
        BamReadinessGapKind::Corpus => "corpus",
        BamReadinessGapKind::Parser => "parser",
        BamReadinessGapKind::Adapter => "adapter",
        BamReadinessGapKind::Support => "support",
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
        render_bam_parser_coverage, BAM_PARSER_COVERAGE_SCHEMA_VERSION,
        DEFAULT_BAM_PARSER_COVERAGE_PATH,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn render_bam_parser_coverage_reports_governed_parser_rows() {
        let root = repo_root();
        let report =
            render_bam_parser_coverage(&root, PathBuf::from(DEFAULT_BAM_PARSER_COVERAGE_PATH))
                .expect("render BAM parser coverage");

        assert_eq!(report.schema_version, BAM_PARSER_COVERAGE_SCHEMA_VERSION);
        assert_eq!(report.stage_count, 24);
        assert_eq!(report.tool_count, 25);
        assert_eq!(report.row_count, 49);
        assert_eq!(report.parser_covered_row_count, 49);
        assert_eq!(report.parser_missing_row_count, 0);
        assert_eq!(report.parser_coverage_percent, 100.0);
        assert_eq!(report.excluded_non_benchmark_ready_row_count, 0);
        assert!(report.excluded_readiness_gap_counts.is_empty());
        assert!(report.rows.iter().all(|row| {
            super::parser_coverage_label(row.parser_coverage) == "covered"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status.starts_with("fixture:")
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "samtools"
                && row.stage_id == "bam.validate"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "mapdamage2"
                && row.stage_id == "bam.damage"
                && row.corpus_status == "fixture:corpus-01-adna-damage-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "angsd"
                && row.stage_id == "bam.genotyping"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "bwa"
                && row.stage_id == "bam.align"
                && row.corpus_status == "fixture:corpus-01-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "bowtie2"
                && row.stage_id == "bam.align"
                && row.corpus_status == "fixture:corpus-01-mini"
        }));
    }

    #[test]
    fn render_bam_parser_coverage_writes_governed_tsv_columns() {
        let root = repo_root();
        let output_path = PathBuf::from(DEFAULT_BAM_PARSER_COVERAGE_PATH);
        let report =
            render_bam_parser_coverage(&root, output_path).expect("render BAM parser coverage");
        let rendered = std::fs::read_to_string(root.join(&report.output_path))
            .expect("read rendered bam parser coverage tsv");
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
                    "samtools\tbam.validate\tcovered\tparser_fixture_validated\tsupported\trunnable\tfixture:corpus-01-bam-mini\t"
                )
            }),
            "the governed BAM validation row must retain parser coverage"
        );
        assert!(
            rows.iter().any(|row| {
                row.starts_with(
                    "mapdamage2\tbam.damage\tcovered\tparser_fixture_validated\tsupported\trunnable\tfixture:corpus-01-adna-damage-mini\t"
                )
            }),
            "the governed BAM damage row must retain parser coverage"
        );
        assert!(
            rows.iter().any(|row| {
                row.starts_with(
                    "angsd\tbam.genotyping\tcovered\tparser_fixture_validated\tsupported\trunnable\tfixture:corpus-01-genotyping-mini\t"
                )
            }),
            "the governed BAM genotyping row must retain parser coverage"
        );
    }
}
