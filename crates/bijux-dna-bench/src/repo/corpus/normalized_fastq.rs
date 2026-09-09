//! Normalized FASTQ corpus discovery and publication cohort metadata.
//!
//! This boundary preserves the MANIFEST.json digest-to-accession join used by corpus benchmarks.
//! It does not replace governed data-index admission or verify current file digests and read content.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::workflow::load_fastq_corpus_source_spec;
use anyhow::{anyhow, Context, Result};

use bijux_dna_bench_model::{
    CorpusAcquisitionLayout, FastqCorpusSourceSample as CorpusSpecSample,
    FastqCorpusSourceSpec as CorpusSpec,
};

#[derive(Debug, Clone, serde::Deserialize, Default)]
struct CorpusManifest {
    #[serde(default)]
    files: BTreeMap<String, String>,
}

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
    let manifest_path = corpus_root.join("MANIFEST.json");
    let raw = fs::read_to_string(&manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    let manifest: CorpusManifest =
        serde_json::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))?;

    let mut hash_to_accessions = BTreeMap::<String, Vec<String>>::new();
    for (relative_path, digest) in &manifest.files {
        let path = Path::new(relative_path);
        let parts = path.iter().collect::<Vec<_>>();
        if parts.len() >= 2 && parts[0].to_str() == Some("raw") {
            let accession = parts[1].to_string_lossy().to_string();
            hash_to_accessions.entry(digest.clone()).or_default().push(accession);
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
                accessions.join(",")
            ));
        }
        let accession = &accessions[0];
        let metadata = spec_by_accession
            .get(accession)
            .cloned()
            .ok_or_else(|| anyhow!("missing curated metadata for accession {accession}"))?;
        metadata_by_sample.insert(sample_id, metadata);
    }

    let mut actual_counts = expected_fastq_publication_cohort_counts(spec, "full")?;
    for count in actual_counts.values_mut() {
        *count = 0;
    }
    for sample in samples {
        let metadata = metadata_by_sample
            .get(&sample.sample_id)
            .ok_or_else(|| anyhow!("missing accession metadata for {}", sample.sample_id))?;
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
    Ok(metadata_by_sample)
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
    let paired = samples
        .iter()
        .filter(|row| {
            metadata_by_sample
                .get(&row.sample_id)
                .is_some_and(|meta| meta.layout == CorpusAcquisitionLayout::Pe)
        })
        .cloned()
        .collect::<Vec<_>>();
    let mut actual_counts = BTreeMap::<String, usize>::new();
    for sample in &paired {
        let metadata = metadata_by_sample
            .get(&sample.sample_id)
            .ok_or_else(|| anyhow!("missing paired metadata for {}", sample.sample_id))?;
        *actual_counts
            .entry(format!("{}_{}", metadata.era, metadata.layout.as_str()))
            .or_default() += 1;
    }
    let expected_counts = expected_fastq_publication_cohort_counts(spec, "paired")?;
    if actual_counts != expected_counts {
        return Err(anyhow!(
            "paired corpus contract drift: expected {expected_counts:?}, found {actual_counts:?}"
        ));
    }
    Ok(paired)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn source_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../benchmarks/corpora/human-short-read-diversity.toml")
    }

    fn corpus() -> Result<(tempfile::TempDir, CorpusSpec, BTreeMap<String, String>)> {
        let root = tempfile::tempdir()?;
        fs::create_dir(root.path().join("normalized"))?;
        let mut spec = load_fastq_corpus_source_spec(&source_path())?;
        spec.target_total = 4;
        spec.target_ancient_se = Some(1);
        spec.target_ancient_pe = Some(1);
        spec.target_modern_se = Some(1);
        spec.target_modern_pe = Some(1);
        let template = spec.samples[0].clone();
        spec.samples.clear();
        let mut files = BTreeMap::new();
        for (sample, era, layout) in [
            ("d", "modern", CorpusAcquisitionLayout::Pe),
            ("b", "ancient", CorpusAcquisitionLayout::Pe),
            ("c", "modern", CorpusAcquisitionLayout::Se),
            ("a", "ancient", CorpusAcquisitionLayout::Se),
        ] {
            let accession = format!("accession_{sample}");
            let mut metadata = template.clone();
            metadata.accession = accession.clone();
            metadata.era = era.to_string();
            metadata.layout = layout;
            spec.samples.push(metadata);
            fs::create_dir_all(root.path().join("raw").join(&accession))?;
            let mates = if layout == CorpusAcquisitionLayout::Pe { 2 } else { 1 };
            for mate in 1..=mates {
                let name = format!("{sample}_R{mate}.fastq.gz");
                let mut gzip =
                    flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
                write!(gzip, "@{sample}/{mate}\nACGT\n+\nIIII\n")?;
                let bytes = gzip.finish()?;
                let digest = bijux_dna_infra::hash_bytes_sha256(&bytes);
                for relative in [format!("normalized/{name}"), format!("raw/{accession}/{name}")] {
                    fs::write(root.path().join(&relative), &bytes)?;
                    files.insert(relative, digest.clone());
                }
            }
        }
        write_manifest(root.path(), &files)?;
        Ok((root, spec, files))
    }

    fn write_manifest(root: &Path, files: &BTreeMap<String, String>) -> Result<()> {
        fs::write(
            root.join("MANIFEST.json"),
            serde_json::to_vec(&serde_json::json!({"files": files}))?,
        )?;
        Ok(())
    }

    #[test]
    fn discovers_ordered_reads_matches_accessions_and_selects_both_paired_cohorts() -> Result<()> {
        let (root, spec, _) = corpus()?;
        let samples = discover_normalized_fastq_samples(
            root.path(),
            &spec.corpus_id,
            normalized_fastq_expected_sample_total(&spec),
        )?;
        assert_eq!(
            samples.iter().map(|row| row.sample_id.as_str()).collect::<Vec<_>>(),
            ["a", "b", "c", "d"]
        );
        assert_eq!(
            samples.iter().map(|row| row.layout.as_str()).collect::<Vec<_>>(),
            ["se", "pe", "se", "pe"]
        );
        let metadata = match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples)?;
        for sample in &samples {
            assert_eq!(
                metadata[&sample.sample_id].accession,
                format!("accession_{}", sample.sample_id)
            );
        }
        let paired = select_normalized_fastq_paired_samples(&spec, &samples, &metadata)?;
        assert_eq!(paired.iter().map(|row| row.sample_id.as_str()).collect::<Vec<_>>(), ["b", "d"]);
        Ok(())
    }

    #[test]
    fn rejects_missing_read_roots_orphan_mates_and_sample_count_drift() -> Result<()> {
        let empty = tempfile::tempdir()?;
        assert!(discover_normalized_fastq_samples(empty.path(), "corpus", 0).is_err());
        let (root, spec, _) = corpus()?;
        assert!(discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 3).is_err());
        fs::remove_file(root.path().join("normalized/b_R1.fastq.gz"))?;
        assert!(discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 4)
            .unwrap_err()
            .to_string()
            .contains("missing R1"));
        Ok(())
    }

    #[test]
    fn rejects_missing_ambiguous_and_unknown_manifest_accessions() -> Result<()> {
        let (root, spec, files) = corpus()?;
        let samples = discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 4)?;
        let key = "raw/accession_a/a_R1.fastq.gz";
        let mut missing = files.clone();
        let digest = missing.remove(key).context("raw digest")?;
        write_manifest(root.path(), &missing)?;
        assert!(match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples)
            .unwrap_err()
            .to_string()
            .contains("missing accession"));
        missing.insert("raw/unlisted/a_R1.fastq.gz".to_string(), digest.clone());
        write_manifest(root.path(), &missing)?;
        assert!(match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples)
            .unwrap_err()
            .to_string()
            .contains("missing curated metadata"));
        missing.insert(key.to_string(), digest);
        write_manifest(root.path(), &missing)?;
        assert!(match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples)
            .unwrap_err()
            .to_string()
            .contains("expected one accession"));
        Ok(())
    }

    #[test]
    fn rejects_incomplete_cohort_authority_and_count_drift() -> Result<()> {
        let (root, mut spec, _) = corpus()?;
        assert!(expected_fastq_publication_cohort_counts(&spec, "unknown").is_err());
        spec.target_ancient_se = None;
        assert!(expected_fastq_publication_cohort_counts(&spec, "full").is_err());
        spec.target_ancient_se = Some(2);
        let samples = discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 4)?;
        assert!(match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples)
            .unwrap_err()
            .to_string()
            .contains("cohort contract drift"));
        Ok(())
    }

    #[test]
    fn rejects_requested_corpus_identity_mismatch() -> Result<()> {
        let spec = load_fastq_corpus_source_spec(&source_path())?;
        assert_eq!(
            load_fastq_corpus_source_for_id(&source_path(), &spec.corpus_id)?.corpus_id,
            spec.corpus_id
        );
        assert!(load_fastq_corpus_source_for_id(&source_path(), "different-corpus").is_err());
        Ok(())
    }
}
