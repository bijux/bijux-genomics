use std::path::Path;

use anyhow::{anyhow, Result};

use super::{
    benchmark_publication_corpus_key, load_benchmark_publication_config,
    BenchmarkPublicationConfig, CorpusBenchmarkContract, CorpusBenchmarkExclusion,
};

pub(crate) fn benchmark_publication_contract(
    cwd: &Path,
    explicit_path: Option<&Path>,
    corpus_id: &str,
    stage_id: &str,
) -> Result<CorpusBenchmarkContract> {
    let publication = load_benchmark_publication_config(cwd, explicit_path)?;
    benchmark_publication_contracts_from_config(&publication, corpus_id)?
        .into_iter()
        .find(|row| row.stage_id == stage_id)
        .ok_or_else(|| anyhow!("missing {corpus_id} publication contract for {stage_id}"))
}

pub(crate) fn benchmark_publication_contracts(
    cwd: &Path,
    explicit_path: Option<&Path>,
    corpus_id: &str,
) -> Result<Vec<CorpusBenchmarkContract>> {
    let publication = load_benchmark_publication_config(cwd, explicit_path)?;
    benchmark_publication_contracts_from_config(&publication, corpus_id)
}

pub(crate) fn benchmark_publication_exclusions(
    cwd: &Path,
    explicit_path: Option<&Path>,
    corpus_id: &str,
) -> Result<Vec<CorpusBenchmarkExclusion>> {
    let publication = load_benchmark_publication_config(cwd, explicit_path)?;
    benchmark_publication_exclusions_from_config(&publication, corpus_id)
}

fn benchmark_publication_contracts_from_config(
    publication: &BenchmarkPublicationConfig,
    corpus_id: &str,
) -> Result<Vec<CorpusBenchmarkContract>> {
    let key = benchmark_publication_corpus_key(corpus_id);
    publication
        .corpora
        .get(&key)
        .cloned()
        .map(|entry| entry.contracts)
        .ok_or_else(|| anyhow!("benchmark publication config is missing [{key}]"))
}

fn benchmark_publication_exclusions_from_config(
    publication: &BenchmarkPublicationConfig,
    corpus_id: &str,
) -> Result<Vec<CorpusBenchmarkExclusion>> {
    let key = benchmark_publication_corpus_key(corpus_id);
    publication
        .corpora
        .get(&key)
        .cloned()
        .map(|entry| entry.exclusions)
        .ok_or_else(|| anyhow!("benchmark publication config is missing [{key}]"))
}
