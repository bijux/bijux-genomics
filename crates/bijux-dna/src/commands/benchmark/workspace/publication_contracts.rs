use std::path::Path;

use anyhow::Result;
use bijux_dna_api::v1::api::bench::{
    benchmark_publication_contract_from_config, benchmark_publication_contracts_from_config,
    benchmark_publication_exclusions_from_config,
};

use super::{load_benchmark_publication_config, CorpusBenchmarkContract, CorpusBenchmarkExclusion};

pub(crate) fn benchmark_publication_contract(
    cwd: &Path,
    explicit_path: Option<&Path>,
    corpus_id: &str,
    stage_id: &str,
) -> Result<CorpusBenchmarkContract> {
    let publication = load_benchmark_publication_config(cwd, explicit_path)?;
    benchmark_publication_contract_from_config(&publication, corpus_id, stage_id)
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
