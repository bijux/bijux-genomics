//! Owner: bijux-dna-bench
//! Deterministic FASTQ corpus dossier-index publication.

use std::path::{Path, PathBuf};

use anyhow::{ensure, Context, Result};
use bijux_dna_bench_model::{
    validate_benchmark_corpus_dossier_summary, validate_benchmark_dossier_index,
    BenchmarkCorpusDossierSummary, BenchmarkDossierIndex, BenchmarkDossierStageEntry,
    BENCHMARK_DOSSIER_INDEX_SCHEMA_V1,
};

const MAX_DOSSIER_SUMMARY_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkDossierStagePublicationRequest {
    pub repository_root: PathBuf,
    pub stage_docs_root: PathBuf,
    pub corpus_id: String,
    pub stage_id: String,
    pub scenario_id: String,
    pub sample_scope: String,
    pub tool_ids: Vec<String>,
    pub remote_corpus_root: PathBuf,
    pub expected_remote_run_root: PathBuf,
    pub expected_local_cache_mirror_run_root: PathBuf,
    pub expected_local_results_run_root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkDossierIndexPublicationRequest {
    pub docs_root: PathBuf,
    pub corpus_id: String,
    pub stages: Vec<BenchmarkDossierStagePublicationRequest>,
}

#[must_use]
pub fn benchmark_dossier_path(stage_docs_root: &Path) -> PathBuf {
    stage_docs_root.join("benchmark.md")
}

/// Build one dossier-index row from resolved roots and an optional published summary.
///
/// # Errors
/// Returns an error when an existing summary cannot be read, parsed, validated, or matched to the
/// requested corpus, stage, scenario, and sample scope.
pub fn build_benchmark_dossier_stage_entry(
    request: &BenchmarkDossierStagePublicationRequest,
) -> Result<BenchmarkDossierStageEntry> {
    let summary_path = request.stage_docs_root.join("summary.json");
    let mut entry = BenchmarkDossierStageEntry {
        stage_id: request.stage_id.clone(),
        sample_scope: request.sample_scope.clone(),
        status: "missing".to_string(),
        summary_path: relative_path(&summary_path, &request.repository_root),
        dossier_path: relative_path(
            &benchmark_dossier_path(&request.stage_docs_root),
            &request.repository_root,
        ),
        expected_remote_run_root: request.expected_remote_run_root.display().to_string(),
        expected_local_cache_mirror_run_root: request
            .expected_local_cache_mirror_run_root
            .display()
            .to_string(),
        expected_local_results_run_root: request
            .expected_local_results_run_root
            .display()
            .to_string(),
        generated_at_utc: None,
        platform: None,
        corpus_root: None,
        run_root: None,
        run_root_source: None,
    };
    if !summary_path.is_file() {
        return Ok(entry);
    }
    let summary: BenchmarkCorpusDossierSummary = serde_json::from_slice(
        &bijux_dna_infra::read_to_end_bounded(&summary_path, MAX_DOSSIER_SUMMARY_BYTES)
            .with_context(|| format!("read {}", summary_path.display()))?,
    )
    .with_context(|| format!("parse {}", summary_path.display()))?;
    validate_benchmark_corpus_dossier_summary(&summary)
        .with_context(|| format!("validate {}", summary_path.display()))?;
    ensure!(
        summary.corpus_id == request.corpus_id,
        "dossier summary corpus drift: expected `{}`, found `{}`",
        request.corpus_id,
        summary.corpus_id
    );
    ensure!(
        summary.stage_id == request.stage_id,
        "dossier summary stage drift: expected `{}`, found `{}`",
        request.stage_id,
        summary.stage_id
    );
    ensure!(
        summary.scenario_id == request.scenario_id,
        "dossier summary scenario drift: expected `{}`, found `{}`",
        request.scenario_id,
        summary.scenario_id
    );
    ensure!(
        summary.sample_scope == request.sample_scope,
        "dossier summary sample-scope drift: expected `{}`, found `{}`",
        request.sample_scope,
        summary.sample_scope
    );
    ensure!(
        summary.tools == request.tool_ids,
        "dossier summary tool-roster drift: expected {:?}, found {:?}",
        request.tool_ids,
        summary.tools
    );
    ensure!(
        Path::new(&summary.corpus_root) == request.remote_corpus_root,
        "dossier summary corpus-root drift: expected `{}`, found `{}`",
        request.remote_corpus_root.display(),
        summary.corpus_root
    );
    let run_root = PathBuf::from(&summary.run_root);
    entry.status = "published".to_string();
    entry.generated_at_utc = Some(summary.generated_at_utc);
    entry.platform = Some(summary.platform);
    entry.corpus_root = Some(summary.corpus_root);
    entry.run_root = Some(run_root.display().to_string());
    entry.run_root_source = Some(classify_run_root_source(&run_root, request));
    Ok(entry)
}

/// Build, validate, render, and atomically publish one corpus dossier index.
///
/// # Errors
/// Returns an error when stage evidence is invalid or publication cannot be written.
pub fn publish_benchmark_dossier_index(
    request: &BenchmarkDossierIndexPublicationRequest,
) -> Result<BenchmarkDossierIndex> {
    let stages = request
        .stages
        .iter()
        .map(|stage| {
            ensure!(
                stage.corpus_id == request.corpus_id,
                "dossier stage request corpus drift: expected `{}`, found `{}`",
                request.corpus_id,
                stage.corpus_id
            );
            build_benchmark_dossier_stage_entry(stage)
        })
        .collect::<Result<Vec<_>>>()?;
    let index = BenchmarkDossierIndex {
        schema_version: BENCHMARK_DOSSIER_INDEX_SCHEMA_V1.to_string(),
        corpus_id: request.corpus_id.clone(),
        stage_count: stages.len(),
        published_stage_count: stages.iter().filter(|stage| stage.status == "published").count(),
        missing_stage_count: stages.iter().filter(|stage| stage.status != "published").count(),
        stages,
    };
    validate_benchmark_dossier_index(&index).context("validate benchmark dossier index")?;
    bijux_dna_infra::ensure_dir(&request.docs_root)
        .with_context(|| format!("create {}", request.docs_root.display()))?;
    let json_path = request.docs_root.join(format!("{}-dossier-index.json", request.corpus_id));
    let markdown_path = request.docs_root.join(format!("{}-dossier-index.md", request.corpus_id));
    let json = serde_json::to_string_pretty(&index)? + "\n";
    bijux_dna_infra::atomic_write_bytes(&json_path, json.as_bytes())?;
    bijux_dna_infra::atomic_write_bytes(
        &markdown_path,
        render_benchmark_dossier_index_markdown(&index).as_bytes(),
    )?;
    Ok(index)
}

#[must_use]
pub fn render_benchmark_dossier_index_markdown(index: &BenchmarkDossierIndex) -> String {
    let mut lines = vec![
        format!("# `{}` FASTQ dossier index", index.corpus_id),
        String::new(),
        format!("- Governed publication stages: `{}`", index.stage_count),
        format!("- Published summaries: `{}`", index.published_stage_count),
        format!("- Missing summaries: `{}`", index.missing_stage_count),
        String::new(),
        "## Stage index".to_string(),
        String::new(),
    ];
    for stage in &index.stages {
        if stage.status == "published" {
            lines.extend([
                format!(
                    "- `{}`: `{}` from `{}`",
                    stage.stage_id,
                    stage.generated_at_utc.as_deref().unwrap_or("missing"),
                    stage.run_root_source.as_deref().unwrap_or("missing")
                ),
                format!(
                    "  - published run root: `{}`",
                    stage.run_root.as_deref().unwrap_or("missing")
                ),
                format!("  - expected remote run root: `{}`", stage.expected_remote_run_root),
                format!(
                    "  - expected local cache mirror run root: `{}`",
                    stage.expected_local_cache_mirror_run_root
                ),
            ]);
        } else {
            lines.extend([
                format!("- `{}`: `missing`", stage.stage_id),
                format!("  - expected remote run root: `{}`", stage.expected_remote_run_root),
            ]);
        }
    }
    lines.join("\n") + "\n"
}

fn classify_run_root_source(
    run_root: &Path,
    request: &BenchmarkDossierStagePublicationRequest,
) -> String {
    if run_root == request.expected_local_cache_mirror_run_root {
        "local-cache-mirror"
    } else if run_root == request.expected_local_results_run_root {
        "local-results-root"
    } else if run_root == request.expected_remote_run_root {
        "remote-results-root"
    } else if request.remote_corpus_root.parent().is_some_and(|root| run_root.starts_with(root)) {
        "remote-custom"
    } else {
        "custom"
    }
    .to_string()
}

fn relative_path(path: &Path, repository_root: &Path) -> String {
    path.strip_prefix(repository_root).unwrap_or(path).display().to_string()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;

    use bijux_dna_bench_model::{
        BenchmarkCorpusDossierHeadline, BenchmarkCorpusDossierToolSummary,
        BENCHMARK_CORPUS_DOSSIER_SUMMARY_SCHEMA_V1,
    };

    use super::*;

    fn request(root: &Path) -> BenchmarkDossierStagePublicationRequest {
        BenchmarkDossierStagePublicationRequest {
            repository_root: root.to_path_buf(),
            stage_docs_root: root.join("docs/benchmark/fastq.validate_reads/corpus-01"),
            corpus_id: "corpus-01".to_string(),
            stage_id: "fastq.validate_reads".to_string(),
            scenario_id: "validate-reads".to_string(),
            sample_scope: "full".to_string(),
            tool_ids: vec!["fastqc".to_string()],
            remote_corpus_root: root.join("remote/corpus-01"),
            expected_remote_run_root: root.join("remote/results/corpus-01/fastq.validate_reads"),
            expected_local_cache_mirror_run_root: root
                .join("cache/results/corpus-01/fastq.validate_reads"),
            expected_local_results_run_root: root.join("archive/corpus-01/fastq.validate_reads"),
        }
    }

    fn summary(request: &BenchmarkDossierStagePublicationRequest) -> BenchmarkCorpusDossierSummary {
        BenchmarkCorpusDossierSummary {
            schema_version: BENCHMARK_CORPUS_DOSSIER_SUMMARY_SCHEMA_V1.to_string(),
            corpus_id: request.corpus_id.clone(),
            stage_id: request.stage_id.clone(),
            scenario_id: request.scenario_id.clone(),
            generated_at_utc: "2026-08-14T00:00:00Z".to_string(),
            platform: "lunarc".to_string(),
            corpus_root: request.remote_corpus_root.display().to_string(),
            run_root: request.expected_remote_run_root.display().to_string(),
            sample_scope: request.sample_scope.clone(),
            samples_total: 1,
            samples_failed: 0,
            tools: request.tool_ids.clone(),
            cohort_counts: BTreeMap::from([("ancient_pe".to_string(), 1)]),
            era_counts: BTreeMap::from([("ancient".to_string(), 1)]),
            layout_counts: BTreeMap::from([("pe".to_string(), 1)]),
            tool_summary: vec![BenchmarkCorpusDossierToolSummary {
                tool: "fastqc".to_string(),
                records: 1,
                pass_rate: Some(1.0),
                median_runtime_s: Some(2.0),
                mean_runtime_s: Some(2.0),
                max_runtime_s: Some(2.0),
            }],
            headline: BenchmarkCorpusDossierHeadline {
                fastest_tool: Some("fastqc".to_string()),
                fastest_runtime_s: Some(2.0),
                highest_pass_rate_tool: Some("fastqc".to_string()),
                highest_pass_rate: Some(1.0),
            },
        }
    }

    #[test]
    fn dossier_entry_classifies_published_remote_root() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let request = request(temporary.path());
        fs::create_dir_all(&request.stage_docs_root)?;
        fs::write(
            request.stage_docs_root.join("summary.json"),
            serde_json::to_vec(&summary(&request))?,
        )?;
        let entry = build_benchmark_dossier_stage_entry(&request)?;
        assert_eq!(entry.status, "published");
        assert_eq!(entry.run_root_source.as_deref(), Some("remote-results-root"));
        Ok(())
    }

    #[test]
    fn dossier_entry_rejects_summary_stage_drift() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let request = request(temporary.path());
        fs::create_dir_all(&request.stage_docs_root)?;
        let mut summary = summary(&request);
        summary.stage_id = "fastq.trim".to_string();
        fs::write(request.stage_docs_root.join("summary.json"), serde_json::to_vec(&summary)?)?;
        let error = build_benchmark_dossier_stage_entry(&request)
            .err()
            .context("stage drift must be rejected")?;
        assert!(error.to_string().contains("dossier summary stage drift"));
        Ok(())
    }

    #[test]
    fn dossier_publication_accounts_for_missing_stage() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let publication = BenchmarkDossierIndexPublicationRequest {
            docs_root: temporary.path().join("docs/benchmark"),
            corpus_id: "corpus-01".to_string(),
            stages: vec![request(temporary.path())],
        };
        let index = publish_benchmark_dossier_index(&publication)?;
        assert_eq!((index.published_stage_count, index.missing_stage_count), (0, 1));
        assert!(publication.docs_root.join("corpus-01-dossier-index.json").is_file());
        assert!(publication.docs_root.join("corpus-01-dossier-index.md").is_file());
        Ok(())
    }

    #[test]
    fn dossier_publication_rejects_stage_request_corpus_drift() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let mut stage = request(temporary.path());
        stage.corpus_id = "other-corpus".to_string();
        let publication = BenchmarkDossierIndexPublicationRequest {
            docs_root: temporary.path().join("docs/benchmark"),
            corpus_id: "corpus-01".to_string(),
            stages: vec![stage],
        };
        let error = publish_benchmark_dossier_index(&publication)
            .err()
            .context("stage-request corpus drift must be rejected")?;
        assert!(error.to_string().contains("dossier stage request corpus drift"));
        Ok(())
    }
}
