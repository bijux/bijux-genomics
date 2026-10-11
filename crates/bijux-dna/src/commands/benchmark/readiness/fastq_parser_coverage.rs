use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
use bijux_dna_bench_fastq::readiness::inputs::FastqAdapterCoverage;
#[cfg(test)]
use bijux_dna_bench_fastq::readiness::parsers::FASTQ_PARSER_COVERAGE_SCHEMA_VERSION;
pub(crate) use bijux_dna_bench_fastq::readiness::parsers::{
    FastqParserCoverageKind, FastqParserCoverageReport, FastqParserCoverageRow,
    DEFAULT_FASTQ_PARSER_COVERAGE_PATH,
};
use std::path::{Path, PathBuf};
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
    let input = inputs(repo_root)?;
    bijux_dna_bench_fastq::readiness::parsers::render_fastq_parser_coverage(
        repo_root,
        &output_path,
        input,
    )
}
pub(crate) fn collect_fastq_parser_coverage_rows(
    repo_root: &Path,
) -> Result<(usize, usize, Vec<FastqParserCoverageRow>)> {
    let input = inputs(repo_root)?;
    bijux_dna_bench_fastq::readiness::parsers::collect_fastq_parser_coverage_rows(input)
}
fn inputs(repo_root: &Path) -> Result<FastqAdapterCoverage> {
    let coverage =
        super::fastq_command_adapter_coverage::collect_fastq_command_adapter_coverage_rows(
            repo_root,
        )?;
    Ok(coverage)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_fastq_parser_coverage, DEFAULT_FASTQ_PARSER_COVERAGE_PATH,
        FASTQ_PARSER_COVERAGE_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn render_fastq_parser_coverage_reports_governed_parser_rows() {
        let root = repo_root();
        let report =
            render_fastq_parser_coverage(&root, PathBuf::from(DEFAULT_FASTQ_PARSER_COVERAGE_PATH))
                .expect("render FASTQ parser coverage");

        assert_eq!(report.schema_version, FASTQ_PARSER_COVERAGE_SCHEMA_VERSION);
        assert_eq!(report.stage_count, 27);
        assert_eq!(report.tool_count, 43);
        assert_eq!(report.row_count, 71);
        assert_eq!(report.parser_covered_row_count, 71);
        assert_eq!(report.parser_missing_row_count, 0);
        assert_eq!(report.parser_coverage_percent, 100.0);
        assert!(report.rows.iter().all(|row| {
            row.parser_coverage.as_str() == "covered"
                && (row.corpus_status.starts_with("fixture:")
                    || row.corpus_status.starts_with("asset:"))
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "fastqc"
                && row.stage_id == "fastq.validate_reads"
                && row.parser_coverage.as_str() == "covered"
                && row.parser_status == "comparable"
                && row.corpus_status == "fixture:corpus-01-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "bijux_dna"
                && row.stage_id == "fastq.detect_duplicates_premerge"
                && row.parser_coverage.as_str() == "covered"
                && row.parser_status == "parse_normalized"
                && row.corpus_status == "fixture:corpus-01-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "kraken2"
                && row.stage_id == "fastq.screen_taxonomy"
                && row.parser_coverage.as_str() == "covered"
                && row.parser_status == "benchmark_normalized"
                && row.corpus_status == "fixture:corpus-02-edna-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "bowtie2_build"
                && row.stage_id == "fastq.index_reference"
                && row.parser_coverage.as_str() == "covered"
                && row.parser_status == "comparable"
                && row.corpus_status == "asset:reference-index-assets"
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
        assert!(
            rows.iter().any(|row| {
                row.starts_with(
                    "bowtie2_build\tfastq.index_reference\tcovered\tcomparable\tobserver_specialized_benchmark\trunnable\tasset:reference-index-assets\trow `fastq.index_reference` / `bowtie2_build` has governed support, adapter-backed command rendering, asset-backed benchmark coverage, and normalized parser output"
                )
            }),
            "the governed reference-index row must retain asset-backed parser coverage"
        );
    }
}
