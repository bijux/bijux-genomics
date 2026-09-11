use super::NormalizedFastqSample;
use anyhow::{anyhow, Context, Result};
use std::{collections::BTreeSet, fs, path::Path};

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
