use super::bam_evidence::BamEvidenceAdapter;
use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
pub(crate) const DEFAULT_BAM_COMMAND_ADAPTER_COVERAGE_PATH: &str =
    "artifacts/benchmark-smoke/readiness/bam/bam-command-adapter-coverage.tsv";
pub(crate) type BamBenchmarkStatus =
    bijux_dna_bench_bam_evidence::command_coverage::BamBenchmarkStatus;
pub(crate) type BamAdapterCoverageKind =
    bijux_dna_bench_bam_evidence::command_coverage::BamAdapterCoverageKind;
pub(crate) type BamReadinessGapKind =
    bijux_dna_bench_bam_evidence::command_coverage::BamReadinessGapKind;
pub(crate) type BamCommandAdapterCoverageRow =
    bijux_dna_bench_bam_evidence::command_coverage::BamCommandAdapterCoverageRow;
pub(crate) type BamCommandAdapterCoverageReport =
    bijux_dna_bench_bam_evidence::command_coverage::BamCommandAdapterCoverageReport;
pub(crate) fn run_render_bam_command_adapter_coverage(
    args: &parse::BenchReadinessRenderBamCommandAdapterCoverageArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_command_adapter_coverage(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_COMMAND_ADAPTER_COVERAGE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_bam_command_adapter_coverage(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamCommandAdapterCoverageReport> {
    bijux_dna_bench_bam_evidence::command_coverage::render_bam_command_adapter_coverage(
        &BamEvidenceAdapter.context(repo_root),
        &output_path,
    )
}
pub(crate) fn collect_bam_command_adapter_coverage_rows(
    repo_root: &Path,
) -> Result<(usize, usize, Vec<BamCommandAdapterCoverageRow>)> {
    bijux_dna_bench_bam_evidence::command_coverage::collect_bam_command_adapter_coverage_rows(
        &BamEvidenceAdapter.context(repo_root),
    )
}
#[cfg(test)]
use bijux_dna_bench_bam_evidence::command_coverage::BAM_COMMAND_ADAPTER_COVERAGE_SCHEMA_VERSION;
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{render_bam_command_adapter_coverage, BAM_COMMAND_ADAPTER_COVERAGE_SCHEMA_VERSION};

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn bam_command_adapter_coverage_reports_governed_row_readiness() {
        let root = repo_root();
        let output = tempfile::tempdir().expect("temporary output");
        let report = render_bam_command_adapter_coverage(&root, output.path().join("bam.tsv"))
            .expect("render BAM command adapter coverage");

        assert_eq!(report.schema_version, BAM_COMMAND_ADAPTER_COVERAGE_SCHEMA_VERSION);
        assert_eq!(report.stage_count, 24);
        assert_eq!(report.row_count, report.rows.len());
        assert_eq!(report.benchmark_ready_row_count, report.row_count);
        assert_eq!(report.benchmark_ready_adapter_covered_row_count, report.row_count);
        assert_eq!(report.benchmark_ready_adapter_missing_row_count, 0);
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "angsd"
                && row.stage_id == "bam.kinship"
                && row.benchmark_status.as_str() == "benchmark_ready"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "ngsbriggs"
                && row.stage_id == "bam.damage"
                && row.benchmark_status.as_str() == "benchmark_ready"
        }));
        assert!(report.readiness_gap_counts.get("parser").is_none());
        assert!(
            report.readiness_gap_counts.get("support").is_none(),
            "the governed BAM readiness slice should no longer carry unsupported rows"
        );
        assert!(
            report.readiness_gap_counts.get("corpus").is_none(),
            "the governed BAM readiness slice should no longer carry corpus gaps"
        );
        assert!(
            report.readiness_gap_counts.get("adapter").is_none(),
            "the governed BAM readiness slice currently carries no adapter gap rows"
        );
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "samtools"
                && row.stage_id == "bam.validate"
                && row.benchmark_status.as_str() == "benchmark_ready"
                && row.adapter_coverage.as_str() == "covered"
                && row.readiness_gap.as_str() == "none"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "mapdamage2"
                && row.stage_id == "bam.damage"
                && row.benchmark_status.as_str() == "benchmark_ready"
                && row.corpus_status == "fixture:corpus-01-adna-damage-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "authenticct"
                && row.stage_id == "bam.authenticity"
                && row.benchmark_status.as_str() == "benchmark_ready"
                && row.adapter_coverage.as_str() == "covered"
                && row.readiness_gap.as_str() == "none"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-adna-damage-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "schmutzi"
                && row.stage_id == "bam.contamination"
                && row.benchmark_status.as_str() == "benchmark_ready"
                && row.adapter_coverage.as_str() == "covered"
                && row.readiness_gap.as_str() == "none"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-adna-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "mapdamage2"
                && row.stage_id == "bam.bias_mitigation"
                && row.benchmark_status.as_str() == "benchmark_ready"
                && row.adapter_coverage.as_str() == "covered"
                && row.readiness_gap.as_str() == "none"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "rxy"
                && row.stage_id == "bam.sex"
                && row.benchmark_status.as_str() == "benchmark_ready"
                && row.adapter_coverage.as_str() == "covered"
                && row.readiness_gap.as_str() == "none"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-adna-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "mosdepth"
                && row.stage_id == "bam.coverage"
                && row.benchmark_status.as_str() == "benchmark_ready"
                && row.adapter_coverage.as_str() == "covered"
                && row.readiness_gap.as_str() == "none"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "picard"
                && row.stage_id == "bam.gc_bias"
                && row.benchmark_status.as_str() == "benchmark_ready"
                && row.adapter_coverage.as_str() == "covered"
                && row.readiness_gap.as_str() == "none"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "picard"
                && row.stage_id == "bam.insert_size"
                && row.benchmark_status.as_str() == "benchmark_ready"
                && row.adapter_coverage.as_str() == "covered"
                && row.readiness_gap.as_str() == "none"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "samtools"
                && row.stage_id == "bam.endogenous_content"
                && row.benchmark_status.as_str() == "benchmark_ready"
                && row.adapter_coverage.as_str() == "covered"
                && row.readiness_gap.as_str() == "none"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "bamutil"
                && row.stage_id == "bam.overlap_correction"
                && row.benchmark_status.as_str() == "benchmark_ready"
                && row.adapter_coverage.as_str() == "covered"
                && row.readiness_gap.as_str() == "none"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-bam-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "bwa"
                && row.stage_id == "bam.align"
                && row.benchmark_status.as_str() == "benchmark_ready"
                && row.adapter_coverage.as_str() == "covered"
                && row.readiness_gap.as_str() == "none"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "bowtie2"
                && row.stage_id == "bam.align"
                && row.benchmark_status.as_str() == "benchmark_ready"
                && row.adapter_coverage.as_str() == "covered"
                && row.readiness_gap.as_str() == "none"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "bcftools"
                && row.stage_id == "bam.genotyping"
                && row.benchmark_status.as_str() == "benchmark_ready"
                && row.adapter_coverage.as_str() == "covered"
                && row.readiness_gap.as_str() == "none"
                && row.support_status == "supported"
                && row.parser_status == "parser_fixture_validated"
                && row.corpus_status == "fixture:corpus-01-genotyping-mini"
        }));
    }

    #[test]
    fn bam_command_adapter_coverage_writes_governed_tsv_columns() {
        let root = repo_root();
        let output = tempfile::tempdir().expect("temporary output");
        let output_path = output.path().join("bam.tsv");
        let report =
            render_bam_command_adapter_coverage(&root, output_path).expect("render coverage");
        let rendered = std::fs::read_to_string(root.join(&report.output_path))
            .expect("read rendered bam command adapter coverage tsv");
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
                    "samtools\tbam.validate\tbenchmark_ready\tcovered\tnone\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini\t"
                )
            }),
            "the governed BAM validation row must remain benchmark-ready and adapter-covered"
        );
        assert!(
            rows.iter().any(|row| {
                row.starts_with(
                    "authenticct\tbam.authenticity\tbenchmark_ready\tcovered\tnone\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-adna-damage-mini\t"
                )
            }),
            "the governed BAM authenticity row must now be benchmark-ready and adapter-covered"
        );
        assert!(
            rows.iter().any(|row| {
                row.starts_with(
                    "schmutzi\tbam.contamination\tbenchmark_ready\tcovered\tnone\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-adna-bam-mini\t"
                )
            }),
            "the governed BAM contamination row must now be benchmark-ready and adapter-covered"
        );
        assert!(
            rows.iter().any(|row| {
                row.starts_with(
                    "samtools\tbam.endogenous_content\tbenchmark_ready\tcovered\tnone\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini\t"
                )
            }),
            "the governed BAM endogenous-content row must now be benchmark-ready and adapter-covered"
        );
        assert!(
            rows.iter().any(|row| {
                row.starts_with(
                    "bamutil\tbam.overlap_correction\tbenchmark_ready\tcovered\tnone\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-bam-mini\t"
                )
            }),
            "the governed BAM overlap-correction row must now be benchmark-ready and adapter-covered"
        );
        assert!(
            rows.iter().any(|row| {
                row.starts_with(
                    "bcftools\tbam.genotyping\tbenchmark_ready\tcovered\tnone\tsupported\trunnable\tparser_fixture_validated\tfixture:corpus-01-genotyping-mini\t"
                )
            }),
            "the governed BAM genotyping row must be benchmark-ready and adapter-covered through the owned bcftools fixture"
        );
    }
}
