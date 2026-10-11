#[cfg(test)]
const FIXTURE_REFERENCE_KIND_CORPUS: &str = "fixture_corpus";
use super::bam_evidence::BamEvidenceAdapter;
use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
pub(crate) const DEFAULT_BAM_PARSER_FIXTURE_COVERAGE_PATH: &str =
    "artifacts/benchmark-smoke/readiness/bam/bam-parser-fixture-coverage.tsv";
pub(crate) type BamParserFixtureCoverageStatus =
    bijux_dna_bench_bam_evidence::parser_coverage::BamParserFixtureCoverageStatus;
pub(crate) type BamParserFixtureCoverageRow =
    bijux_dna_bench_bam_evidence::parser_coverage::BamParserFixtureCoverageRow;
pub(crate) type BamParserFixtureCoverageReport =
    bijux_dna_bench_bam_evidence::parser_coverage::BamParserFixtureCoverageReport;
pub(crate) fn run_render_bam_parser_fixture_coverage(
    args: &parse::BenchReadinessRenderBamParserFixtureCoverageArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_parser_fixture_coverage(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_PARSER_FIXTURE_COVERAGE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_bam_parser_fixture_coverage(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamParserFixtureCoverageReport> {
    bijux_dna_bench_bam_evidence::parser_coverage::render_bam_parser_fixture_coverage(
        &BamEvidenceAdapter.context(repo_root),
        &output_path,
    )
}
pub(crate) fn collect_bam_parser_fixture_coverage_rows(
    repo_root: &Path,
) -> Result<(usize, usize, Vec<BamParserFixtureCoverageRow>, BTreeMap<String, usize>)> {
    let collected =
        bijux_dna_bench_bam_evidence::parser_coverage::collect_bam_parser_fixture_coverage_rows(
            &BamEvidenceAdapter.context(repo_root),
        )?;
    Ok((
        collected.stage_count,
        collected.tool_count,
        collected.rows,
        collected.excluded_readiness_gap_counts,
    ))
}
#[cfg(test)]
use bijux_dna_bench_bam_evidence::parser_coverage::BAM_PARSER_FIXTURE_COVERAGE_SCHEMA_VERSION;
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_bam_parser_fixture_coverage, BAM_PARSER_FIXTURE_COVERAGE_SCHEMA_VERSION,
        DEFAULT_BAM_PARSER_FIXTURE_COVERAGE_PATH, FIXTURE_REFERENCE_KIND_CORPUS,
    };

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn render_bam_parser_fixture_coverage_reports_governed_parser_rows() {
        let root = repo_root();
        let report = render_bam_parser_fixture_coverage(
            &root,
            PathBuf::from(DEFAULT_BAM_PARSER_FIXTURE_COVERAGE_PATH),
        )
        .expect("render BAM parser fixture coverage");

        assert_eq!(report.schema_version, BAM_PARSER_FIXTURE_COVERAGE_SCHEMA_VERSION);
        assert_eq!(report.stage_count, 24);
        assert_eq!(report.tool_count, 26);
        assert_eq!(report.row_count, 49);
        assert_eq!(report.covered_row_count, 49);
        assert_eq!(report.missing_row_count, 0);
        assert_eq!(report.parser_fixture_coverage_percent, 100.0);
        assert_eq!(report.excluded_non_benchmark_ready_row_count, 0);
        assert!(report.excluded_readiness_gap_counts.is_empty());
        assert!(report.rows.iter().all(|row| {
            row.coverage_status.as_str() == "covered"
                && row.parser_status == "parser_fixture_validated"
                && row.parser_fixture_reference_kind == FIXTURE_REFERENCE_KIND_CORPUS
                && row.parser_fixture_reference.starts_with("fixture:")
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "samtools"
                && row.stage_id == "bam.validate"
                && row.parser_fixture_reference == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "mapdamage2"
                && row.stage_id == "bam.damage"
                && row.parser_fixture_reference == "fixture:corpus-01-adna-damage-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "bcftools"
                && row.stage_id == "bam.genotyping"
                && row.parser_fixture_reference == "fixture:corpus-01-genotyping-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "bwa"
                && row.stage_id == "bam.align"
                && row.parser_fixture_reference == "fixture:corpus-01-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "bowtie2"
                && row.stage_id == "bam.align"
                && row.parser_fixture_reference == "fixture:corpus-01-mini"
        }));
    }

    #[test]
    fn render_bam_parser_fixture_coverage_writes_governed_tsv_columns() {
        let root = repo_root();
        let output_path = PathBuf::from(DEFAULT_BAM_PARSER_FIXTURE_COVERAGE_PATH);
        let report = render_bam_parser_fixture_coverage(&root, output_path)
            .expect("render BAM parser fixture coverage");
        let rendered = std::fs::read_to_string(root.join(&report.output_path))
            .expect("read rendered bam parser fixture coverage tsv");
        let rows = rendered.lines().collect::<Vec<_>>();

        assert_eq!(
            rows.first().copied(),
            Some(
                "tool_id\tstage_id\tparser_fixture_reference_kind\tparser_fixture_reference\tcoverage_status\tparser_status\tsupport_status\tadapter_status\treason"
            )
        );
        assert!(
            rows.iter().any(|row| {
                row.starts_with(
                    "samtools\tbam.validate\tfixture_corpus\tfixture:corpus-01-bam-mini\tcovered\tparser_fixture_validated\tsupported\trunnable\t"
                )
            }),
            "the governed BAM validation row must retain parser fixture coverage"
        );
        assert!(
            rows.iter().any(|row| {
                row.starts_with(
                    "mapdamage2\tbam.damage\tfixture_corpus\tfixture:corpus-01-adna-damage-mini\tcovered\tparser_fixture_validated\tsupported\trunnable\t"
                )
            }),
            "the governed BAM damage row must retain parser fixture coverage"
        );
        assert!(
            rows.iter().any(|row| {
                row.starts_with(
                    "bcftools\tbam.genotyping\tfixture_corpus\tfixture:corpus-01-genotyping-mini\tcovered\tparser_fixture_validated\tsupported\trunnable\t"
                )
            }),
            "the governed BAM genotyping row must retain parser fixture coverage"
        );
    }
}
