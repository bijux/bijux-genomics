use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
use bijux_dna_bench_fastq::readiness::active::FastqActiveStageToolMatrixRow;
#[cfg(test)]
use bijux_dna_bench_fastq::readiness::fixtures::coverage_status_label;
#[cfg(test)]
use bijux_dna_bench_fastq::readiness::fixtures::FASTQ_PARSER_FIXTURE_COVERAGE_SCHEMA_VERSION;
pub(crate) use bijux_dna_bench_fastq::readiness::fixtures::{
    FastqParserFixtureCoverageReport, FastqParserFixtureCoverageRow,
    FastqParserFixtureCoverageStatus, DEFAULT_FASTQ_PARSER_FIXTURE_COVERAGE_PATH,
};
use std::path::{Path, PathBuf};
pub(crate) fn run_render_fastq_parser_fixture_coverage(
    args: &parse::BenchReadinessRenderFastqParserFixtureCoverageArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_fastq_parser_fixture_coverage(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_FASTQ_PARSER_FIXTURE_COVERAGE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_fastq_parser_fixture_coverage(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<FastqParserFixtureCoverageReport> {
    let input = inputs(repo_root)?;
    bijux_dna_bench_fastq::readiness::fixtures::render_fastq_parser_fixture_coverage(
        repo_root,
        &output_path,
        input,
    )
}
pub(crate) fn collect_fastq_parser_fixture_coverage_rows(
    repo_root: &Path,
) -> Result<(usize, usize, Vec<FastqParserFixtureCoverageRow>)> {
    let input = inputs(repo_root)?;
    bijux_dna_bench_fastq::readiness::fixtures::collect_fastq_parser_fixture_coverage_rows(input)
}
fn inputs(repo_root: &Path) -> Result<Vec<FastqActiveStageToolMatrixRow>> {
    let active =
        super::fastq_active_stage_tool_matrix::collect_fastq_active_stage_tool_matrix_rows(
            repo_root,
        )?
        .rows;
    Ok(active)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        coverage_status_label, render_fastq_parser_fixture_coverage,
        FastqParserFixtureCoverageStatus, DEFAULT_FASTQ_PARSER_FIXTURE_COVERAGE_PATH,
        FASTQ_PARSER_FIXTURE_COVERAGE_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn render_fastq_parser_fixture_coverage_reports_governed_rows() {
        let root = repo_root();
        let report = render_fastq_parser_fixture_coverage(
            &root,
            PathBuf::from(DEFAULT_FASTQ_PARSER_FIXTURE_COVERAGE_PATH),
        )
        .expect("render FASTQ parser fixture coverage");

        assert_eq!(report.schema_version, FASTQ_PARSER_FIXTURE_COVERAGE_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_FASTQ_PARSER_FIXTURE_COVERAGE_PATH);
        assert!(report.stage_count > 0);
        assert!(report.tool_count > 0);
        assert_eq!(report.row_count, report.rows.len());
        assert_eq!(report.covered_row_count, report.row_count);
        assert_eq!(report.missing_row_count, 0);
        assert_eq!(report.parser_fixture_coverage_percent, 100.0);
        assert_eq!(report.coverage_status_counts.get("covered"), Some(&report.row_count));

        assert!(report.rows.iter().all(|row| {
            row.coverage_status == FastqParserFixtureCoverageStatus::Covered
                && row.parser_fixture_reference_kind == "fixture_case"
                && !row.parser_fixture_reference.is_empty()
                && !row.parser_fixture_parser_id.is_empty()
                && !row.parser_fixture_schema_id.is_empty()
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.trim_reads"
                && row.tool_id == "trimmomatic"
                && row.parser_fixture_parser_id == "parse_trim_reads_report"
                && row.parser_fixture_reference == "fastq.trim_reads.report_json"
                && row.parser_fixture_canonical_tool_id == "fastp"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.detect_duplicates_premerge"
                && row.tool_id == "bijux_dna"
                && row.parser_fixture_parser_id == "parse_detect_duplicates_premerge_report"
                && row.parser_fixture_reference == "fastq.detect_duplicates_premerge.report_json"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.screen_taxonomy"
                && row.tool_id == "kraken2"
                && row.parser_fixture_parser_id == "parse_screen_taxonomy_report"
                && row.parser_fixture_schema_id == "bijux.fastq.screen_taxonomy.report.v2"
        }));
    }

    #[test]
    fn coverage_status_labels_are_stable() {
        assert_eq!(coverage_status_label(FastqParserFixtureCoverageStatus::Covered), "covered");
        assert_eq!(
            coverage_status_label(FastqParserFixtureCoverageStatus::MissingFixtureBinding),
            "missing_fixture_binding"
        );
        assert_eq!(
            coverage_status_label(FastqParserFixtureCoverageStatus::MissingFixtureCase),
            "missing_fixture_case"
        );
        assert_eq!(
            coverage_status_label(FastqParserFixtureCoverageStatus::InvalidFixtureCase),
            "invalid_fixture_case"
        );
    }
}
