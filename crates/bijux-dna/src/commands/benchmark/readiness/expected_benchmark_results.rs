use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
use bijux_dna_bench_bindings::expected as owner;
use std::path::{Path, PathBuf};
pub(crate) const DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH: &str =
    "artifacts/benchmark-smoke/readiness/expected-benchmark-results.tsv";

use super::binding_evidence::ExpectedAdapter;
use crate::commands::benchmark::path_resolution::BenchmarkPathResolver;
#[cfg(test)]
use owner::EXPECTED_BENCHMARK_RESULTS_SCHEMA_VERSION;
pub(crate) use owner::{ExpectedBenchmarkResultRow, ExpectedBenchmarkResultsReport};
pub(crate) fn run_render_expected_benchmark_results(
    args: &parse::BenchReadinessRenderExpectedBenchmarkResultsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let report = render_expected_benchmark_results(
        &repo_root,
        args.output.clone().unwrap_or_else(|| {
            benchmark_paths.benchmark_readiness_root().join("expected-benchmark-results.tsv")
        }),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_expected_benchmark_results(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<ExpectedBenchmarkResultsReport> {
    owner::render_expected_benchmark_results(&ExpectedAdapter.context(repo_root), &output_path)
}
pub(crate) fn collect_expected_benchmark_result_rows(
    repo_root: &Path,
) -> Result<Vec<ExpectedBenchmarkResultRow>> {
    owner::collect_expected_benchmark_result_rows(&ExpectedAdapter.context(repo_root))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_expected_benchmark_results, DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH,
        EXPECTED_BENCHMARK_RESULTS_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn expected_benchmark_results_report_tracks_governed_result_rows() {
        let root = repo_root();
        let report = render_expected_benchmark_results(
            &root,
            PathBuf::from(DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH),
        )
        .expect("render expected benchmark results");

        assert_eq!(report.schema_version, EXPECTED_BENCHMARK_RESULTS_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH);
        assert!(report.row_count > 0);
        assert_eq!(report.stage_count, 51);
        assert_eq!(report.rows.len(), report.row_count);
        assert_eq!(report.domain_counts.values().sum::<usize>(), report.row_count);
        assert!(report.domain_counts.contains_key("fastq"));
        assert!(report.domain_counts.contains_key("bam"));
        assert!(report.rows.iter().any(|row| {
            row.domain == "fastq"
                && row.stage_id == "fastq.index_reference"
                && row.tool_id == "bowtie2_build"
                && row.fixture_id == "reference-index-assets"
                && row.sample_scope == "asset-set"
                && row.normalized_metrics_output_id.as_deref() == Some("report_json")
                && row.stage_result_manifest_path
                    == "artifacts/runs/bench/slurm-dry-run/runs/local-benchmark-dry-run/reference-index-assets/fastq.index_reference/asset-set/bowtie2_build/stage-result.json"
        }));
        assert!(report.rows.iter().any(|row| {
            row.domain == "fastq"
                && row.stage_id == "fastq.screen_taxonomy"
                && row.tool_id == "kraken2"
                && row.fixture_id == "corpus-02-edna-mini"
                && row.sample_scope == "sample-set"
                && row.normalized_metrics_output_id.as_deref() == Some("classification_report_json")
                && row
                    .stage_result_manifest_path
                    == "artifacts/runs/bench/slurm-dry-run/runs/local-benchmark-dry-run/corpus-02-edna-mini/fastq.screen_taxonomy/sample-set/kraken2/stage-result.json"
        }));
        assert!(report.rows.iter().any(|row| {
            row.domain == "bam"
                && row.stage_id == "bam.damage"
                && row.tool_id == "mapdamage2"
                && row.fixture_id == "corpus-01-adna-damage-mini"
                && row.sample_scope == "adna_damage_non_udg"
                && row.normalized_metrics_output_id.as_deref() == Some("damage_report")
        }));
        assert!(report.rows.iter().all(|row| !row.result_row_id.is_empty()));
    }
}
