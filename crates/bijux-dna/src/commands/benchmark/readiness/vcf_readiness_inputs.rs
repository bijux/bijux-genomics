use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use crate::commands::benchmark::local_corpus_fixture::vcf::{
    load_vcf_corpus_fixture_manifest_path, DEFAULT_VCF_MINI_MANIFEST_PATH,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GovernedVcfFixtureInputs {
    pub(crate) reference_fasta_path: String,
    pub(crate) reference_fasta_index_path: String,
    pub(crate) raw_vcf_path: String,
    pub(crate) filtered_vcf_path: String,
    pub(crate) multisample_vcf_path: String,
    pub(crate) phased_vcf_path: String,
    pub(crate) panel_vcf_path: String,
    pub(crate) target_sites_bed_path: String,
    pub(crate) sample_metadata_path: String,
    pub(crate) population_metadata_path: String,
}

pub(crate) fn load_governed_vcf_fixture_inputs(
    repo_root: &Path,
) -> Result<GovernedVcfFixtureInputs> {
    let manifest_path = repo_root.join(DEFAULT_VCF_MINI_MANIFEST_PATH);
    let manifest = load_vcf_corpus_fixture_manifest_path(&manifest_path)
        .with_context(|| format!("load governed VCF fixture from {}", manifest_path.display()))?;
    let manifest_dir = manifest_path.parent().ok_or_else(|| {
        anyhow!("VCF fixture manifest has no parent: {}", manifest_path.display())
    })?;

    Ok(GovernedVcfFixtureInputs {
        reference_fasta_path: path_relative_to_repo(
            repo_root,
            &resolve_manifest_relative_path(manifest_dir, &manifest.reference_fasta_path),
        ),
        reference_fasta_index_path: path_relative_to_repo(
            repo_root,
            &resolve_manifest_relative_path(manifest_dir, &manifest.reference_fasta_index_path),
        ),
        raw_vcf_path: path_relative_to_repo(
            repo_root,
            &resolve_manifest_relative_path(manifest_dir, &manifest.raw_vcf_path),
        ),
        filtered_vcf_path: path_relative_to_repo(
            repo_root,
            &resolve_manifest_relative_path(manifest_dir, &manifest.filtered_vcf_path),
        ),
        multisample_vcf_path: path_relative_to_repo(
            repo_root,
            &resolve_manifest_relative_path(manifest_dir, &manifest.multisample_vcf_path),
        ),
        phased_vcf_path: path_relative_to_repo(
            repo_root,
            &resolve_manifest_relative_path(manifest_dir, &manifest.phased_vcf_path),
        ),
        panel_vcf_path: path_relative_to_repo(
            repo_root,
            &resolve_manifest_relative_path(manifest_dir, &manifest.panel_vcf_path),
        ),
        target_sites_bed_path: path_relative_to_repo(
            repo_root,
            &resolve_manifest_relative_path(manifest_dir, &manifest.target_sites_bed_path),
        ),
        sample_metadata_path: path_relative_to_repo(
            repo_root,
            &resolve_manifest_relative_path(manifest_dir, &manifest.sample_metadata_path),
        ),
        population_metadata_path: path_relative_to_repo(
            repo_root,
            &resolve_manifest_relative_path(manifest_dir, &manifest.population_metadata_path),
        ),
    })
}

pub(crate) fn materialize_indexed_vcf_input(
    repo_root: &Path,
    source_relative_path: &str,
    staging_root: &Path,
    output_name: &str,
) -> Result<(String, String)> {
    let source_vcf = repo_root.join(source_relative_path);
    let output_root = repo_root.join(staging_root);
    fs::create_dir_all(&output_root)
        .with_context(|| format!("create {}", output_root.display()))?;
    let output_vcfgz = output_root.join(output_name);
    let output_tbi =
        bijux_dna_stages_vcf::vcf_io::vcf_index_bgzip_tabix(&source_vcf, &output_vcfgz)
            .with_context(|| {
                format!(
                    "materialize indexed governed VCF {} into {}",
                    source_vcf.display(),
                    output_vcfgz.display()
                )
            })?;
    Ok((
        path_relative_to_repo(repo_root, &output_vcfgz),
        path_relative_to_repo(repo_root, &output_tbi),
    ))
}

pub(crate) fn materialize_reference_fasta_with_index(
    repo_root: &Path,
    source_relative_path: &str,
    staging_root: &Path,
) -> Result<(String, String)> {
    let (fasta, index) = bijux_dna_api::v1::api::bench::materialize_benchmark_reference_fasta(
        &repo_root.join(source_relative_path),
        &repo_root.join(staging_root),
    )?;
    Ok((path_relative_to_repo(repo_root, &fasta), path_relative_to_repo(repo_root, &index)))
}

fn resolve_manifest_relative_path(manifest_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        manifest_dir.join(path)
    }
}

pub(crate) fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
