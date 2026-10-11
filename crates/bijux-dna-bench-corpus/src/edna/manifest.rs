//! Owner: bijux-dna-bench-corpus
//! EDNA corpus manifest ownership.
use super::{
    anyhow, fs, BTreeSet, EdnaCorpusFixtureManifest, Path, Result,
    EDNA_CORPUS_FIXTURE_SCHEMA_VERSION,
};
use anyhow::Context;

/// Load the typed corpus document from its explicit local path.
///
/// # Errors
/// Rejects unreadable documents, malformed syntax and incompatible typed fields.
pub fn load_edna_corpus_fixture_manifest_path(
    manifest_path: &Path,
) -> Result<EdnaCorpusFixtureManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

/// Validate the typed corpus declaration before observing its payloads.
///
/// # Errors
/// Rejects unsupported schemas, inconsistent identities and incomplete corpus declarations.
pub fn validate_edna_corpus_fixture_manifest_contract(
    manifest: &EdnaCorpusFixtureManifest,
) -> Result<()> {
    if manifest.schema_version != EDNA_CORPUS_FIXTURE_SCHEMA_VERSION {
        return Err(anyhow!(
            "unsupported eDNA corpus fixture schema `{}`",
            manifest.schema_version
        ));
    }
    if manifest.corpus_id.trim().is_empty() {
        return Err(anyhow!("eDNA corpus fixture must declare a non-empty `corpus_id`"));
    }
    if manifest.community_id.trim().is_empty() {
        return Err(anyhow!("eDNA corpus fixture must declare a non-empty `community_id`"));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!("eDNA corpus fixture must declare a non-empty `description`"));
    }
    if manifest.expected_taxa_path.as_os_str().is_empty() {
        return Err(anyhow!("eDNA corpus fixture must declare a non-empty `expected_taxa_path`"));
    }
    if manifest.expected_taxa.is_empty() {
        return Err(anyhow!("eDNA corpus fixture must declare at least one `expected_taxa` entry"));
    }
    if manifest.samples.is_empty() {
        return Err(anyhow!("eDNA corpus fixture must declare at least one sample"));
    }

    let mut taxon_ids = BTreeSet::new();
    for taxon in &manifest.expected_taxa {
        if taxon.taxon_id == 0 {
            return Err(anyhow!(
                "eDNA corpus fixture expected_taxa entries must declare a non-zero `taxon_id`"
            ));
        }
        if !taxon_ids.insert(taxon.taxon_id) {
            return Err(anyhow!(
                "eDNA corpus fixture repeats expected taxon_id `{}`",
                taxon.taxon_id
            ));
        }
        if taxon.name.trim().is_empty() {
            return Err(anyhow!(
                "eDNA corpus fixture expected_taxa entries must declare a non-empty `name`"
            ));
        }
        if taxon.rank.trim().is_empty() {
            return Err(anyhow!(
                "eDNA corpus fixture expected_taxa entries must declare a non-empty `rank`"
            ));
        }
    }

    let mut sample_ids = BTreeSet::new();
    for sample in &manifest.samples {
        if sample.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "eDNA corpus fixture samples must declare a non-empty `sample_id`"
            ));
        }
        if !sample_ids.insert(sample.sample_id.clone()) {
            return Err(anyhow!("eDNA corpus fixture repeats sample_id `{}`", sample.sample_id));
        }
        if sample.community_label.trim().is_empty() {
            return Err(anyhow!(
                "eDNA corpus fixture sample `{}` must declare a non-empty `community_label`",
                sample.sample_id
            ));
        }
        if sample.expected_read_count == 0 {
            return Err(anyhow!(
                "eDNA corpus fixture sample `{}` must declare a positive `expected_read_count`",
                sample.sample_id
            ));
        }
        if sample.source_paths.is_empty() {
            return Err(anyhow!(
                "eDNA corpus fixture sample `{}` must declare at least one `source_paths` entry",
                sample.sample_id
            ));
        }
    }

    Ok(())
}
