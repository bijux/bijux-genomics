//! Owner: bijux-dna-bench
//! Repository-backed loading for benchmark observations.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::repo::{bench_bundles_dir, bench_corpora_dir};
use crate::repo::RunRepository;
use bijux_dna_bench_model::contract::{validate_bundle_manifest, validate_corpus_manifest};
use bijux_dna_bench_model::{BenchmarkBundleManifest, BenchmarkCorpusManifest, BenchmarkObservation};

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

/// Load and validate a single benchmark bundle manifest.
///
/// # Errors
/// Returns an error if the manifest cannot be read, parsed, or validated.
pub fn load_bundle_manifest(path: &Path) -> Result<BenchmarkBundleManifest> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("read bundle manifest {}", path.display()))?;
    let manifest: BenchmarkBundleManifest = toml::from_str(&raw)
        .with_context(|| format!("parse bundle manifest {}", path.display()))?;
    validate_bundle_manifest(&manifest)
        .with_context(|| format!("validate bundle manifest {}", path.display()))?;
    Ok(manifest)
}

/// Load and validate all checked-in benchmark bundle manifests.
///
/// # Errors
/// Returns an error if manifests cannot be loaded or validated.
pub fn load_bundle_catalog() -> Result<Vec<BenchmarkBundleManifest>> {
    let mut manifests = Vec::new();
    for entry in fs::read_dir(bench_bundles_dir()).context("read benchmark bundle directory")? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("toml") {
            continue;
        }
        manifests.push(load_bundle_manifest(&path)?);
    }
    manifests.sort_by(|a, b| a.bundle_id.cmp(&b.bundle_id));
    Ok(manifests)
}

#[cfg(test)]
mod tests {
    use bijux_dna_bench_model::{CorpusDomain, CorpusScale};

    use super::{load_bundle_catalog, load_corpus_catalog};

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

    #[test]
    fn bundle_catalog_contains_scientific_caveat_bundle() -> anyhow::Result<()> {
        let catalog = load_bundle_catalog()?;
        assert!(
            catalog.iter().any(|bundle| !bundle.scientific_caveats.is_empty()),
            "checked-in bundle catalog must include at least one caveated bundle"
        );
        Ok(())
    }
}
