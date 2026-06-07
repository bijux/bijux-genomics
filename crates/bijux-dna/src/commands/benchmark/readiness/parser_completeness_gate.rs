use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use super::bam_command_adapter_coverage::{
    collect_bam_command_adapter_coverage_rows, BamBenchmarkStatus, BamReadinessGapKind,
};
use super::fastq_command_adapter_coverage::{
    collect_fastq_command_adapter_coverage_rows, FastqBenchmarkStatus, FastqReadinessGapKind,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_PARSER_COMPLETENESS_GATE_PATH: &str =
    "target/bench-readiness/gate-parser-complete.json";
const PARSER_COMPLETENESS_GATE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.parser_completeness_gate.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ParserCompletenessGateScope {
    BenchmarkReporting,
    Excluded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ParserCompletenessGateStatus {
    Pass,
    Fail,
    Excluded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ParserCompletenessGateRow {
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) gate_scope: ParserCompletenessGateScope,
    pub(crate) gate_status: ParserCompletenessGateStatus,
    pub(crate) benchmark_status: String,
    pub(crate) readiness_gap: String,
    pub(crate) support_status: String,
    pub(crate) adapter_status: String,
    pub(crate) parser_status: String,
    pub(crate) corpus_status: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ParserCompletenessGateReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) passes_gate: bool,
    pub(crate) row_count: usize,
    pub(crate) benchmark_ready_row_count: usize,
    pub(crate) gate_row_count: usize,
    pub(crate) gate_passed_row_count: usize,
    pub(crate) gate_failed_row_count: usize,
    pub(crate) excluded_row_count: usize,
    pub(crate) domain_stage_counts: BTreeMap<String, usize>,
    pub(crate) domain_tool_counts: BTreeMap<String, usize>,
    pub(crate) domain_row_counts: BTreeMap<String, usize>,
    pub(crate) gate_domain_row_counts: BTreeMap<String, usize>,
    pub(crate) excluded_readiness_gap_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<ParserCompletenessGateRow>,
}

pub(crate) fn run_render_parser_completeness_gate(
    args: &parse::BenchReadinessRenderParserCompletenessGateArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_parser_completeness_gate(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_PARSER_COMPLETENESS_GATE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_parser_completeness_gate(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<ParserCompletenessGateReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let (domain_stage_counts, domain_tool_counts, rows) =
        collect_parser_completeness_gate_rows(repo_root)?;
    let row_count = rows.len();
    let benchmark_ready_row_count =
        rows.iter().filter(|row| row.benchmark_status == "benchmark_ready").count();
    let gate_row_count = rows
        .iter()
        .filter(|row| row.gate_scope == ParserCompletenessGateScope::BenchmarkReporting)
        .count();
    let gate_passed_row_count =
        rows.iter().filter(|row| row.gate_status == ParserCompletenessGateStatus::Pass).count();
    let gate_failed_row_count =
        rows.iter().filter(|row| row.gate_status == ParserCompletenessGateStatus::Fail).count();
    let excluded_row_count =
        rows.iter().filter(|row| row.gate_status == ParserCompletenessGateStatus::Excluded).count();

    let mut domain_row_counts = BTreeMap::<String, usize>::new();
    let mut gate_domain_row_counts = BTreeMap::<String, usize>::new();
    let mut excluded_readiness_gap_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_row_counts.entry(row.domain.clone()).or_default() += 1;
        if row.gate_scope == ParserCompletenessGateScope::BenchmarkReporting {
            *gate_domain_row_counts.entry(row.domain.clone()).or_default() += 1;
        } else if row.readiness_gap != "none" {
            *excluded_readiness_gap_counts.entry(row.readiness_gap.clone()).or_default() += 1;
        }
    }

    let report = ParserCompletenessGateReport {
        schema_version: PARSER_COMPLETENESS_GATE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        passes_gate: gate_failed_row_count == 0,
        row_count,
        benchmark_ready_row_count,
        gate_row_count,
        gate_passed_row_count,
        gate_failed_row_count,
        excluded_row_count,
        domain_stage_counts,
        domain_tool_counts,
        domain_row_counts,
        gate_domain_row_counts,
        excluded_readiness_gap_counts,
        rows,
    };
    let payload =
        serde_json::to_string_pretty(&report).context("render parser completeness gate to JSON")?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_bytes(&output_path, payload.as_bytes())?;
    Ok(report)
}

fn collect_parser_completeness_gate_rows(
    repo_root: &Path,
) -> Result<(BTreeMap<String, usize>, BTreeMap<String, usize>, Vec<ParserCompletenessGateRow>)> {
    let (fastq_stage_count, fastq_tool_count, fastq_rows) =
        collect_fastq_command_adapter_coverage_rows(repo_root)?;
    let (bam_stage_count, bam_tool_count, bam_rows) =
        collect_bam_command_adapter_coverage_rows(repo_root)?;
    let mut domain_stage_counts = BTreeMap::from([
        ("fastq".to_string(), fastq_stage_count),
        ("bam".to_string(), bam_stage_count),
    ]);
    let mut domain_tool_counts = BTreeMap::from([
        ("fastq".to_string(), fastq_tool_count),
        ("bam".to_string(), bam_tool_count),
    ]);
    let mut rows = fastq_rows
        .into_iter()
        .map(render_fastq_row)
        .chain(bam_rows.into_iter().map(render_bam_row))
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    domain_stage_counts.retain(|_, value| *value > 0);
    domain_tool_counts.retain(|_, value| *value > 0);
    Ok((domain_stage_counts, domain_tool_counts, rows))
}

fn render_fastq_row(
    row: super::fastq_command_adapter_coverage::FastqCommandAdapterCoverageRow,
) -> ParserCompletenessGateRow {
    let gate_scope = if fastq_row_requires_parser(&row) {
        ParserCompletenessGateScope::BenchmarkReporting
    } else {
        ParserCompletenessGateScope::Excluded
    };
    let gate_status = match gate_scope {
        ParserCompletenessGateScope::BenchmarkReporting => {
            if row.parser_status != "not_normalized" {
                ParserCompletenessGateStatus::Pass
            } else {
                ParserCompletenessGateStatus::Fail
            }
        }
        ParserCompletenessGateScope::Excluded => ParserCompletenessGateStatus::Excluded,
    };
    let reason = match gate_status {
        ParserCompletenessGateStatus::Pass => format!(
            "row `{}` / `{}` requires parser completeness for benchmark reporting and satisfies it with `{}`",
            row.stage_id, row.tool_id, row.parser_status
        ),
        ParserCompletenessGateStatus::Fail => format!(
            "row `{}` / `{}` requires parser completeness for benchmark reporting but still reports `{}`",
            row.stage_id, row.tool_id, row.parser_status
        ),
        ParserCompletenessGateStatus::Excluded => row.reason.clone(),
    };

    ParserCompletenessGateRow {
        domain: "fastq".to_string(),
        stage_id: row.stage_id,
        tool_id: row.tool_id,
        gate_scope,
        gate_status,
        benchmark_status: fastq_benchmark_status_label(row.benchmark_status).to_string(),
        readiness_gap: fastq_readiness_gap_label(row.readiness_gap).to_string(),
        support_status: row.support_status,
        adapter_status: row.adapter_status,
        parser_status: row.parser_status,
        corpus_status: row.corpus_status,
        reason,
    }
}

fn render_bam_row(
    row: super::bam_command_adapter_coverage::BamCommandAdapterCoverageRow,
) -> ParserCompletenessGateRow {
    let gate_scope = if bam_row_requires_parser(&row) {
        ParserCompletenessGateScope::BenchmarkReporting
    } else {
        ParserCompletenessGateScope::Excluded
    };
    let gate_status = match gate_scope {
        ParserCompletenessGateScope::BenchmarkReporting => {
            if row.parser_status == "parser_fixture_validated" {
                ParserCompletenessGateStatus::Pass
            } else {
                ParserCompletenessGateStatus::Fail
            }
        }
        ParserCompletenessGateScope::Excluded => ParserCompletenessGateStatus::Excluded,
    };
    let reason = match gate_status {
        ParserCompletenessGateStatus::Pass => format!(
            "row `{}` / `{}` requires parser completeness for benchmark reporting and satisfies it with `{}`",
            row.stage_id, row.tool_id, row.parser_status
        ),
        ParserCompletenessGateStatus::Fail => format!(
            "row `{}` / `{}` requires parser completeness for benchmark reporting but still reports `{}`",
            row.stage_id, row.tool_id, row.parser_status
        ),
        ParserCompletenessGateStatus::Excluded => row.reason.clone(),
    };

    ParserCompletenessGateRow {
        domain: "bam".to_string(),
        stage_id: row.stage_id,
        tool_id: row.tool_id,
        gate_scope,
        gate_status,
        benchmark_status: bam_benchmark_status_label(row.benchmark_status).to_string(),
        readiness_gap: bam_readiness_gap_label(row.readiness_gap).to_string(),
        support_status: row.support_status,
        adapter_status: row.adapter_status,
        parser_status: row.parser_status,
        corpus_status: row.corpus_status,
        reason,
    }
}

fn fastq_row_requires_parser(
    row: &super::fastq_command_adapter_coverage::FastqCommandAdapterCoverageRow,
) -> bool {
    matches!(
        row.support_status.as_str(),
        "governed_execution" | "governed_benchmark_cohort" | "observer_specialized_benchmark"
    ) && matches!(row.adapter_status.as_str(), "runnable" | "plannable")
        && row.corpus_status.starts_with("fixture:")
}

fn bam_row_requires_parser(
    row: &super::bam_command_adapter_coverage::BamCommandAdapterCoverageRow,
) -> bool {
    row.support_status == "supported"
        && matches!(row.adapter_status.as_str(), "runnable" | "plannable")
        && row.corpus_status.starts_with("fixture:")
}

fn fastq_benchmark_status_label(value: FastqBenchmarkStatus) -> &'static str {
    match value {
        FastqBenchmarkStatus::BenchmarkReady => "benchmark_ready",
        FastqBenchmarkStatus::NotBenchmarkReady => "not_benchmark_ready",
    }
}

fn fastq_readiness_gap_label(value: FastqReadinessGapKind) -> &'static str {
    match value {
        FastqReadinessGapKind::None => "none",
        FastqReadinessGapKind::Corpus => "corpus",
        FastqReadinessGapKind::Parser => "parser",
        FastqReadinessGapKind::Adapter => "adapter",
        FastqReadinessGapKind::Support => "support",
    }
}

fn bam_benchmark_status_label(value: BamBenchmarkStatus) -> &'static str {
    match value {
        BamBenchmarkStatus::BenchmarkReady => "benchmark_ready",
        BamBenchmarkStatus::NotBenchmarkReady => "not_benchmark_ready",
    }
}

fn bam_readiness_gap_label(value: BamReadinessGapKind) -> &'static str {
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_parser_completeness_gate, DEFAULT_PARSER_COMPLETENESS_GATE_PATH,
        PARSER_COMPLETENESS_GATE_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn render_parser_completeness_gate_passes_when_reporting_rows_are_parser_complete() {
        let root = repo_root();
        let report = render_parser_completeness_gate(
            &root,
            PathBuf::from(DEFAULT_PARSER_COMPLETENESS_GATE_PATH),
        )
        .expect("render parser completeness gate");

        assert_eq!(report.schema_version, PARSER_COMPLETENESS_GATE_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_PARSER_COMPLETENESS_GATE_PATH);
        assert!(report.passes_gate);
        assert_eq!(report.row_count, 123);
        assert_eq!(report.benchmark_ready_row_count, 112);
        assert_eq!(report.gate_row_count, 112);
        assert_eq!(report.gate_passed_row_count, 112);
        assert_eq!(report.gate_failed_row_count, 0);
        assert_eq!(report.excluded_row_count, 11);
        assert_eq!(report.domain_stage_counts.get("fastq"), Some(&27));
        assert_eq!(report.domain_stage_counts.get("bam"), Some(&24));
        assert_eq!(report.domain_tool_counts.get("fastq"), Some(&44));
        assert_eq!(report.domain_tool_counts.get("bam"), Some(&25));
        assert_eq!(report.domain_row_counts.get("fastq"), Some(&74));
        assert_eq!(report.domain_row_counts.get("bam"), Some(&49));
        assert_eq!(report.gate_domain_row_counts.get("fastq"), Some(&63));
        assert_eq!(report.gate_domain_row_counts.get("bam"), Some(&49));
        assert_eq!(report.excluded_readiness_gap_counts.get("corpus"), Some(&6));
        assert_eq!(report.excluded_readiness_gap_counts.get("support"), Some(&5));
        assert!(!report.excluded_readiness_gap_counts.contains_key("parser"));
        assert!(report
            .rows
            .iter()
            .all(|row| row.gate_status != super::ParserCompletenessGateStatus::Fail));
        assert!(report.rows.iter().any(|row| {
            row.domain == "bam"
                && row.stage_id == "bam.align"
                && row.tool_id == "bwa"
                && row.gate_status == super::ParserCompletenessGateStatus::Pass
                && row.parser_status == "parser_fixture_validated"
        }));
        assert!(report.rows.iter().any(|row| {
            row.domain == "bam"
                && row.stage_id == "bam.align"
                && row.tool_id == "bowtie2"
                && row.gate_status == super::ParserCompletenessGateStatus::Pass
                && row.parser_status == "parser_fixture_validated"
        }));
        assert!(report.rows.iter().any(|row| {
            row.domain == "fastq"
                && row.stage_id == "fastq.index_reference"
                && row.tool_id == "bowtie2_build"
                && row.gate_status == super::ParserCompletenessGateStatus::Excluded
                && row.readiness_gap == "corpus"
        }));
    }
}
