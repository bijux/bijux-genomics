use super::cohorts::expected_fastq_publication_cohort_counts;
use super::manifest;
use super::NormalizedFastqSample;
use crate::source::{
    validate_fastq_corpus_source_spec, CorpusAcquisitionLayout,
    FastqCorpusSourceSample as CorpusSpecSample, FastqCorpusSourceSpec as CorpusSpec,
};
use anyhow::{anyhow, Context, Result};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

/// Join normalized manifest digests to curated accessions and validate cohort counts.
///
/// # Errors
/// Returns an error when the required corpus, sample, or cohort contract is not satisfied.
pub fn match_normalized_fastq_corpus_metadata(
    corpus_root: &Path,
    spec: &CorpusSpec,
    samples: &[NormalizedFastqSample],
) -> Result<BTreeMap<String, CorpusSpecSample>> {
    validate_fastq_corpus_source_spec(spec)?;
    let manifest = manifest::load(corpus_root)?;

    let mut hash_to_accessions = BTreeMap::<String, BTreeSet<String>>::new();
    for (relative_path, digest) in &manifest.files {
        let path = Path::new(relative_path);
        let parts = path.iter().collect::<Vec<_>>();
        if parts.len() >= 3 && parts[0].to_str() == Some("raw") {
            let accession = parts[1].to_string_lossy().to_string();
            hash_to_accessions.entry(digest.clone()).or_default().insert(accession);
        }
    }
    let spec_by_accession = spec
        .samples
        .iter()
        .cloned()
        .map(|row| (row.accession.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let mut metadata_by_sample = BTreeMap::<String, CorpusSpecSample>::new();
    for (relative_path, digest) in &manifest.files {
        let path = Path::new(relative_path);
        let parts = path.iter().collect::<Vec<_>>();
        if parts.len() != 2 || parts[0].to_str() != Some("normalized") {
            continue;
        }
        let file_name = parts[1].to_string_lossy();
        let sample_id = if let Some(sample_id) = file_name.strip_suffix("_R1.fastq.gz") {
            sample_id.to_string()
        } else if let Some(sample_id) = file_name.strip_suffix("_R2.fastq.gz") {
            sample_id.to_string()
        } else {
            continue;
        };
        let accessions = hash_to_accessions
            .get(digest)
            .ok_or_else(|| anyhow!("missing accession for {relative_path}"))?;
        if accessions.len() != 1 {
            return Err(anyhow!(
                "expected one accession for {}, found {}",
                relative_path,
                accessions.iter().map(String::as_str).collect::<Vec<_>>().join(",")
            ));
        }
        let accession = accessions.first().context("missing accession identity")?;
        let metadata = spec_by_accession
            .get(accession)
            .cloned()
            .ok_or_else(|| anyhow!("missing curated metadata for accession {accession}"))?;
        if let Some(previous) = metadata_by_sample.insert(sample_id.clone(), metadata.clone()) {
            if previous.accession != metadata.accession {
                return Err(anyhow!("conflicting mate accessions for {sample_id}"));
            }
        }
    }

    validate_sample_metadata(spec, samples, &metadata_by_sample)?;
    let mut expected_paths = BTreeSet::new();
    for sample in samples {
        for (mate, path) in [(1, Some(&sample.r1)), (2, sample.r2.as_ref())] {
            let Some(path) = path else { continue };
            let relative = format!("normalized/{}_R{mate}.fastq.gz", sample.sample_id);
            if path != &corpus_root.join(&relative) || !manifest.files.contains_key(&relative) {
                return Err(anyhow!("read path or manifest entry mismatch for {relative}"));
            }
            expected_paths.insert(relative);
        }
    }
    let declared_paths = manifest
        .files
        .keys()
        .filter(|path| path.starts_with("normalized/"))
        .cloned()
        .collect::<BTreeSet<_>>();
    if declared_paths != expected_paths {
        return Err(anyhow!("normalized manifest read paths do not close over discovered samples"));
    }
    manifest::verify_content(corpus_root, &manifest)?;
    Ok(metadata_by_sample)
}

pub(super) fn validate_sample_metadata(
    spec: &CorpusSpec,
    samples: &[NormalizedFastqSample],
    metadata_by_sample: &BTreeMap<String, CorpusSpecSample>,
) -> Result<()> {
    validate_fastq_corpus_source_spec(spec)?;
    if samples.len() != spec.target_total || metadata_by_sample.len() != samples.len() {
        return Err(anyhow!("normalized samples and metadata do not close over source total"));
    }
    let spec_by_accession =
        spec.samples.iter().map(|row| (row.accession.as_str(), row)).collect::<BTreeMap<_, _>>();
    let mut sample_ids = BTreeSet::new();
    let mut accessions = BTreeSet::new();
    let mut actual_counts = expected_fastq_publication_cohort_counts(spec, "full")?;
    for count in actual_counts.values_mut() {
        *count = 0;
    }
    for sample in samples {
        if sample.sample_id.is_empty() || !sample_ids.insert(&sample.sample_id) {
            return Err(anyhow!("empty or repeated normalized sample identity"));
        }
        let metadata = metadata_by_sample
            .get(&sample.sample_id)
            .ok_or_else(|| anyhow!("missing accession metadata for {}", sample.sample_id))?;
        if !accessions.insert(&metadata.accession)
            || spec_by_accession.get(metadata.accession.as_str()).copied() != Some(metadata)
        {
            return Err(anyhow!(
                "repeated or inconsistent curated accession for {}",
                sample.sample_id
            ));
        }
        if sample.layout != metadata.layout.as_str()
            || sample.r2.is_some() != (metadata.layout == CorpusAcquisitionLayout::Pe)
        {
            return Err(anyhow!(
                "read layout disagrees with curated metadata for {}",
                sample.sample_id
            ));
        }
        *actual_counts
            .entry(format!("{}_{}", metadata.era, metadata.layout.as_str()))
            .or_default() += 1;
    }

    let expected_counts = expected_fastq_publication_cohort_counts(spec, "full")?;
    if actual_counts != expected_counts {
        return Err(anyhow!(
            "{} cohort contract drift: expected {:?}, found {:?}",
            spec.corpus_id,
            expected_counts,
            actual_counts
        ));
    }
    Ok(())
}
