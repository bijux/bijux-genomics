//! Owner: bijux-dna-bench
//! Deterministic FASTQ corpus dossier-index publication.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_dna_bench_model::{
    validate_benchmark_dossier_index, BenchmarkDossierIndex, BenchmarkDossierStageEntry,
    BENCHMARK_DOSSIER_INDEX_SCHEMA_V1,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkDossierStagePublicationRequest {
    pub repository_root: PathBuf,
    pub stage_docs_root: PathBuf,
    pub stage_id: String,
    pub sample_scope: String,
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
/// Returns an error when an existing summary cannot be read or parsed.
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
    let summary: serde_json::Value = serde_json::from_slice(
        &fs::read(&summary_path).with_context(|| format!("read {}", summary_path.display()))?,
    )
    .with_context(|| format!("parse {}", summary_path.display()))?;
    let run_root = summary
        .get("run_root")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from);
    entry.status = "published".to_string();
    entry.generated_at_utc = string_field(&summary, "generated_at_utc");
    entry.platform = string_field(&summary, "platform");
    entry.corpus_root = string_field(&summary, "corpus_root");
    entry.run_root = run_root.as_ref().map(|path| path.display().to_string());
    entry.run_root_source = run_root.as_ref().map(|path| classify_run_root_source(path, request));
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
        .map(build_benchmark_dossier_stage_entry)
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
    fs::create_dir_all(&request.docs_root)
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

fn string_field(value: &serde_json::Value, field: &str) -> Option<String> {
    value.get(field).and_then(serde_json::Value::as_str).map(ToOwned::to_owned)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn request(root: &Path) -> BenchmarkDossierStagePublicationRequest {
        BenchmarkDossierStagePublicationRequest {
            repository_root: root.to_path_buf(),
            stage_docs_root: root.join("docs/benchmark/fastq.validate_reads/corpus-01"),
            stage_id: "fastq.validate_reads".to_string(),
            sample_scope: "full".to_string(),
            remote_corpus_root: root.join("remote/corpus-01"),
            expected_remote_run_root: root.join("remote/results/corpus-01/fastq.validate_reads"),
            expected_local_cache_mirror_run_root: root
                .join("cache/results/corpus-01/fastq.validate_reads"),
            expected_local_results_run_root: root.join("archive/corpus-01/fastq.validate_reads"),
        }
    }

    #[test]
    fn dossier_entry_classifies_published_remote_root() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let request = request(temporary.path());
        fs::create_dir_all(&request.stage_docs_root)?;
        fs::write(
            request.stage_docs_root.join("summary.json"),
            serde_json::to_vec(&serde_json::json!({
                "generated_at_utc": "2026-08-14T00:00:00Z",
                "platform": "lunarc",
                "corpus_root": request.remote_corpus_root,
                "run_root": request.expected_remote_run_root,
            }))?,
        )?;
        let entry = build_benchmark_dossier_stage_entry(&request)?;
        assert_eq!(entry.status, "published");
        assert_eq!(entry.run_root_source.as_deref(), Some("remote-results-root"));
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
}
