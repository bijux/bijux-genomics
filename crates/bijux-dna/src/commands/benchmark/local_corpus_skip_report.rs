use std::path::Path;

use anyhow::{Context, Result};
use bijux_dna_api::v1::api::bench::{
    publish_benchmark_local_corpus_skip_report, BenchmarkLocalCorpusSkipPublicationRequest,
    BenchmarkLocalCorpusSkipReport,
};

use crate::commands::benchmark::local_corpus_stage_compatibility::validate_corpus_stage_compatibility_path;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_CORPUS_SKIP_REPORT_PATH: &str =
    "artifacts/benchmark-smoke/readiness/local-ready/corpus-skip-report.json";
pub(crate) type LocalCorpusSkipReport = BenchmarkLocalCorpusSkipReport;

pub(crate) fn run_render_corpus_skip_report(
    args: &parse::BenchLocalRenderCorpusSkipReportArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let matrix_path = match &args.matrix {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => repo_root.join(path),
        None => repo_root.join(
            crate::commands::benchmark::local_corpus_stage_compatibility::DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH,
        ),
    };
    let output_path = match &args.output {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => repo_root.join(path),
        None => repo_root.join(DEFAULT_CORPUS_SKIP_REPORT_PATH),
    };

    let report = render_corpus_skip_report_path(&repo_root, &matrix_path, &output_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_corpus_skip_report_path(
    repo_root: &Path,
    matrix_path: &Path,
    output_path: &Path,
) -> Result<LocalCorpusSkipReport> {
    let compatibility = validate_corpus_stage_compatibility_path(repo_root, matrix_path)?;
    publish_benchmark_local_corpus_skip_report(&BenchmarkLocalCorpusSkipPublicationRequest {
        repository_root: repo_root,
        output_path,
        compatibility: &compatibility,
    })
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{render_corpus_skip_report_path, DEFAULT_CORPUS_SKIP_REPORT_PATH};
    use crate::commands::benchmark::local_corpus_stage_compatibility::DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH;

    fn repo_root() -> std::path::PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn corpus_skip_report_tracks_fixture_skips_asset_backed_and_planner_only_stages() {
        let repo_root = repo_root();
        let output_path = repo_root.join(DEFAULT_CORPUS_SKIP_REPORT_PATH);
        let report = render_corpus_skip_report_path(
            &repo_root,
            &repo_root.join(DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH),
            &output_path,
        )
        .expect("render corpus skip report");

        assert_eq!(report.fixture_count, 8);
        assert_eq!(report.stage_count, 51);
        assert_eq!(report.skip_count, 350);
        assert_eq!(report.asset_backed_stage_count, 1);
        assert_eq!(report.planner_only_stage_count, 0);
        assert!(
            report.skips.iter().any(|skip| {
                skip.stage_id == "fastq.screen_taxonomy"
                    && skip.corpus_id == "corpus-01-mini"
                    && skip.replacement_corpus_id == "corpus-02-edna-mini"
            }),
            "fixture-backed taxonomy stage should emit an explicit replacement for incompatible corpora"
        );
        assert!(
            report.skips.iter().any(|skip| {
                skip.stage_id == "fastq.filter_reads"
                    && skip.corpus_id == "corpus-02-edna-mini"
                    && skip.replacement_corpus_id == "corpus-01-mini"
            }),
            "fixture-backed filter-reads stage should emit an explicit replacement for incompatible corpora"
        );
        assert!(
            report.skips.iter().any(|skip| {
                skip.stage_id == "fastq.estimate_library_complexity_prealign"
                    && skip.corpus_id == "corpus-02-edna-mini"
                    && skip.replacement_corpus_id == "corpus-01-mini"
            }),
            "fixture-backed estimate-library-complexity stage should emit an explicit replacement for incompatible corpora"
        );
        assert!(
            report.skips.iter().any(|skip| {
                skip.stage_id == "fastq.trim_polyg_tails"
                    && skip.corpus_id == "corpus-02-edna-mini"
                    && skip.replacement_corpus_id == "corpus-01-mini"
            }),
            "fixture-backed trim-polyg stage should emit an explicit replacement for incompatible corpora"
        );
        assert!(
            report.skips.iter().any(|skip| {
                skip.stage_id == "fastq.trim_terminal_damage"
                    && skip.corpus_id == "corpus-02-edna-mini"
                    && skip.replacement_corpus_id == "corpus-01-mini"
            }),
            "fixture-backed trim-terminal-damage stage should emit an explicit replacement for incompatible corpora"
        );
        assert!(
            report.skips.iter().any(|skip| {
                skip.stage_id == "bam.contamination"
                    && skip.corpus_id == "corpus-01-mini"
                    && skip.replacement_corpus_id == "corpus-01-adna-bam-mini"
            }),
            "fixture-backed contamination stage should emit an explicit replacement for incompatible corpora"
        );
        assert!(
            report.skips.iter().any(|skip| {
                skip.stage_id == "bam.authenticity"
                    && skip.corpus_id == "corpus-01-mini"
                    && skip.replacement_corpus_id == "corpus-01-adna-damage-mini"
            }),
            "fixture-backed authenticity stage should emit an explicit replacement for incompatible corpora"
        );
        assert!(
            report.skips.iter().any(|skip| {
                skip.stage_id == "bam.bias_mitigation"
                    && skip.corpus_id == "corpus-01-mini"
                    && skip.replacement_corpus_id == "corpus-01-bam-mini"
            }),
            "fixture-backed bias-mitigation stage should emit an explicit replacement for incompatible corpora"
        );
        assert!(
            report.asset_backed_stages.iter().any(|stage| {
                stage.stage_id == "fastq.index_reference"
                    && stage.benchmark_scope_id == "reference-index-assets"
                    && stage.reason.contains("owned reference assets")
            }),
            "asset-backed stages must stay explicit in the skip report"
        );
        assert!(
            report.planner_only_stages.is_empty(),
            "planner-only stages should be empty once fastq.report_qc is fixture-backed"
        );
    }
}
