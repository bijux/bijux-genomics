use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::models::{DossierIndex, DossierStageEntry};
use super::{
    benchmark_publication_contracts, benchmark_runtime_corpus_dir_name,
    benchmark_stage_run_relative_root, classify_run_root_source, load_benchmark_config,
    publication_artifact_file_name, publication_stage_docs_root, relative_to_repo_root,
    workspace_local_cache_mirror_root, workspace_local_results_root, workspace_remote_corpus_root,
    workspace_remote_results_root, BenchmarkWorkspaceConfig, CorpusBenchmarkContract,
};

pub(super) fn write_corpus_fastq_dossier_index(
    cwd: &Path,
    explicit_config: Option<&Path>,
    docs_root: &Path,
    corpus_id: &str,
) -> Result<()> {
    let config = load_benchmark_config(cwd, explicit_config)?;
    let workspace = &config.workspace;
    let contracts = benchmark_publication_contracts(cwd, explicit_config, corpus_id)?;

    let stages = contracts
        .iter()
        .map(|contract| build_dossier_stage_entry(cwd, docs_root, workspace, corpus_id, contract))
        .collect::<Result<Vec<_>>>()?;
    let index = DossierIndex {
        corpus_id: corpus_id.to_string(),
        stage_count: stages.len(),
        published_stage_count: stages.iter().filter(|stage| stage.status == "published").count(),
        missing_stage_count: stages.iter().filter(|stage| stage.status != "published").count(),
        stages,
    };

    fs::create_dir_all(docs_root).with_context(|| format!("create {}", docs_root.display()))?;
    let json_path = docs_root.join(publication_artifact_file_name(corpus_id, "dossier-index.json"));
    fs::write(&json_path, format!("{}\n", serde_json::to_string_pretty(&index)?))
        .with_context(|| format!("write {}", json_path.display()))?;

    let markdown_path =
        docs_root.join(publication_artifact_file_name(corpus_id, "dossier-index.md"));
    fs::write(&markdown_path, render_dossier_index_markdown(&index))
        .with_context(|| format!("write {}", markdown_path.display()))?;
    Ok(())
}

pub(super) fn build_dossier_stage_entry(
    repo_root: &Path,
    docs_root: &Path,
    workspace: &BenchmarkWorkspaceConfig,
    corpus_id: &str,
    contract: &CorpusBenchmarkContract,
) -> Result<DossierStageEntry> {
    let stage_docs_root = publication_stage_docs_root(docs_root, &contract.stage_id, corpus_id);
    let summary_path = stage_docs_root.join("summary.json");
    let dossier_path = resolve_existing_dossier_path(&stage_docs_root);

    let remote_corpus_root = workspace_remote_corpus_root(workspace)?;
    let remote_corpus_id = benchmark_runtime_corpus_dir_name(workspace, corpus_id)?;
    let expected_remote_run_root =
        workspace_remote_results_root(workspace)?.join(benchmark_stage_run_relative_root(
            workspace,
            "remote",
            &remote_corpus_id,
            &contract.stage_id,
        )?);
    let expected_local_cache_mirror_run_root =
        workspace_local_cache_mirror_root(workspace)?.join(benchmark_stage_run_relative_root(
            workspace,
            "local-cache",
            &remote_corpus_id,
            &contract.stage_id,
        )?);
    let expected_local_results_run_root =
        workspace_local_results_root(workspace)?.join(benchmark_stage_run_relative_root(
            workspace,
            "local-archive",
            &remote_corpus_id,
            &contract.stage_id,
        )?);
    let mut entry = DossierStageEntry {
        stage_id: contract.stage_id.clone(),
        sample_scope: contract.sample_scope.clone(),
        status: "missing".to_string(),
        summary_path: relative_to_repo_root(&summary_path, repo_root),
        dossier_path: relative_to_repo_root(&dossier_path, repo_root),
        expected_remote_run_root: expected_remote_run_root.display().to_string(),
        expected_local_cache_mirror_run_root: expected_local_cache_mirror_run_root
            .display()
            .to_string(),
        expected_local_results_run_root: expected_local_results_run_root.display().to_string(),
        generated_at_utc: None,
        platform: None,
        corpus_root: None,
        run_root: None,
        run_root_source: None,
    };

    if !summary_path.is_file() {
        return Ok(entry);
    }

    let summary: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&summary_path)
            .with_context(|| format!("read {}", summary_path.display()))?,
    )
    .with_context(|| format!("parse {}", summary_path.display()))?;
    let run_root = summary
        .get("run_root")
        .and_then(|value| value.as_str())
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from);

    entry.status = "published".to_string();
    entry.generated_at_utc =
        summary.get("generated_at_utc").and_then(|value| value.as_str()).map(ToOwned::to_owned);
    entry.platform =
        summary.get("platform").and_then(|value| value.as_str()).map(ToOwned::to_owned);
    entry.corpus_root =
        summary.get("corpus_root").and_then(|value| value.as_str()).map(ToOwned::to_owned);
    entry.run_root = run_root.as_ref().map(|value| value.display().to_string());
    entry.run_root_source = run_root.as_ref().map(|path| {
        classify_run_root_source(
            path,
            &expected_remote_run_root,
            &expected_local_cache_mirror_run_root,
            &expected_local_results_run_root,
            &remote_corpus_root,
        )
    });
    Ok(entry)
}

pub(super) fn resolve_existing_dossier_path(stage_docs_root: &Path) -> PathBuf {
    stage_docs_root.join("benchmark.md")
}

fn render_dossier_index_markdown(index: &DossierIndex) -> String {
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
            lines.push(format!(
                "- `{}`: `{}` from `{}`",
                stage.stage_id,
                stage.generated_at_utc.as_deref().unwrap_or("missing"),
                stage.run_root_source.as_deref().unwrap_or("missing")
            ));
            lines.push(format!(
                "  - published run root: `{}`",
                stage.run_root.as_deref().unwrap_or("missing")
            ));
            lines.push(format!(
                "  - expected remote run root: `{}`",
                stage.expected_remote_run_root
            ));
            lines.push(format!(
                "  - expected local cache mirror run root: `{}`",
                stage.expected_local_cache_mirror_run_root
            ));
        } else {
            lines.push(format!("- `{}`: `missing`", stage.stage_id));
            lines.push(format!(
                "  - expected remote run root: `{}`",
                stage.expected_remote_run_root
            ));
        }
    }
    lines.join("\n") + "\n"
}
