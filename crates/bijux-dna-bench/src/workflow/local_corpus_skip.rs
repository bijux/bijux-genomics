//! Owner: bijux-dna-bench
//! Deterministic local corpus-skip report derivation and publication.

use std::path::Path;

use anyhow::{Context, Result};
use bijux_dna_bench_model::{
    validate_benchmark_local_corpus_skip_report, BenchmarkLocalAssetBackedStage,
    BenchmarkLocalCorpusFixture, BenchmarkLocalCorpusSkip, BenchmarkLocalCorpusSkipReport,
    BenchmarkLocalCorpusStageCompatibilityReport, BenchmarkLocalPlannerOnlyStage,
    BENCHMARK_LOCAL_CORPUS_SKIP_REPORT_SCHEMA_V1,
};

/// Inputs for one governed local corpus-skip report publication.
pub struct BenchmarkLocalCorpusSkipPublicationRequest<'a> {
    pub repository_root: &'a Path,
    pub output_path: &'a Path,
    pub compatibility: &'a BenchmarkLocalCorpusStageCompatibilityReport,
}

/// Derive and validate one corpus-skip report from an admitted compatibility report.
///
/// # Errors
/// Returns an error when the derived report violates its persisted model contract.
pub fn derive_benchmark_local_corpus_skip_report(
    compatibility: &BenchmarkLocalCorpusStageCompatibilityReport,
    output_path: &str,
) -> Result<BenchmarkLocalCorpusSkipReport> {
    let fixtures = compatibility
        .fixtures
        .iter()
        .map(|fixture| BenchmarkLocalCorpusFixture {
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
                    skips.push(BenchmarkLocalCorpusSkip {
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
                asset_backed_stages.push(BenchmarkLocalAssetBackedStage {
                    stage_id: stage.stage_id.clone(),
                    readiness_kind: stage.readiness_kind.clone(),
                    benchmark_scope_id: benchmark_scope_id.to_string(),
                    reason: stage.compatibility_note.clone(),
                });
            }
            (None, None, None, "planner_only") => {
                planner_only_stages.push(BenchmarkLocalPlannerOnlyStage {
                    stage_id: stage.stage_id.clone(),
                    readiness_kind: stage.readiness_kind.clone(),
                    reason: stage.compatibility_note.clone(),
                });
            }
            _ => {}
        }
    }

    let report = BenchmarkLocalCorpusSkipReport {
        schema_version: BENCHMARK_LOCAL_CORPUS_SKIP_REPORT_SCHEMA_V1.to_string(),
        matrix_path: compatibility.matrix_path.clone(),
        output_path: output_path.to_string(),
        fixture_count: fixtures.len(),
        stage_count: compatibility.stage_count,
        skip_count: skips.len(),
        asset_backed_stage_count: asset_backed_stages.len(),
        planner_only_stage_count: planner_only_stages.len(),
        fixtures,
        skips,
        asset_backed_stages,
        planner_only_stages,
    };
    validate_benchmark_local_corpus_skip_report(&report)
        .context("validate derived local corpus-skip report")?;
    Ok(report)
}

/// Derive, validate, and atomically publish one local corpus-skip report.
///
/// # Errors
/// Returns an error when derivation, directory admission, or atomic publication fails.
pub fn publish_benchmark_local_corpus_skip_report(
    request: &BenchmarkLocalCorpusSkipPublicationRequest<'_>,
) -> Result<BenchmarkLocalCorpusSkipReport> {
    let output_path = request
        .output_path
        .strip_prefix(request.repository_root)
        .unwrap_or(request.output_path)
        .display()
        .to_string();
    let report = derive_benchmark_local_corpus_skip_report(request.compatibility, &output_path)?;
    if let Some(parent) = request.output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(request.output_path, &report)
        .with_context(|| format!("write {}", request.output_path.display()))?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use bijux_dna_bench_model::{
        BenchmarkLocalCorpusStageCompatibilityEntry, BenchmarkLocalCorpusStageValidatedFixture,
        BENCHMARK_LOCAL_CORPUS_STAGE_COMPATIBILITY_SCHEMA_V1,
    };

    use super::*;

    fn compatibility() -> BenchmarkLocalCorpusStageCompatibilityReport {
        BenchmarkLocalCorpusStageCompatibilityReport {
            schema_version: BENCHMARK_LOCAL_CORPUS_STAGE_COMPATIBILITY_SCHEMA_V1.to_string(),
            matrix_path: "configs/bench/local/corpus-stage-compatibility.toml".to_string(),
            fixture_count: 2,
            stage_count: 3,
            fixture_backed_stage_count: 1,
            asset_backed_stage_count: 1,
            planner_only_stage_count: 1,
            corpus_family_counts: BTreeMap::from([("short-read".to_string(), 1)]),
            valid: true,
            fixtures: ["corpus-a", "corpus-b"]
                .into_iter()
                .map(|fixture_id| BenchmarkLocalCorpusStageValidatedFixture {
                    corpus_family_id: "short-read".to_string(),
                    fixture_id: fixture_id.to_string(),
                    fixture_manifest: format!("fixtures/{fixture_id}.json"),
                    summary: format!("{fixture_id} fixture"),
                })
                .collect(),
            stages: vec![
                BenchmarkLocalCorpusStageCompatibilityEntry {
                    stage_id: "fastq.validate_reads".to_string(),
                    readiness_kind: "smoke".to_string(),
                    compatibility_kind: "fixture".to_string(),
                    corpus_family_id: Some("short-read".to_string()),
                    fixture_id: Some("corpus-a".to_string()),
                    benchmark_scope_id: None,
                    compatibility_note: "fixture-backed".to_string(),
                },
                BenchmarkLocalCorpusStageCompatibilityEntry {
                    stage_id: "fastq.index_reference".to_string(),
                    readiness_kind: "dry_or_smoke".to_string(),
                    compatibility_kind: "asset_backed".to_string(),
                    corpus_family_id: None,
                    fixture_id: None,
                    benchmark_scope_id: Some("reference-index-assets".to_string()),
                    compatibility_note: "owned reference assets".to_string(),
                },
                BenchmarkLocalCorpusStageCompatibilityEntry {
                    stage_id: "fastq.report_qc".to_string(),
                    readiness_kind: "dry_run".to_string(),
                    compatibility_kind: "planner_only".to_string(),
                    corpus_family_id: None,
                    fixture_id: None,
                    benchmark_scope_id: None,
                    compatibility_note: "planning contract only".to_string(),
                },
            ],
        }
    }

    #[test]
    fn derives_each_corpus_skip_disposition_without_omitting_stages() -> Result<()> {
        let report = derive_benchmark_local_corpus_skip_report(
            &compatibility(),
            "artifacts/benchmark-smoke/corpus-skip-report.json",
        )?;

        assert_eq!(report.fixture_count, 2);
        assert_eq!(report.stage_count, 3);
        assert_eq!(report.skip_count, 1);
        assert_eq!(report.skips[0].corpus_id, "corpus-b");
        assert_eq!(report.skips[0].replacement_corpus_id, "corpus-a");
        assert_eq!(report.asset_backed_stage_count, 1);
        assert_eq!(report.planner_only_stage_count, 1);
        Ok(())
    }

    #[test]
    fn publishes_validated_report_with_repository_relative_identity() -> Result<()> {
        let root = tempfile::tempdir()?;
        let output_path = root.path().join("artifacts/corpus-skip-report.json");
        let compatibility = compatibility();
        let report = publish_benchmark_local_corpus_skip_report(
            &BenchmarkLocalCorpusSkipPublicationRequest {
                repository_root: root.path(),
                output_path: &output_path,
                compatibility: &compatibility,
            },
        )?;

        assert_eq!(report.output_path, "artifacts/corpus-skip-report.json");
        assert_eq!(
            serde_json::from_slice::<BenchmarkLocalCorpusSkipReport>(&std::fs::read(output_path)?)?,
            report
        );
        Ok(())
    }
}
