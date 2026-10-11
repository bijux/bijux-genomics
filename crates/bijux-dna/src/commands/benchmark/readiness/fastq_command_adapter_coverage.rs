use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
#[cfg(test)]
use bijux_dna_bench_fastq::readiness::adapters::FASTQ_COMMAND_ADAPTER_COVERAGE_SCHEMA_VERSION;
pub(crate) use bijux_dna_bench_fastq::readiness::adapters::{
    FastqBenchmarkStatus, FastqCommandAdapterCoverageReport, FastqCommandAdapterCoverageRow,
    FastqReadinessGapKind, DEFAULT_FASTQ_COMMAND_ADAPTER_COVERAGE_PATH,
};
use bijux_dna_bench_fastq::readiness::inputs::FastqServingSnapshot;
use std::path::{Path, PathBuf};
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
    let input = inputs(repo_root)?;
    bijux_dna_bench_fastq::readiness::adapters::render_fastq_command_adapter_coverage(
        repo_root,
        &output_path,
        input,
    )
}
pub(crate) fn collect_fastq_command_adapter_coverage_rows(
    repo_root: &Path,
) -> Result<(usize, usize, Vec<FastqCommandAdapterCoverageRow>)> {
    let input = inputs(repo_root)?;
    bijux_dna_bench_fastq::readiness::adapters::collect_fastq_command_adapter_coverage_rows(input)
}
fn inputs(repo_root: &Path) -> Result<FastqServingSnapshot> {
    let (stage_count, tool_count, rows) =
        super::tool_serving_map::collect_fastq_tool_serving_map_rows(repo_root)?;
    let rows = rows
        .into_iter()
        .map(|row| bijux_dna_bench_fastq::readiness::inputs::FastqServingRow {
            stage_id: row.stage_id,
            tool_id: row.tool_id,
            support_status: row.support_status,
            adapter_status: row.adapter_status,
            parser_status: row.parser_status,
            corpus_status: row.corpus_status,
        })
        .collect();
    Ok(FastqServingSnapshot { stage_count, tool_count, rows })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_fastq_command_adapter_coverage, FASTQ_COMMAND_ADAPTER_COVERAGE_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn fastq_command_adapter_coverage_reports_governed_row_readiness() {
        let root = repo_root();
        let output = tempfile::tempdir().expect("temporary output");
        let report = render_fastq_command_adapter_coverage(&root, output.path().join("fastq.tsv"))
            .expect("render FASTQ command adapter coverage");

        assert_eq!(report.schema_version, FASTQ_COMMAND_ADAPTER_COVERAGE_SCHEMA_VERSION);
        assert_eq!(report.stage_count, 27);
        assert_eq!(report.row_count, report.rows.len());
        assert_eq!(report.benchmark_ready_row_count, report.row_count);
        assert_eq!(report.benchmark_ready_adapter_covered_row_count, report.row_count);
        assert_eq!(report.benchmark_ready_adapter_missing_row_count, 0);
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "fastuniq"
                && row.stage_id == "fastq.remove_duplicates"
                && row.benchmark_status.as_str() == "benchmark_ready"
                && row.adapter_coverage.as_str() == "covered"
                && row.readiness_gap.as_str() == "none"
        }));
        assert!(report.readiness_gap_counts.is_empty());
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
                && row.benchmark_status.as_str() == "benchmark_ready"
                && row.adapter_coverage.as_str() == "covered"
                && row.readiness_gap.as_str() == "none"
                && row.corpus_status == "fixture:corpus-01-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "kraken2"
                && row.stage_id == "fastq.screen_taxonomy"
                && row.benchmark_status.as_str() == "benchmark_ready"
                && row.corpus_status == "fixture:corpus-02-edna-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "bowtie2_build"
                && row.stage_id == "fastq.index_reference"
                && row.benchmark_status.as_str() == "benchmark_ready"
                && row.adapter_coverage.as_str() == "covered"
                && row.readiness_gap.as_str() == "none"
                && row.corpus_status == "asset:reference-index-assets"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "star"
                && row.stage_id == "fastq.index_reference"
                && row.benchmark_status.as_str() == "benchmark_ready"
                && row.adapter_coverage.as_str() == "covered"
                && row.readiness_gap.as_str() == "none"
                && row.corpus_status == "asset:reference-index-assets"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "fastqc"
                && row.stage_id == "fastq.detect_adapters"
                && row.benchmark_status.as_str() == "benchmark_ready"
                && row.adapter_coverage.as_str() == "covered"
                && row.readiness_gap.as_str() == "none"
                && row.corpus_status == "fixture:corpus-01-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "fastp"
                && row.stage_id == "fastq.filter_reads"
                && row.benchmark_status.as_str() == "benchmark_ready"
                && row.adapter_coverage.as_str() == "covered"
                && row.readiness_gap.as_str() == "none"
                && row.corpus_status == "fixture:corpus-01-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "fastp"
                && row.stage_id == "fastq.trim_polyg_tails"
                && row.benchmark_status.as_str() == "benchmark_ready"
                && row.adapter_coverage.as_str() == "covered"
                && row.readiness_gap.as_str() == "none"
                && row.corpus_status == "fixture:corpus-01-mini"
        }));
        assert!(report.rows.iter().any(|row| {
            row.tool_id == "cutadapt"
                && row.stage_id == "fastq.trim_terminal_damage"
                && row.benchmark_status.as_str() == "benchmark_ready"
                && row.adapter_coverage.as_str() == "covered"
                && row.readiness_gap.as_str() == "none"
                && row.corpus_status == "fixture:corpus-01-mini"
        }));
        for tool_id in ["fastq_scan", "fastqc", "seqkit"] {
            assert!(report.rows.iter().any(|row| {
                row.tool_id == tool_id
                    && row.stage_id == "fastq.profile_overrepresented_sequences"
                    && row.benchmark_status.as_str() == "benchmark_ready"
                    && row.adapter_coverage.as_str() == "covered"
                    && row.readiness_gap.as_str() == "none"
                    && row.corpus_status == "fixture:corpus-01-mini"
            }));
        }
        for tool_id in ["centrifuge", "kaiju", "kraken2", "krakenuniq"] {
            assert!(report.rows.iter().any(|row| {
                row.tool_id == tool_id
                    && row.stage_id == "fastq.screen_taxonomy"
                    && row.benchmark_status.as_str() == "benchmark_ready"
                    && row.adapter_coverage.as_str() == "covered"
                    && row.readiness_gap.as_str() == "none"
                    && row.corpus_status == "fixture:corpus-02-edna-mini"
            }));
        }
    }

    #[test]
    fn fastq_command_adapter_coverage_writes_governed_tsv_columns() {
        let root = repo_root();
        let output = tempfile::tempdir().expect("temporary output");
        let output_path = output.path().join("fastq.tsv");
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
                    "bowtie2_build\tfastq.index_reference\tbenchmark_ready\tcovered\tnone\tobserver_specialized_benchmark\trunnable\tcomparable\tasset:reference-index-assets\t"
                )
            }),
            "the governed bowtie2 reference-index row must remain benchmark-ready and asset-backed"
        );
        assert!(
            rows.iter().any(|row| {
                row.starts_with(
                    "star\tfastq.index_reference\tbenchmark_ready\tcovered\tnone\tobserver_specialized_benchmark\trunnable\tcomparable\tasset:reference-index-assets\t"
                )
            }),
            "the governed star reference-index row must remain benchmark-ready and asset-backed"
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
