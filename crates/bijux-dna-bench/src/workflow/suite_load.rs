//! Owner: bijux-dna-bench
//! Repository-backed loading for benchmark observations.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::repo::bench_corpora_dir;
use crate::repo::RunRepository;
use bijux_dna_bench_model::contract::validate_corpus_manifest;
use bijux_dna_bench_model::{BenchmarkCorpusManifest, BenchmarkObservation};

/// Load observations for a suite from a repository.
///
/// # Errors
/// Returns an error if the repository cannot load observations.
pub fn load_suite(
    repo: &dyn RunRepository,
    run_ids: Option<&[String]>,
) -> Result<Vec<BenchmarkObservation>> {
    let ids = match run_ids {
        Some(ids) => ids.to_vec(),
        None => repo.list_runs()?,
    };
    let mut observations = Vec::new();
    for run_id in ids {
        observations.extend(repo.load_observations(&run_id)?);
    }
    Ok(observations)
}

/// Load and validate a single benchmark corpus manifest.
///
/// # Errors
/// Returns an error if the manifest cannot be read, parsed, or validated.
pub fn load_corpus_manifest(path: &Path) -> Result<BenchmarkCorpusManifest> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("read corpus manifest {}", path.display()))?;
    let manifest: BenchmarkCorpusManifest = toml::from_str(&raw)
        .with_context(|| format!("parse corpus manifest {}", path.display()))?;
    validate_corpus_manifest(&manifest)
        .with_context(|| format!("validate corpus manifest {}", path.display()))?;
    Ok(manifest)
}

/// Load and validate all checked-in benchmark corpus manifests.
///
/// # Errors
/// Returns an error if manifests cannot be loaded or validated.
pub fn load_corpus_catalog() -> Result<Vec<BenchmarkCorpusManifest>> {
    let mut manifests = Vec::new();
    for entry in fs::read_dir(bench_corpora_dir()).context("read benchmark corpus directory")? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("toml") {
            continue;
        }
        manifests.push(load_corpus_manifest(&path)?);
    }
    manifests.sort_by(|a, b| a.corpus_id.cmp(&b.corpus_id));
    Ok(manifests)
}

#[cfg(test)]
mod tests {
    use bijux_dna_bench_model::{CorpusDomain, CorpusScale};

    use super::load_corpus_catalog;

    #[test]
    fn corpus_catalog_contains_fastq_ci_small_fixture_matrix() -> anyhow::Result<()> {
        let catalog = load_corpus_catalog()?;
        let has_fastq_ci = catalog.iter().any(|manifest| {
            manifest.domain == CorpusDomain::Fastq
                && manifest.scale == CorpusScale::CiSmall
                && manifest.datasets.len() >= 8
        });
        assert!(
            has_fastq_ci,
            "checked-in corpus catalog must include fastq ci-small coverage with at least 8 datasets"
        );
        Ok(())
    }
}
