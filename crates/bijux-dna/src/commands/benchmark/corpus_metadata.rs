use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

use crate::commands::benchmark_workspace::benchmark_corpus_spec_path;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CorpusSpec {
    pub(crate) corpus_id: String,
    #[serde(default)]
    pub(crate) target_ancient_se: usize,
    #[serde(default)]
    pub(crate) target_ancient_pe: usize,
    #[serde(default)]
    pub(crate) target_modern_se: usize,
    #[serde(default)]
    pub(crate) target_modern_pe: usize,
    #[serde(default)]
    pub(crate) samples: Vec<CorpusSpecSample>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub(crate) struct CorpusSpecSample {
    pub(crate) accession: String,
    #[serde(default)]
    pub(crate) study_accession: String,
    pub(crate) era: String,
    pub(crate) layout: String,
    #[serde(default)]
    pub(crate) size_band: String,
    #[serde(default)]
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct CorpusManifest {
    #[serde(default)]
    files: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub(crate) struct CorpusNormalizedSample {
    pub(crate) sample_id: String,
    pub(crate) r1: PathBuf,
    pub(crate) r2: Option<PathBuf>,
    pub(crate) layout: String,
}

pub(crate) fn load_corpus_spec(
    repo_root: &Path,
    config_path: Option<&Path>,
    corpus_id: &str,
) -> Result<CorpusSpec> {
    let path = benchmark_corpus_spec_path(repo_root, config_path, corpus_id)?;
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

pub(crate) fn corpus_expected_sample_total(spec: &CorpusSpec) -> usize {
    spec.target_ancient_se + spec.target_ancient_pe + spec.target_modern_se + spec.target_modern_pe
}

pub(crate) fn expected_counts_for_scope(
    spec: &CorpusSpec,
    sample_scope: &str,
) -> Result<BTreeMap<String, usize>> {
    match sample_scope {
        "full" => Ok(BTreeMap::from([
            ("ancient_pe".to_string(), spec.target_ancient_pe),
            ("ancient_se".to_string(), spec.target_ancient_se),
            ("modern_pe".to_string(), spec.target_modern_pe),
            ("modern_se".to_string(), spec.target_modern_se),
        ])),
        "paired" => Ok(BTreeMap::from([
            ("ancient_pe".to_string(), spec.target_ancient_pe),
            ("modern_pe".to_string(), spec.target_modern_pe),
        ])),
        other => Err(anyhow!("unsupported corpus benchmark sample scope `{other}`")),
    }
}

pub(crate) fn discover_normalized_samples(
    corpus_root: &Path,
    corpus_id: &str,
    expected_total: usize,
) -> Result<Vec<CorpusNormalizedSample>> {
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
        samples.push(CorpusNormalizedSample {
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

pub(crate) fn validate_corpus_contract(
    corpus_root: &Path,
    spec: &CorpusSpec,
    samples: &[CorpusNormalizedSample],
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

    let mut actual_counts = expected_counts_for_scope(spec, "full")?;
    for count in actual_counts.values_mut() {
        *count = 0;
    }
    for sample in samples {
        let metadata = metadata_by_sample
            .get(&sample.sample_id)
            .ok_or_else(|| anyhow!("missing accession metadata for {}", sample.sample_id))?;
        *actual_counts.entry(format!("{}_{}", metadata.era, metadata.layout)).or_default() += 1;
    }

    let expected_counts = expected_counts_for_scope(spec, "full")?;
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

pub(crate) fn select_paired_samples(
    spec: &CorpusSpec,
    samples: &[CorpusNormalizedSample],
    metadata_by_sample: &BTreeMap<String, CorpusSpecSample>,
) -> Result<Vec<CorpusNormalizedSample>> {
    let paired = samples
        .iter()
        .filter(|row| {
            metadata_by_sample.get(&row.sample_id).is_some_and(|meta| meta.layout == "pe")
        })
        .cloned()
        .collect::<Vec<_>>();
    let mut actual_counts = BTreeMap::<String, usize>::new();
    for sample in &paired {
        let metadata = metadata_by_sample
            .get(&sample.sample_id)
            .ok_or_else(|| anyhow!("missing paired metadata for {}", sample.sample_id))?;
        *actual_counts.entry(format!("{}_{}", metadata.era, metadata.layout)).or_default() += 1;
    }
    let expected_counts = expected_counts_for_scope(spec, "paired")?;
    if actual_counts != expected_counts {
        return Err(anyhow!(
            "paired corpus contract drift: expected {expected_counts:?}, found {actual_counts:?}"
        ));
    }
    Ok(paired)
}
