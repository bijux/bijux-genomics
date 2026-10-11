//! Owner: bijux-dna-bench-population
//! PCA admission input boundary.
use crate::pca::admission::cohort::build_metadata_by_sample;
use crate::pca::admission::cohort::load_population_labels;
use crate::pca::execution::artifacts::resolve_manifest_relative_path;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use bijux_dna_bench_corpus::vcf::load_sample_metadata;
use bijux_dna_bench_corpus::vcf::load_vcf_corpus_fixture_manifest_path;
use bijux_dna_bench_corpus::vcf::DEFAULT_VCF_MINI_MANIFEST_PATH;
use std::path::Path;

pub(crate) struct CohortAdmission {
    pub(crate) fixture_manifest_path: std::path::PathBuf,
    pub(crate) input_vcf_source: std::path::PathBuf,
    pub(crate) sample_metadata_source: std::path::PathBuf,
    pub(crate) population_metadata_source: std::path::PathBuf,
    pub(crate) expected_samples: Vec<String>,
    pub(crate) metadata_by_sample:
        std::collections::BTreeMap<String, bijux_dna_bench_corpus::vcf::SampleMetadataRow>,
    pub(crate) population_labels: std::collections::BTreeMap<String, String>,
}

pub(crate) fn admit_cohort(repo_root: &Path) -> Result<CohortAdmission> {
    let fixture_manifest_path = repo_root.join(DEFAULT_VCF_MINI_MANIFEST_PATH);
    let fixture_manifest = load_vcf_corpus_fixture_manifest_path(&fixture_manifest_path)?;
    let fixture_dir = fixture_manifest_path.parent().ok_or_else(|| {
        anyhow!("fixture manifest has no parent directory: {}", fixture_manifest_path.display())
    })?;
    let input_vcf_source =
        resolve_manifest_relative_path(fixture_dir, &fixture_manifest.multisample_vcf_path);
    let sample_metadata_source =
        resolve_manifest_relative_path(fixture_dir, &fixture_manifest.sample_metadata_path);
    let population_metadata_source =
        resolve_manifest_relative_path(fixture_dir, &fixture_manifest.population_metadata_path);
    let expected_samples = fixture_manifest.expected_multisample_sample_ids.clone();
    if expected_samples.is_empty() {
        bail!("governed VCF PCA smoke fixture must declare multisample cohort sample ids");
    }

    let sample_metadata = load_sample_metadata(&sample_metadata_source)?;
    let metadata_by_sample = build_metadata_by_sample(&sample_metadata, &expected_samples)?;
    let population_labels = load_population_labels(&population_metadata_source)?;

    Ok(CohortAdmission {
        fixture_manifest_path,
        input_vcf_source,
        sample_metadata_source,
        population_metadata_source,
        expected_samples,
        metadata_by_sample,
        population_labels,
    })
}
