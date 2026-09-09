//! Owner: bijux-dna-bench
//! Normalized FASTQ corpus discovery and publication cohort metadata.
//!
//! Manifest content identity, sample/accession closure and publication cohorts are admitted here.
//! FASTQ record semantics and governed data-index admission remain separate contracts.

mod manifest;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::workflow::load_fastq_corpus_source_spec;
use anyhow::{anyhow, Context, Result};

use bijux_dna_bench_model::{
    validate_fastq_corpus_source_spec, CorpusAcquisitionLayout,
    FastqCorpusSourceSample as CorpusSpecSample, FastqCorpusSourceSpec as CorpusSpec,
};

/// Discovered normalized read paths; content admission remains a separate operation.
#[derive(Debug, Clone)]
pub struct NormalizedFastqSample {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub layout: String,
}

/// Load a source specification for the requested corpus identity.
///
/// # Errors
/// Returns an error when the required corpus, sample, or cohort contract is not satisfied.
pub fn load_fastq_corpus_source_for_id(path: &Path, corpus_id: &str) -> Result<CorpusSpec> {
    let spec = load_fastq_corpus_source_spec(path)?;
    if spec.corpus_id != corpus_id {
        return Err(anyhow!(
            "FASTQ corpus source `{}` does not match requested corpus `{corpus_id}`",
            spec.corpus_id
        ));
    }
    Ok(spec)
}

/// Return the total declared by the source specification.
#[must_use]
pub fn normalized_fastq_expected_sample_total(spec: &CorpusSpec) -> usize {
    spec.target_total
}

/// Resolve the declared publication cohort counts for a full or paired scope.
///
/// # Errors
/// Returns an error when the required corpus, sample, or cohort contract is not satisfied.
pub fn expected_fastq_publication_cohort_counts(
    spec: &CorpusSpec,
    sample_scope: &str,
) -> Result<BTreeMap<String, usize>> {
    let cohort_targets = publication_cohort_targets(spec)?;
    match sample_scope {
        "full" => Ok(BTreeMap::from([
            ("ancient_pe".to_string(), cohort_targets[1]),
            ("ancient_se".to_string(), cohort_targets[0]),
            ("modern_pe".to_string(), cohort_targets[3]),
            ("modern_se".to_string(), cohort_targets[2]),
        ])),
        "paired" => Ok(BTreeMap::from([
            ("ancient_pe".to_string(), cohort_targets[1]),
            ("modern_pe".to_string(), cohort_targets[3]),
        ])),
        other => Err(anyhow!("unsupported corpus benchmark sample scope `{other}`")),
    }
}

fn publication_cohort_targets(spec: &CorpusSpec) -> Result<[usize; 4]> {
    let targets = [
        spec.target_ancient_se,
        spec.target_ancient_pe,
        spec.target_modern_se,
        spec.target_modern_pe,
    ];
    if targets.iter().any(Option::is_none) {
        Err(anyhow!(
            "FASTQ corpus source `{}` must declare every publication cohort target",
            spec.corpus_id
        ))
    } else {
        Ok(targets.map(Option::unwrap_or_default))
    }
}

/// Discover deterministically ordered R1/R2 paths and enforce the expected sample count.
///
/// # Errors
/// Returns an error when the required corpus, sample, or cohort contract is not satisfied.
pub fn discover_normalized_fastq_samples(
    corpus_root: &Path,
    corpus_id: &str,
    expected_total: usize,
) -> Result<Vec<NormalizedFastqSample>> {
    let normalized = corpus_root.join("normalized");
    if !normalized.is_dir() {
        return Err(anyhow!("missing normalized corpus directory: {}", normalized.display()));
    }

    let mut sample_ids = BTreeSet::new();
    for entry in
        fs::read_dir(&normalized).with_context(|| format!("read {}", normalized.display()))?
    {
        let path = entry?.path();
        let Some(name) = path.file_name().and_then(|row| row.to_str()) else {
            continue;
        };
        if let Some(sample_id) = name.strip_suffix("_R1.fastq.gz") {
            sample_ids.insert(sample_id.to_string());
        }
        if let Some(sample_id) = name.strip_suffix("_R2.fastq.gz") {
            sample_ids.insert(sample_id.to_string());
        }
    }

    let mut samples = Vec::new();
    for sample_id in sample_ids {
        let r1 = normalized.join(format!("{sample_id}_R1.fastq.gz"));
        let r2 = normalized.join(format!("{sample_id}_R2.fastq.gz"));
        if !r1.is_file() {
            return Err(anyhow!("missing R1 for sample {sample_id}: {}", r1.display()));
        }
        let r2_value = r2.is_file().then_some(r2);
        samples.push(NormalizedFastqSample {
            sample_id,
            r1,
            r2: r2_value.clone(),
            layout: if r2_value.is_some() { "pe".to_string() } else { "se".to_string() },
        });
    }

    if samples.len() != expected_total {
        return Err(anyhow!(
            "expected {expected_total} normalized samples for {corpus_id}, found {}",
            samples.len()
        ));
    }
    Ok(samples)
}

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

fn validate_sample_metadata(
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

/// Select paired samples using curated metadata and enforce paired cohort counts.
///
/// # Errors
/// Returns an error when the required corpus, sample, or cohort contract is not satisfied.
pub fn select_normalized_fastq_paired_samples(
    spec: &CorpusSpec,
    samples: &[NormalizedFastqSample],
    metadata_by_sample: &BTreeMap<String, CorpusSpecSample>,
) -> Result<Vec<NormalizedFastqSample>> {
    validate_sample_metadata(spec, samples, metadata_by_sample)?;
    let paired = samples
        .iter()
        .filter(|row| {
            metadata_by_sample
                .get(&row.sample_id)
                .is_some_and(|meta| meta.layout == CorpusAcquisitionLayout::Pe)
        })
        .cloned()
        .collect::<Vec<_>>();
    let expected_counts = expected_fastq_publication_cohort_counts(spec, "paired")?;
    let mut actual_counts =
        expected_counts.keys().map(|key| (key.clone(), 0)).collect::<BTreeMap<_, _>>();
    for sample in &paired {
        let metadata = metadata_by_sample
            .get(&sample.sample_id)
            .ok_or_else(|| anyhow!("missing paired metadata for {}", sample.sample_id))?;
        *actual_counts
            .entry(format!("{}_{}", metadata.era, metadata.layout.as_str()))
            .or_default() += 1;
    }
    if actual_counts != expected_counts {
        return Err(anyhow!(
            "paired corpus contract drift: expected {expected_counts:?}, found {actual_counts:?}"
        ));
    }
    Ok(paired)
}

#[cfg(test)]
mod tests;
