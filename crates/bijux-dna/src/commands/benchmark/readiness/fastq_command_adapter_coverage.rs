use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use super::tool_serving_map::{
    render_fastq_tool_serving_map, ToolServingMapRow, DEFAULT_FASTQ_TOOL_SERVING_MAP_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_FASTQ_COMMAND_ADAPTER_COVERAGE_PATH: &str =
    "benchmarks/readiness/fastq-command-adapter-coverage.tsv";
const FASTQ_COMMAND_ADAPTER_COVERAGE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.fastq_command_adapter_coverage.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FastqBenchmarkStatus {
    BenchmarkReady,
    NotBenchmarkReady,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FastqAdapterCoverageKind {
    Covered,
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FastqReadinessGapKind {
    None,
    Corpus,
    Parser,
    Adapter,
    Support,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct FastqCommandAdapterCoverageRow {
    pub(crate) tool_id: String,
    pub(crate) stage_id: String,
    pub(crate) benchmark_status: FastqBenchmarkStatus,
    pub(crate) adapter_coverage: FastqAdapterCoverageKind,
    pub(crate) readiness_gap: FastqReadinessGapKind,
    pub(crate) support_status: String,
    pub(crate) adapter_status: String,
    pub(crate) parser_status: String,
    pub(crate) corpus_status: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FastqCommandAdapterCoverageReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) row_count: usize,
    pub(crate) benchmark_ready_row_count: usize,
    pub(crate) benchmark_ready_adapter_covered_row_count: usize,
    pub(crate) benchmark_ready_adapter_missing_row_count: usize,
    pub(crate) readiness_gap_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<FastqCommandAdapterCoverageRow>,
}

pub(crate) fn run_render_fastq_command_adapter_coverage(
    args: &parse::BenchReadinessRenderFastqCommandAdapterCoverageArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_fastq_command_adapter_coverage(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_FASTQ_COMMAND_ADAPTER_COVERAGE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_fastq_command_adapter_coverage(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<FastqCommandAdapterCoverageReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let (stage_count, tool_count, rows) = collect_fastq_command_adapter_coverage_rows(repo_root)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_fastq_command_adapter_coverage_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let benchmark_ready_row_count = rows
        .iter()
        .filter(|row| row.benchmark_status == FastqBenchmarkStatus::BenchmarkReady)
        .count();
    let benchmark_ready_adapter_covered_row_count = rows
        .iter()
        .filter(|row| {
            row.benchmark_status == FastqBenchmarkStatus::BenchmarkReady
                && row.adapter_coverage == FastqAdapterCoverageKind::Covered
        })
        .count();
    let benchmark_ready_adapter_missing_row_count = rows
        .iter()
        .filter(|row| {
            row.benchmark_status == FastqBenchmarkStatus::BenchmarkReady
                && row.adapter_coverage == FastqAdapterCoverageKind::Missing
        })
        .count();

    let mut readiness_gap_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        if row.readiness_gap != FastqReadinessGapKind::None {
            *readiness_gap_counts
                .entry(readiness_gap_label(row.readiness_gap).to_string())
                .or_default() += 1;
        }
    }

    Ok(FastqCommandAdapterCoverageReport {
        schema_version: FASTQ_COMMAND_ADAPTER_COVERAGE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        stage_count,
        tool_count,
        row_count: rows.len(),
        benchmark_ready_row_count,
        benchmark_ready_adapter_covered_row_count,
        benchmark_ready_adapter_missing_row_count,
        readiness_gap_counts,
        rows,
    })
}

pub(crate) fn collect_fastq_command_adapter_coverage_rows(
    repo_root: &Path,
) -> Result<(usize, usize, Vec<FastqCommandAdapterCoverageRow>)> {
    let tool_map = render_fastq_tool_serving_map(
        repo_root,
        PathBuf::from(DEFAULT_FASTQ_TOOL_SERVING_MAP_PATH),
    )?;
    let rows = tool_map.rows.iter().map(render_coverage_row).collect::<Vec<_>>();
    Ok((tool_map.stage_count, tool_map.tool_count, rows))
}

fn render_coverage_row(row: &ToolServingMapRow) -> FastqCommandAdapterCoverageRow {
    let support_eligible = row_has_governed_support(row);
    let adapter_covered = row_has_adapter(row);
    let parser_eligible = row_has_normalized_parser(row);
    let fixture_backed = row_has_fixture_corpus(row);
    let readiness_gap = if support_eligible && adapter_covered && parser_eligible && fixture_backed
    {
        FastqReadinessGapKind::None
    } else if support_eligible && adapter_covered && parser_eligible {
        FastqReadinessGapKind::Corpus
    } else if support_eligible && adapter_covered {
        FastqReadinessGapKind::Parser
    } else if support_eligible {
        FastqReadinessGapKind::Adapter
    } else {
        FastqReadinessGapKind::Support
    };
    let benchmark_status = if readiness_gap == FastqReadinessGapKind::None {
        FastqBenchmarkStatus::BenchmarkReady
    } else {
        FastqBenchmarkStatus::NotBenchmarkReady
    };
    let adapter_coverage = if adapter_covered {
        FastqAdapterCoverageKind::Covered
    } else {
        FastqAdapterCoverageKind::Missing
    };
    let reason = match readiness_gap {
        FastqReadinessGapKind::None => format!(
            "row `{}` / `{}` is benchmark_ready with governed support, adapter-backed command rendering, normalized parser output, and fixture-backed corpus coverage",
            row.stage_id, row.tool_id
        ),
        FastqReadinessGapKind::Corpus => format!(
            "row `{}` / `{}` has governed support, adapter-backed command rendering, and normalized parser output but still resolves only `{}` corpus coverage",
            row.stage_id, row.tool_id, row.corpus_status
        ),
        FastqReadinessGapKind::Parser => format!(
            "row `{}` / `{}` has governed support and adapter-backed command rendering but no normalized parser contract (`{}`)",
            row.stage_id, row.tool_id, row.parser_status
        ),
        FastqReadinessGapKind::Adapter => format!(
            "row `{}` / `{}` has governed benchmark support but no runnable or plannable command adapter (`{}`)",
            row.stage_id, row.tool_id, row.adapter_status
        ),
        FastqReadinessGapKind::Support => format!(
            "row `{}` / `{}` is not yet in governed benchmark support for HPC readiness (`{}`)",
            row.stage_id, row.tool_id, row.support_status
        ),
    };

    FastqCommandAdapterCoverageRow {
        tool_id: row.tool_id.clone(),
        stage_id: row.stage_id.clone(),
        benchmark_status,
        adapter_coverage,
        readiness_gap,
        support_status: row.support_status.clone(),
        adapter_status: row.adapter_status.clone(),
        parser_status: row.parser_status.clone(),
        corpus_status: row.corpus_status.clone(),
        reason,
    }
}

fn row_has_governed_support(row: &ToolServingMapRow) -> bool {
    matches!(
        row.support_status.as_str(),
        "governed_execution" | "governed_benchmark_cohort" | "observer_specialized_benchmark"
    )
}

fn row_has_adapter(row: &ToolServingMapRow) -> bool {
    matches!(row.adapter_status.as_str(), "runnable" | "plannable")
}

fn row_has_normalized_parser(row: &ToolServingMapRow) -> bool {
    row.parser_status != "not_normalized"
}

fn row_has_fixture_corpus(row: &ToolServingMapRow) -> bool {
    row.corpus_status.starts_with("fixture:")
}

fn render_fastq_command_adapter_coverage_tsv(rows: &[FastqCommandAdapterCoverageRow]) -> String {
    let mut rendered = String::from(
        "tool_id\tstage_id\tbenchmark_status\tadapter_coverage\treadiness_gap\tsupport_status\tadapter_status\tparser_status\tcorpus_status\treason\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(benchmark_status_label(row.benchmark_status)),
            sanitize_tsv(adapter_coverage_label(row.adapter_coverage)),
            sanitize_tsv(readiness_gap_label(row.readiness_gap)),
            sanitize_tsv(&row.support_status),
            sanitize_tsv(&row.adapter_status),
            sanitize_tsv(&row.parser_status),
            sanitize_tsv(&row.corpus_status),
            sanitize_tsv(&row.reason),
        ));
    }
    rendered
}

fn benchmark_status_label(status: FastqBenchmarkStatus) -> &'static str {
    match status {
        FastqBenchmarkStatus::BenchmarkReady => "benchmark_ready",
        FastqBenchmarkStatus::NotBenchmarkReady => "not_benchmark_ready",
    }
}

fn adapter_coverage_label(coverage: FastqAdapterCoverageKind) -> &'static str {
    match coverage {
        FastqAdapterCoverageKind::Covered => "covered",
        FastqAdapterCoverageKind::Missing => "missing",
    }
}

fn readiness_gap_label(gap: FastqReadinessGapKind) -> &'static str {
    match gap {
        FastqReadinessGapKind::None => "none",
        FastqReadinessGapKind::Corpus => "corpus",
        FastqReadinessGapKind::Parser => "parser",
        FastqReadinessGapKind::Adapter => "adapter",
        FastqReadinessGapKind::Support => "support",
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
        render_fastq_command_adapter_coverage, DEFAULT_FASTQ_COMMAND_ADAPTER_COVERAGE_PATH,
        FASTQ_COMMAND_ADAPTER_COVERAGE_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn fastq_command_adapter_coverage_reports_governed_row_readiness() {
        let root = repo_root();
        let report = render_fastq_command_adapter_coverage(
            &root,
            PathBuf::from(DEFAULT_FASTQ_COMMAND_ADAPTER_COVERAGE_PATH),
        )
        .expect("render FASTQ command adapter coverage");

        assert_eq!(report.schema_version, FASTQ_COMMAND_ADAPTER_COVERAGE_SCHEMA_VERSION);
        assert_eq!(report.stage_count, 27);
        assert_eq!(report.tool_count, 44);
        assert_eq!(report.row_count, 74);
        assert_eq!(report.benchmark_ready_row_count, 66);
        assert_eq!(report.benchmark_ready_adapter_covered_row_count, 66);
        assert_eq!(report.benchmark_ready_adapter_missing_row_count, 0);
        assert_eq!(report.readiness_gap_counts.get("corpus"), Some(&3));
        assert_eq!(report.readiness_gap_counts.get("support"), Some(&5));
        assert!(
            report.readiness_gap_counts.get("adapter").is_none(),
            "the governed FASTQ readiness slice currently carries no adapter gap rows"
        );
        assert!(
            report.readiness_gap_counts.get("parser").is_none(),
            "the governed FASTQ readiness slice currently carries no parser gap rows"
        );
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "fastqc"
                && row.stage_id == "fastq.validate_reads"
                && super::benchmark_status_label(row.benchmark_status) == "benchmark_ready"
                && super::adapter_coverage_label(row.adapter_coverage) == "covered"
                && super::readiness_gap_label(row.readiness_gap) == "none"
                && row.corpus_status == "fixture:corpus-01-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "kraken2"
                && row.stage_id == "fastq.screen_taxonomy"
                && super::benchmark_status_label(row.benchmark_status) == "benchmark_ready"
                && row.corpus_status == "fixture:corpus-02-edna-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "fastqc"
                && row.stage_id == "fastq.detect_adapters"
                && super::benchmark_status_label(row.benchmark_status) == "benchmark_ready"
                && super::adapter_coverage_label(row.adapter_coverage) == "covered"
                && super::readiness_gap_label(row.readiness_gap) == "none"
                && row.corpus_status == "fixture:corpus-01-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "fastp"
                && row.stage_id == "fastq.filter_reads"
                && super::benchmark_status_label(row.benchmark_status) == "benchmark_ready"
                && super::adapter_coverage_label(row.adapter_coverage) == "covered"
                && super::readiness_gap_label(row.readiness_gap) == "none"
                && row.corpus_status == "fixture:corpus-01-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "fastp"
                && row.stage_id == "fastq.trim_polyg_tails"
                && super::benchmark_status_label(row.benchmark_status) == "benchmark_ready"
                && super::adapter_coverage_label(row.adapter_coverage) == "covered"
                && super::readiness_gap_label(row.readiness_gap) == "none"
                && row.corpus_status == "fixture:corpus-01-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "cutadapt"
                && row.stage_id == "fastq.trim_terminal_damage"
                && super::benchmark_status_label(row.benchmark_status) == "benchmark_ready"
                && super::adapter_coverage_label(row.adapter_coverage) == "covered"
                && super::readiness_gap_label(row.readiness_gap) == "none"
                && row.corpus_status == "fixture:corpus-01-mini"
        }));
        for tool_id in ["fastq_scan", "fastqc", "seqkit"] {
            assert!(report.rows.iter().any(|row| {
                row.tool_id == tool_id
                    && row.stage_id == "fastq.profile_overrepresented_sequences"
                    && super::benchmark_status_label(row.benchmark_status) == "benchmark_ready"
                    && super::adapter_coverage_label(row.adapter_coverage) == "covered"
                    && super::readiness_gap_label(row.readiness_gap) == "none"
                    && row.corpus_status == "fixture:corpus-01-mini"
            }));
        }
        for tool_id in ["centrifuge", "kaiju", "kraken2", "krakenuniq"] {
            assert!(report.rows.iter().any(|row| {
                row.tool_id == tool_id
                    && row.stage_id == "fastq.screen_taxonomy"
                    && super::benchmark_status_label(row.benchmark_status) == "benchmark_ready"
                    && super::adapter_coverage_label(row.adapter_coverage) == "covered"
                    && super::readiness_gap_label(row.readiness_gap) == "none"
                    && row.corpus_status == "fixture:corpus-02-edna-mini"
            }));
        }
    }

    #[test]
    fn fastq_command_adapter_coverage_writes_governed_tsv_columns() {
        let root = repo_root();
        let output_path = PathBuf::from(DEFAULT_FASTQ_COMMAND_ADAPTER_COVERAGE_PATH);
        let report =
            render_fastq_command_adapter_coverage(&root, output_path).expect("render coverage");
        let rendered = std::fs::read_to_string(root.join(&report.output_path))
            .expect("read rendered fastq command adapter coverage tsv");
        let rows = rendered.lines().collect::<Vec<_>>();

        assert_eq!(
            rows.first().copied(),
            Some(
                "tool_id\tstage_id\tbenchmark_status\tadapter_coverage\treadiness_gap\tsupport_status\tadapter_status\tparser_status\tcorpus_status\treason"
            )
        );
        assert!(
            rows.iter().any(|row| {
                row.starts_with(
                    "fastqc\tfastq.validate_reads\tbenchmark_ready\tcovered\tnone\tobserver_specialized_benchmark\trunnable\tcomparable\tfixture:corpus-01-mini\t"
                )
            }),
            "the governed FASTQ validation row must remain benchmark-ready and adapter-covered"
        );
        assert!(
            rows.iter().any(|row| {
                row.starts_with(
                    "fastp\tfastq.trim_polyg_tails\tbenchmark_ready\tcovered\tnone\tgoverned_benchmark_cohort\trunnable\tbenchmark_normalized\tfixture:corpus-01-mini\t"
                )
            }),
            "the governed trim-polyg row must remain benchmark-ready and adapter-covered"
        );
        assert!(
            rows.iter().any(|row| {
                row.starts_with(
                    "cutadapt\tfastq.trim_terminal_damage\tbenchmark_ready\tcovered\tnone\tgoverned_benchmark_cohort\trunnable\tbenchmark_normalized\tfixture:corpus-01-mini\t"
                )
            }),
            "the governed trim-terminal-damage row must remain benchmark-ready and adapter-covered"
        );
        for tool_id in ["fastq_scan", "fastqc", "seqkit"] {
            assert!(
                rows.iter().any(|row| {
                    row.starts_with(&format!(
                        "{tool_id}\tfastq.profile_overrepresented_sequences\tbenchmark_ready\tcovered\tnone\tobserver_specialized_benchmark\trunnable\tcomparable\tfixture:corpus-01-mini\t"
                    ))
                }),
                "the governed overrepresented-sequence row must remain benchmark-ready and adapter-covered for {tool_id}"
            );
        }
        for tool_id in ["centrifuge", "kaiju", "kraken2", "krakenuniq"] {
            assert!(
                rows.iter().any(|row| {
                    row.starts_with(&format!(
                        "{tool_id}\tfastq.screen_taxonomy\tbenchmark_ready\tcovered\tnone\tgoverned_benchmark_cohort\trunnable\tbenchmark_normalized\tfixture:corpus-02-edna-mini\t"
                    ))
                }),
                "the governed taxonomy row must remain benchmark-ready and adapter-covered for {tool_id}"
            );
        }
    }
}
