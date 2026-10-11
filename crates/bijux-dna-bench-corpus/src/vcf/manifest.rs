//! Owner: bijux-dna-bench-corpus
//! VCF corpus manifest ownership.
use super::{
    anyhow, fs, BTreeSet, Path, Result, VcfCorpusFixtureManifest, VCF_CORPUS_FIXTURE_SCHEMA_VERSION,
};
use anyhow::Context;

/// Load the typed corpus document from its explicit local path.
///
/// # Errors
/// Rejects unreadable documents, malformed syntax and incompatible typed fields.
pub fn load_vcf_corpus_fixture_manifest_path(
    manifest_path: &Path,
) -> Result<VcfCorpusFixtureManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

/// Validate the typed corpus declaration before observing its payloads.
///
/// # Errors
/// Rejects unsupported schemas, inconsistent identities and incomplete corpus declarations.
pub fn validate_vcf_corpus_fixture_manifest_contract(
    manifest: &VcfCorpusFixtureManifest,
) -> Result<()> {
    if manifest.schema_version != VCF_CORPUS_FIXTURE_SCHEMA_VERSION {
        return Err(anyhow!("unsupported VCF corpus fixture schema `{}`", manifest.schema_version));
    }
    if manifest.corpus_id.trim().is_empty() {
        return Err(anyhow!("VCF corpus fixture must declare a non-empty `corpus_id`"));
    }
    if manifest.reference_id.trim().is_empty() {
        return Err(anyhow!("VCF corpus fixture must declare a non-empty `reference_id`"));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!("VCF corpus fixture must declare a non-empty `description`"));
    }
    for (field_name, path) in [
        ("reference_fasta_path", &manifest.reference_fasta_path),
        ("reference_fasta_index_path", &manifest.reference_fasta_index_path),
        ("reference_dict_path", &manifest.reference_dict_path),
        ("raw_vcf_path", &manifest.raw_vcf_path),
        ("filtered_vcf_path", &manifest.filtered_vcf_path),
        ("multisample_vcf_path", &manifest.multisample_vcf_path),
        ("phased_vcf_path", &manifest.phased_vcf_path),
        ("panel_vcf_path", &manifest.panel_vcf_path),
        ("target_sites_bed_path", &manifest.target_sites_bed_path),
        ("sample_metadata_path", &manifest.sample_metadata_path),
        ("population_metadata_path", &manifest.population_metadata_path),
    ] {
        if path.as_os_str().is_empty() {
            return Err(anyhow!("VCF corpus fixture must declare a non-empty `{field_name}`"));
        }
    }

    validate_expected_sample_ids("expected_raw_sample_ids", &manifest.expected_raw_sample_ids)?;
    validate_expected_sample_ids(
        "expected_filtered_sample_ids",
        &manifest.expected_filtered_sample_ids,
    )?;
    validate_expected_sample_ids(
        "expected_multisample_sample_ids",
        &manifest.expected_multisample_sample_ids,
    )?;
    validate_expected_sample_ids(
        "expected_phased_sample_ids",
        &manifest.expected_phased_sample_ids,
    )?;
    validate_expected_sample_ids("expected_panel_sample_ids", &manifest.expected_panel_sample_ids)?;

    if manifest.expected_raw_sample_ids != manifest.expected_filtered_sample_ids {
        return Err(anyhow!("VCF corpus fixture raw and filtered sample ids must match exactly"));
    }
    if manifest.expected_multisample_sample_ids != manifest.expected_phased_sample_ids {
        return Err(anyhow!(
            "VCF corpus fixture multisample and phased sample ids must match exactly"
        ));
    }

    Ok(())
}

pub(super) fn validate_expected_sample_ids(field_name: &str, sample_ids: &[String]) -> Result<()> {
    if sample_ids.is_empty() {
        return Err(anyhow!("VCF corpus fixture `{field_name}` must not be empty"));
    }
    let mut seen = BTreeSet::new();
    for sample_id in sample_ids {
        if sample_id.trim().is_empty() {
            return Err(anyhow!(
                "VCF corpus fixture `{field_name}` must not contain empty sample ids"
            ));
        }
        if !seen.insert(sample_id.clone()) {
            return Err(anyhow!(
                "VCF corpus fixture `{field_name}` repeats sample id `{sample_id}`"
            ));
        }
    }
    Ok(())
}
