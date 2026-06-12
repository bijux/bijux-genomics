use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;

use crate::commands::benchmark::local_corpus_stage_compatibility::{
    validate_corpus_stage_compatibility_path, LocalCorpusStageCompatibilityValidationReport,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_CORPUS_SKIP_REPORT_PATH: &str =
    "benchmarks/readiness/local-ready/corpus-skip-report.json";
const LOCAL_CORPUS_SKIP_REPORT_SCHEMA_VERSION: &str = "bijux.bench.local_corpus_skip_report.v1";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LocalCorpusSkipReportFixture {
    pub(crate) corpus_id: String,
    pub(crate) corpus_family_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LocalCorpusSkipEntry {
    pub(crate) stage_id: String,
    pub(crate) readiness_kind: String,
    pub(crate) corpus_id: String,
    pub(crate) corpus_family_id: String,
    pub(crate) reason: String,
    pub(crate) replacement_corpus_id: String,
    pub(crate) replacement_corpus_family_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LocalPlannerOnlyStageReport {
    pub(crate) stage_id: String,
    pub(crate) readiness_kind: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LocalAssetBackedStageReport {
    pub(crate) stage_id: String,
    pub(crate) readiness_kind: String,
    pub(crate) benchmark_scope_id: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LocalCorpusSkipReport {
    pub(crate) schema_version: &'static str,
    pub(crate) matrix_path: String,
    pub(crate) output_path: String,
    pub(crate) fixture_count: usize,
    pub(crate) stage_count: usize,
    pub(crate) skip_count: usize,
    pub(crate) asset_backed_stage_count: usize,
    pub(crate) planner_only_stage_count: usize,
    pub(crate) fixtures: Vec<LocalCorpusSkipReportFixture>,
    pub(crate) skips: Vec<LocalCorpusSkipEntry>,
    pub(crate) asset_backed_stages: Vec<LocalAssetBackedStageReport>,
    pub(crate) planner_only_stages: Vec<LocalPlannerOnlyStageReport>,
}

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
    let report = build_skip_report(repo_root, output_path, &compatibility);
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(output_path, &report)
        .with_context(|| format!("write {}", output_path.display()))?;
    Ok(report)
}

fn build_skip_report(
    repo_root: &Path,
    output_path: &Path,
    compatibility: &LocalCorpusStageCompatibilityValidationReport,
) -> LocalCorpusSkipReport {
    let fixtures = compatibility
        .fixtures
        .iter()
        .map(|fixture| LocalCorpusSkipReportFixture {
            corpus_id: fixture.fixture_id.clone(),
            corpus_family_id: fixture.corpus_family_id.clone(),
        })
        .collect::<Vec<_>>();

    let mut skips = Vec::new();
    let mut asset_backed_stages = Vec::new();
    let mut planner_only_stages = Vec::new();
    for stage in &compatibility.stages {
        match (
            stage.fixture_id.as_deref(),
            stage.corpus_family_id.as_deref(),
            stage.benchmark_scope_id.as_deref(),
            stage.compatibility_kind.as_str(),
        ) {
            (Some(replacement_corpus_id), Some(replacement_corpus_family_id), None, "fixture") => {
                for fixture in &fixtures {
                    if fixture.corpus_id == replacement_corpus_id {
                        continue;
                    }
                    skips.push(LocalCorpusSkipEntry {
                        stage_id: stage.stage_id.clone(),
                        readiness_kind: stage.readiness_kind.clone(),
                        corpus_id: fixture.corpus_id.clone(),
                        corpus_family_id: fixture.corpus_family_id.clone(),
                        reason: format!(
                            "stage is governed against `{replacement_corpus_id}`; `{}` does not own the required local compatibility contract",
                            fixture.corpus_id
                        ),
                        replacement_corpus_id: replacement_corpus_id.to_string(),
                        replacement_corpus_family_id: replacement_corpus_family_id.to_string(),
                    });
                }
            }
            (None, None, Some(benchmark_scope_id), "asset_backed") => {
                asset_backed_stages.push(LocalAssetBackedStageReport {
                    stage_id: stage.stage_id.clone(),
                    readiness_kind: stage.readiness_kind.clone(),
                    benchmark_scope_id: benchmark_scope_id.to_string(),
                    reason: stage.compatibility_note.clone(),
                });
            }
            (None, None, None, "planner_only") => {
                planner_only_stages.push(LocalPlannerOnlyStageReport {
                    stage_id: stage.stage_id.clone(),
                    readiness_kind: stage.readiness_kind.clone(),
                    reason: stage.compatibility_note.clone(),
                });
            }
            _ => {}
        }
    }

    LocalCorpusSkipReport {
        schema_version: LOCAL_CORPUS_SKIP_REPORT_SCHEMA_VERSION,
        matrix_path: compatibility.matrix_path.clone(),
        output_path: path_relative_to_repo(repo_root, output_path),
        fixture_count: fixtures.len(),
        stage_count: compatibility.stage_count,
        skip_count: skips.len(),
        asset_backed_stage_count: asset_backed_stages.len(),
        planner_only_stage_count: planner_only_stages.len(),
        fixtures,
        skips,
        asset_backed_stages,
        planner_only_stages,
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{render_corpus_skip_report_path, DEFAULT_CORPUS_SKIP_REPORT_PATH};
    use crate::commands::benchmark::local_corpus_stage_compatibility::DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH;

    fn repo_root() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
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
        assert_eq!(report.skip_count, 343);
        assert_eq!(report.asset_backed_stage_count, 1);
        assert_eq!(report.planner_only_stage_count, 1);
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
            report.planner_only_stages.iter().any(|stage| {
                stage.stage_id == "fastq.report_qc"
                    && stage.reason.contains("not yet owned by any corpus fixture manifest")
            }),
            "planner-only stages must stay explicit in the skip report"
        );
    }
}
