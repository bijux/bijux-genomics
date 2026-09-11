use super::metadata::validate_sample_metadata;
use super::NormalizedFastqSample;
use crate::source::{
    CorpusAcquisitionLayout, FastqCorpusSourceSample as CorpusSpecSample,
    FastqCorpusSourceSpec as CorpusSpec,
};
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;

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
