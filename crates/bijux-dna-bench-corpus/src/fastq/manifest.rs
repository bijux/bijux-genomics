//! Owner: bijux-dna-bench-corpus
//! FASTQ corpus manifest ownership.
use super::{
    anyhow, fs, BTreeSet, FastqCorpusFixtureLayout, FastqCorpusFixtureManifest, Path, Result,
    FASTQ_CORPUS_FIXTURE_SCHEMA_VERSION,
};
use anyhow::Context;

/// Load the typed corpus document from its explicit local path.
///
/// # Errors
/// Rejects unreadable documents, malformed syntax and incompatible typed fields.
pub fn load_fastq_corpus_fixture_manifest_path(
    manifest_path: &Path,
) -> Result<FastqCorpusFixtureManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

pub(super) fn validate_fastq_corpus_fixture_manifest_contract(
    manifest: &FastqCorpusFixtureManifest,
) -> Result<()> {
    if manifest.schema_version != FASTQ_CORPUS_FIXTURE_SCHEMA_VERSION {
        return Err(anyhow!(
            "unsupported FASTQ corpus fixture schema `{}`",
            manifest.schema_version
        ));
    }
    if manifest.corpus_id.trim().is_empty() {
        return Err(anyhow!("FASTQ corpus fixture must declare a non-empty `corpus_id`"));
    }
    if manifest.species.trim().is_empty() {
        return Err(anyhow!("FASTQ corpus fixture must declare a non-empty `species`"));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!("FASTQ corpus fixture must declare a non-empty `description`"));
    }
    if manifest.reference_id.as_deref().is_some_and(|reference_id| reference_id.trim().is_empty()) {
        return Err(anyhow!(
            "FASTQ corpus fixture `reference_id`, when declared, must not be empty"
        ));
    }
    if manifest.samples.is_empty() {
        return Err(anyhow!("FASTQ corpus fixture must declare at least one sample"));
    }

    let mut sample_ids = BTreeSet::new();
    for sample in &manifest.samples {
        if sample.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "FASTQ corpus fixture samples must declare a non-empty `sample_id`"
            ));
        }
        if !sample_ids.insert(sample.sample_id.clone()) {
            return Err(anyhow!("FASTQ corpus fixture repeats sample_id `{}`", sample.sample_id));
        }
        if sample.cohort.trim().is_empty() {
            return Err(anyhow!(
                "FASTQ corpus fixture sample `{}` must declare a non-empty `cohort`",
                sample.sample_id
            ));
        }
        if sample.source_paths.is_empty() {
            return Err(anyhow!(
                "FASTQ corpus fixture sample `{}` must declare at least one `source_paths` entry",
                sample.sample_id
            ));
        }
        match sample.layout {
            FastqCorpusFixtureLayout::Se => {
                if sample.r2_path.is_some() {
                    return Err(anyhow!(
                        "FASTQ corpus fixture sample `{}` is single-end but declares `r2_path`",
                        sample.sample_id
                    ));
                }
                if sample.expected_read_count_r2.is_some() {
                    return Err(anyhow!(
                        "FASTQ corpus fixture sample `{}` is single-end but declares `expected_read_count_r2`",
                        sample.sample_id
                    ));
                }
                if sample.expected_read_count_total != sample.expected_read_count_r1 {
                    return Err(anyhow!(
                        "FASTQ corpus fixture sample `{}` single-end total must match `expected_read_count_r1`",
                        sample.sample_id
                    ));
                }
            }
            FastqCorpusFixtureLayout::Pe => {
                let expected_r2 = sample.expected_read_count_r2.ok_or_else(|| {
                    anyhow!(
                        "FASTQ corpus fixture sample `{}` paired-end layout requires `expected_read_count_r2`",
                        sample.sample_id
                    )
                })?;
                if sample.r2_path.is_none() {
                    return Err(anyhow!(
                        "FASTQ corpus fixture sample `{}` paired-end layout requires `r2_path`",
                        sample.sample_id
                    ));
                }
                if sample.expected_read_count_total
                    != sample.expected_read_count_r1.saturating_add(expected_r2)
                {
                    return Err(anyhow!(
                        "FASTQ corpus fixture sample `{}` paired-end total must equal `expected_read_count_r1 + expected_read_count_r2`",
                        sample.sample_id
                    ));
                }
            }
        }
    }
    Ok(())
}
