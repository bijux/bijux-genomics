use std::path::{Path, PathBuf};

use anyhow::Result;
use bijux_dna_api::v1::api::bench::{
    benchmark_dossier_path, build_benchmark_dossier_stage_entry, publish_benchmark_dossier_index,
    BenchmarkDossierIndexPublicationRequest, BenchmarkDossierStageEntry as DossierStageEntry,
    BenchmarkDossierStagePublicationRequest,
};

use super::{
    benchmark_publication_contracts, benchmark_runtime_corpus_dir_name,
    benchmark_stage_run_relative_root, load_benchmark_config, publication_stage_docs_root,
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
    let stages = benchmark_publication_contracts(cwd, explicit_config, corpus_id)?
        .iter()
        .map(|contract| {
            dossier_stage_request(cwd, docs_root, &config.workspace, corpus_id, contract)
        })
        .collect::<Result<Vec<_>>>()?;
    publish_benchmark_dossier_index(&BenchmarkDossierIndexPublicationRequest {
        docs_root: docs_root.to_path_buf(),
        corpus_id: corpus_id.to_string(),
        stages,
    })?;
    Ok(())
}

pub(super) fn build_dossier_stage_entry(
    repo_root: &Path,
    docs_root: &Path,
    workspace: &BenchmarkWorkspaceConfig,
    corpus_id: &str,
    contract: &CorpusBenchmarkContract,
) -> Result<DossierStageEntry> {
    build_benchmark_dossier_stage_entry(&dossier_stage_request(
        repo_root, docs_root, workspace, corpus_id, contract,
    )?)
}

fn dossier_stage_request(
    repo_root: &Path,
    docs_root: &Path,
    workspace: &BenchmarkWorkspaceConfig,
    corpus_id: &str,
    contract: &CorpusBenchmarkContract,
) -> Result<BenchmarkDossierStagePublicationRequest> {
    let remote_corpus_root = workspace_remote_corpus_root(workspace)?;
    let remote_corpus_id = benchmark_runtime_corpus_dir_name(workspace, corpus_id)?;
    let run_relative = |layout| {
        benchmark_stage_run_relative_root(workspace, layout, &remote_corpus_id, &contract.stage_id)
    };
    Ok(BenchmarkDossierStagePublicationRequest {
        repository_root: repo_root.to_path_buf(),
        stage_docs_root: publication_stage_docs_root(docs_root, &contract.stage_id, corpus_id),
        corpus_id: corpus_id.to_string(),
        stage_id: contract.stage_id.clone(),
        scenario_id: contract.scenario_id.clone(),
        sample_scope: contract.sample_scope.clone(),
        tool_ids: contract.tools.clone(),
        remote_corpus_root,
        expected_remote_run_root: workspace_remote_results_root(workspace)?
            .join(run_relative("remote")?),
        expected_local_cache_mirror_run_root: workspace_local_cache_mirror_root(workspace)?
            .join(run_relative("local-cache")?),
        expected_local_results_run_root: workspace_local_results_root(workspace)?
            .join(run_relative("local-archive")?),
    })
}

pub(super) fn resolve_existing_dossier_path(stage_docs_root: &Path) -> PathBuf {
    benchmark_dossier_path(stage_docs_root)
}
