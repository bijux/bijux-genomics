//! Owner: bijux-dna-bench-corpus
//! BAM corpus manifest ownership.
use super::{
    anyhow, fs, BTreeSet, BamCorpusFixtureManifest, Path, Result, BAM_CORPUS_FIXTURE_SCHEMA_VERSION,
};
use anyhow::Context;

/// Load the typed corpus document from its explicit local path.
///
/// # Errors
/// Rejects unreadable documents, malformed syntax and incompatible typed fields.
pub fn load_bam_corpus_fixture_manifest_path(
    manifest_path: &Path,
) -> Result<BamCorpusFixtureManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

pub(super) fn validate_bam_corpus_fixture_manifest_contract(
    manifest: &BamCorpusFixtureManifest,
) -> Result<()> {
    if manifest.schema_version != BAM_CORPUS_FIXTURE_SCHEMA_VERSION {
        return Err(anyhow!("unsupported BAM corpus fixture schema `{}`", manifest.schema_version));
    }
    if manifest.corpus_id.trim().is_empty() {
        return Err(anyhow!("BAM corpus fixture must declare a non-empty `corpus_id`"));
    }
    if manifest.species.trim().is_empty() {
        return Err(anyhow!("BAM corpus fixture must declare a non-empty `species`"));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!("BAM corpus fixture must declare a non-empty `description`"));
    }
    if manifest.reference_id.as_deref().is_some_and(|reference_id| reference_id.trim().is_empty()) {
        return Err(anyhow!("BAM corpus fixture `reference_id`, when declared, must not be empty"));
    }
    let has_adna_profile = manifest.udg_model.is_some()
        || manifest.damage_signal.is_some()
        || manifest.expected_terminal_pattern_class.is_some();
    if has_adna_profile {
        if manifest.udg_model.as_deref().is_none_or(|value| value.trim().is_empty()) {
            return Err(anyhow!(
                "BAM corpus fixture with an aDNA profile must declare a non-empty `udg_model`"
            ));
        }
        if manifest.damage_signal.as_deref().is_none_or(|value| value.trim().is_empty()) {
            return Err(anyhow!(
                "BAM corpus fixture with an aDNA profile must declare a non-empty `damage_signal`"
            ));
        }
        if manifest
            .expected_terminal_pattern_class
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
        {
            return Err(anyhow!(
                "BAM corpus fixture with an aDNA profile must declare a non-empty `expected_terminal_pattern_class`"
            ));
        }
    }
    if manifest.samples.is_empty() {
        return Err(anyhow!("BAM corpus fixture must declare at least one sample"));
    }

    let mut sample_ids = BTreeSet::new();
    for sample in &manifest.samples {
        if sample.sample_id.trim().is_empty() {
            return Err(anyhow!("BAM corpus fixture samples must declare a non-empty `sample_id`"));
        }
        if !sample_ids.insert(sample.sample_id.clone()) {
            return Err(anyhow!("BAM corpus fixture repeats sample_id `{}`", sample.sample_id));
        }
        if sample.cohort.trim().is_empty() {
            return Err(anyhow!(
                "BAM corpus fixture sample `{}` must declare a non-empty `cohort`",
                sample.sample_id
            ));
        }
        if sample.expected_contigs.is_empty() {
            return Err(anyhow!(
                "BAM corpus fixture sample `{}` must declare at least one `expected_contigs` entry",
                sample.sample_id
            ));
        }
        if sample.expected_contigs.iter().any(|contig| contig.trim().is_empty()) {
            return Err(anyhow!(
                "BAM corpus fixture sample `{}` has an empty `expected_contigs` entry",
                sample.sample_id
            ));
        }
        if sample.expected_header_sample_ids.is_empty() {
            return Err(anyhow!(
                "BAM corpus fixture sample `{}` must declare at least one `expected_header_sample_ids` entry",
                sample.sample_id
            ));
        }
        if sample.expected_read_group_ids.is_empty() {
            return Err(anyhow!(
                "BAM corpus fixture sample `{}` must declare at least one `expected_read_group_ids` entry",
                sample.sample_id
            ));
        }
        if sample.expected_record_count == 0 {
            return Err(anyhow!(
                "BAM corpus fixture sample `{}` must declare a positive `expected_record_count`",
                sample.sample_id
            ));
        }
        if sample.source_paths.is_empty() {
            return Err(anyhow!(
                "BAM corpus fixture sample `{}` must declare at least one `source_paths` entry",
                sample.sample_id
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_expected_output_contract(
    outputs: &[String],
    label: &str,
) -> Result<Vec<String>> {
    if outputs.is_empty() {
        return Err(anyhow!("{label} must declare at least one expected output"));
    }
    let mut seen = BTreeSet::new();
    let mut validated = Vec::with_capacity(outputs.len());
    for output in outputs {
        if output.trim().is_empty() {
            return Err(anyhow!("{label} must not contain an empty expected output entry"));
        }
        if !seen.insert(output.clone()) {
            return Err(anyhow!("{label} repeats expected output `{output}`"));
        }
        validated.push(output.clone());
    }
    Ok(validated)
}
