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
mod tests {
    //! Owner: bijux-dna-bench
    //! Corpus admission regressions.

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
        assert!(match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples).is_err());
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

    #[test]
    fn normalized_metadata_rejects_conflicting_mate_accessions() -> Result<()> {
        let (root, spec, mut files) = corpus()?;
        let samples = discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 4)?;
        files.insert(
            "normalized/b_R1.fastq.gz".to_string(),
            files["raw/accession_d/d_R1.fastq.gz"].clone(),
        );
        write_manifest(root.path(), &files)?;
        assert!(match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples).is_err());
        Ok(())
    }

    #[test]
    fn normalized_metadata_rejects_a_missing_curated_mate() -> Result<()> {
        let (root, spec, mut files) = corpus()?;
        fs::remove_file(root.path().join("normalized/b_R2.fastq.gz"))?;
        files.remove("normalized/b_R2.fastq.gz");
        write_manifest(root.path(), &files)?;
        let samples = discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 4)?;
        assert!(match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples).is_err());
        Ok(())
    }

    #[test]
    fn normalized_metadata_rejects_incoherent_sample_layout() -> Result<()> {
        let (root, spec, _) = corpus()?;
        let mut samples = discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 4)?;
        samples[1].layout = "se".to_string();
        assert!(match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples).is_err());
        Ok(())
    }

    #[test]
    fn normalized_metadata_rejects_missing_selection_metadata() -> Result<()> {
        let (root, spec, _) = corpus()?;
        let samples = discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 4)?;
        let mut metadata = match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples)?;
        metadata.remove("a");
        assert!(select_normalized_fastq_paired_samples(&spec, &samples, &metadata).is_err());
        Ok(())
    }

    #[test]
    fn normalized_metadata_accepts_zero_target_paired_cohorts() -> Result<()> {
        let (root, mut spec, _) = corpus()?;
        let samples = discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 4)?;
        let mut metadata = match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples)?;
        spec.target_total = 1;
        spec.target_ancient_se = Some(0);
        spec.target_ancient_pe = Some(0);
        spec.target_modern_se = Some(0);
        spec.target_modern_pe = Some(1);
        spec.samples.retain(|row| row.accession == "accession_d");
        metadata.retain(|id, _| id == "d");
        let selected =
            select_normalized_fastq_paired_samples(&spec, &[samples[3].clone()], &metadata)?;
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].sample_id, "d");
        Ok(())
    }

    #[test]
    fn normalized_metadata_groups_equal_mate_digests_by_accession() -> Result<()> {
        let (root, spec, mut files) = corpus()?;
        let mut gzip = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        gzip.write_all(b"@b\nACGT\n+\nIIII\n")?;
        let bytes = gzip.finish()?;
        let digest = bijux_dna_infra::hash_bytes_sha256(&bytes);
        for mate in [1, 2] {
            for relative in [
                format!("normalized/b_R{mate}.fastq.gz"),
                format!("raw/accession_b/b_R{mate}.fastq.gz"),
            ] {
                fs::write(root.path().join(&relative), &bytes)?;
                files.insert(relative, digest.clone());
            }
        }
        write_manifest(root.path(), &files)?;
        let samples = discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 4)?;
        let metadata = match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples)?;
        assert_eq!(metadata["b"].accession, "accession_b");
        Ok(())
    }

    #[test]
    fn rejects_repeated_source_sample_and_accession_identities() -> Result<()> {
        let (root, mut spec, _) = corpus()?;
        let samples = discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 4)?;
        let metadata = match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples)?;
        let mut duplicate_samples = samples.clone();
        duplicate_samples[1] = samples[0].clone();
        assert!(
            select_normalized_fastq_paired_samples(&spec, &duplicate_samples, &metadata).is_err()
        );
        let mut duplicate_accessions = metadata.clone();
        duplicate_accessions.insert("c".to_string(), metadata["a"].clone());
        assert!(
            select_normalized_fastq_paired_samples(&spec, &samples, &duplicate_accessions).is_err()
        );
        spec.samples[1] = spec.samples[0].clone();
        assert!(match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples).is_err());
        Ok(())
    }

    #[test]
    fn rejects_missing_manifest_mates_and_redirected_sample_paths() -> Result<()> {
        let (root, spec, mut files) = corpus()?;
        let mut samples = discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 4)?;
        samples[1].r1 = samples[3].r1.clone();
        assert!(match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples).is_err());
        samples[1].r1 = root.path().join("normalized/b_R1.fastq.gz");
        files.remove("normalized/b_R1.fastq.gz");
        write_manifest(root.path(), &files)?;
        assert!(match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples).is_err());
        Ok(())
    }

    #[test]
    fn rejects_stale_normalized_and_raw_content() -> Result<()> {
        for relative in ["normalized/b_R1.fastq.gz", "raw/accession_b/b_R1.fastq.gz"] {
            let (root, spec, _) = corpus()?;
            let samples = discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 4)?;
            fs::write(root.path().join(relative), b"changed bytes")?;
            let err =
                match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples).unwrap_err();
            assert!(err.to_string().contains("checksum mismatch"), "{err:#}");
        }
        Ok(())
    }

    #[test]
    fn rejects_ambiguous_or_unsupported_manifest_documents() -> Result<()> {
        let (root, spec, files) = corpus()?;
        let samples = discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 4)?;
        let map = serde_json::to_string(&files)?;
        let digest = &files["normalized/a_R1.fastq.gz"];
        for document in [
            format!("{{\"files\":{map},\"files\":{map}}}"),
            format!("{{\"files\":{{\"a\":\"{digest}\",\"a\":\"{digest}\"}}}}"),
            format!("{{\"schema_version\":\"unknown\",\"files\":{map}}}"),
            format!("{{\"unknown\":true,\"files\":{map}}}"),
            "{}".to_string(),
            "{\"files\":{}}".to_string(),
        ] {
            fs::write(root.path().join("MANIFEST.json"), document)?;
            assert!(match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples).is_err());
        }
        // Both the explicit schema and its supported legacy omission have the same identity contract.
        fs::write(
            root.path().join("MANIFEST.json"),
            format!("{{\"schema_version\":\"bijux.corpus_manifest.v1\",\"files\":{map}}}"),
        )?;
        match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples)?;
        Ok(())
    }

    #[test]
    fn rejects_noncanonical_manifest_paths_and_digests() -> Result<()> {
        let (root, spec, files) = corpus()?;
        let samples = discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 4)?;
        for path in [
            "../outside",
            "/absolute",
            "raw/../outside",
            "raw/./file",
            "raw//file",
            "raw\\file",
            "",
        ] {
            let mut invalid = files.clone();
            invalid.insert(path.to_string(), "a".repeat(64));
            write_manifest(root.path(), &invalid)?;
            assert!(
                match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples).is_err(),
                "{path}"
            );
        }
        for digest in ["", "abc", &"A".repeat(64), &"g".repeat(64)] {
            let mut invalid = files.clone();
            invalid.insert("normalized/a_R1.fastq.gz".to_string(), digest.to_string());
            write_manifest(root.path(), &invalid)?;
            assert!(match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples).is_err());
        }
        Ok(())
    }

    #[test]
    fn rejects_oversized_manifest_before_decoding() -> Result<()> {
        let (root, spec, _) = corpus()?;
        let samples = discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 4)?;
        let file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(root.path().join("MANIFEST.json"))?;
        file.set_len(64 * 1024 * 1024 + 1)?;
        let err = match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples).unwrap_err();
        assert!(err.to_string().contains("exceeds"), "{err:#}");
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinked_manifest_reads_and_raw_directories() -> Result<()> {
        for relative in ["MANIFEST.json", "normalized/a_R1.fastq.gz", "raw/accession_b"] {
            let (root, spec, _) = corpus()?;
            let samples = discover_normalized_fastq_samples(root.path(), &spec.corpus_id, 4)?;
            let original = root.path().join(relative);
            let saved = root.path().join("preserved");
            fs::rename(&original, &saved)?;
            std::os::unix::fs::symlink(&saved, &original)?;
            let err =
                match_normalized_fastq_corpus_metadata(root.path(), &spec, &samples).unwrap_err();
            assert!(err.to_string().contains("symlink"), "{err:#}");
        }
        Ok(())
    }
}
