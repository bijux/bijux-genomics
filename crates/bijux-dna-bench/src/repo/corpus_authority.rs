//! Owner: bijux-dna-bench
//! Admission of governed FASTQ corpora into benchmark execution.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_benchmark_data_index, validate_governed_data_catalog_document,
    AdmittedAlignmentTruthSample, AdmittedAlignmentTruthSet, AdmittedBamCorpusFile,
    AdmittedFastqCorpus, AdmittedFastqCorpusFile, AdmittedFastqCorpusSample, BenchmarkDataIndex,
    BenchmarkDataIndexFile, BenchmarkDataIndexRecord, BenchmarkEligibility, BenchmarkFastqLayout,
    CorpusMaterializationDisposition, DataIntegrityStatus, GovernedDataCatalogDocument,
    GOVERNED_DATA_CATALOG_SCHEMA_V1, GOVERNED_DATA_CATALOG_SCHEMA_V2,
};
use serde::Deserialize;

const MAX_AUTHORITY_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Debug, Deserialize)]
struct DataCatalog {
    schema_version: String,
    id: String,
    samples: Vec<DataCatalogSample>,
    files: Vec<DataCatalogFile>,
}

#[derive(Debug, Deserialize)]
struct DataCatalogSample {
    accession: String,
    layout: String,
}

#[derive(Debug, Deserialize)]
struct DataCatalogFile {
    accession: String,
    kind: String,
    layout: String,
    mate: Option<u8>,
    #[serde(rename = "materialization_status")]
    _materialization_status: CorpusMaterializationDisposition,
    path: String,
    sha256: String,
    size_bytes: u64,
}

/// Admit one exact FASTQ corpus from a governed STUDY data index.
///
/// Admission verifies schema and corpus identity, benchmark eligibility, catalog closure, every
/// declared file's regular-file status, size, and SHA-256 digest, and FASTQ sample layout closure.
/// Returned samples and files have deterministic order independent of source JSON ordering.
///
/// # Errors
/// Returns an error when an authority escapes `study_root`, is a symlink or non-file, exceeds the
/// bounded authority size, has an unsupported schema or duplicate identity, disagrees across the
/// index and catalog, contains an invalid sample layout, or differs from the expected closure.
pub fn admit_fastq_benchmark_corpus(
    study_root: &Path,
    data_index_path: &Path,
    corpus_id: &str,
    expected_sample_count: usize,
    expected_file_count: usize,
) -> Result<AdmittedFastqCorpus> {
    validate_identifier(corpus_id, "corpus")?;
    let study_root = std::fs::canonicalize(study_root)
        .with_context(|| format!("resolve STUDY root {}", study_root.display()))?;
    if !study_root.is_dir() {
        bail!("STUDY root is not a directory: {}", study_root.display());
    }

    let data_index_path = governed_regular_file(&study_root, data_index_path, "data index")?;
    let data_index_bytes = read_authority(&data_index_path, "data index")?;
    let data_index_sha256 = bijux_dna_infra::hash_bytes_sha256(&data_index_bytes);
    let index: BenchmarkDataIndex = serde_json::from_slice(&data_index_bytes)
        .with_context(|| format!("parse data index {}", data_index_path.display()))?;
    validate_benchmark_data_index(&index).context("validate benchmark data index")?;
    let mut records = index.records.into_iter().filter(|record| record.id == corpus_id);
    let record = records.next().with_context(|| format!("data index lacks corpus {corpus_id}"))?;
    if records.next().is_some() {
        bail!("data index corpus identity is not unique: {corpus_id}");
    }
    if record.benchmark_eligibility != BenchmarkEligibility::Eligible {
        bail!("corpus is not benchmark-eligible: {corpus_id}");
    }
    if record.integrity_status != DataIntegrityStatus::Verified {
        bail!("corpus lacks verified index integrity: {corpus_id}");
    }
    validate_sha256(&record.catalog_sha256, "catalog")?;

    let catalog_path = resolve_study_uri(&study_root, &record.catalog_path, "catalog")?;
    let catalog_path = governed_regular_file(&study_root, &catalog_path, "catalog")?;
    let catalog_bytes = read_authority(&catalog_path, "catalog")?;
    let catalog_sha256 = bijux_dna_infra::hash_bytes_sha256(&catalog_bytes);
    if catalog_sha256 != record.catalog_sha256 {
        bail!("corpus catalog digest differs from the data index");
    }
    let catalog = parse_data_catalog(&catalog_bytes, &catalog_path)?;
    if !matches!(
        catalog.schema_version.as_str(),
        GOVERNED_DATA_CATALOG_SCHEMA_V1 | GOVERNED_DATA_CATALOG_SCHEMA_V2
    ) || catalog.id != corpus_id
    {
        bail!("corpus catalog identity is unsupported");
    }
    let raw_fastq_count = catalog.files.iter().filter(|file| file.kind == "raw-fastq").count();
    let indexed_raw_fastq_count =
        record.files.iter().filter(|file| file.role == "raw-fastq").count();
    if catalog.samples.len() != expected_sample_count
        || raw_fastq_count != expected_file_count
        || indexed_raw_fastq_count != expected_file_count
    {
        bail!(
            "corpus closure differs from expected {expected_sample_count} samples and \
             {expected_file_count} files"
        );
    }

    let indexed_files = index_files(record.files)?;
    let sample_layouts = sample_layouts(catalog.samples)?;
    let mut catalog_names = BTreeSet::new();
    let mut files_by_sample = BTreeMap::<String, Vec<AdmittedFastqCorpusFile>>::new();
    let mut input_bytes = 0_u64;

    for file in catalog.files {
        if file.kind != "raw-fastq" {
            continue;
        }
        let layout = parse_layout(&file.layout)?;
        let declared_layout = sample_layouts
            .get(&file.accession)
            .with_context(|| format!("catalog file has unknown sample {}", file.accession))?;
        if layout != *declared_layout {
            bail!("catalog file layout disagrees for sample {}", file.accession);
        }
        validate_mate(layout, file.mate, &file.accession)?;
        validate_sha256(&file.sha256, "catalog file")?;
        let name = file_name(&file.path, "catalog file")?;
        validate_fastq_name(&name)?;
        validate_accession_parent(&file.path, &file.accession, "catalog file")?;
        if !catalog_names.insert(name.clone()) {
            bail!("corpus catalog contains duplicate file name {name}");
        }
        let indexed = indexed_files
            .get(&name)
            .with_context(|| format!("data index lacks catalog file {name}"))?;
        if indexed.sha256 != file.sha256 || indexed.size_bytes != file.size_bytes {
            bail!("catalog and data index disagree for file {name}");
        }
        if indexed.role != "raw-fastq" {
            bail!("data index file role is not raw-fastq: {name}");
        }
        validate_accession_parent(&indexed.path, &file.accession, "data index file")?;
        let governed_path = resolve_study_uri(&study_root, &indexed.path, "corpus file")?;
        let governed_path = governed_regular_file(&study_root, &governed_path, "corpus file")?;
        let metadata = std::fs::metadata(&governed_path)
            .with_context(|| format!("inspect corpus file {}", governed_path.display()))?;
        if metadata.len() != indexed.size_bytes {
            bail!("corpus file size differs from its authorities: {name}");
        }
        let observed_sha256 = bijux_dna_infra::hash_file_sha256(&governed_path)
            .with_context(|| format!("hash corpus file {}", governed_path.display()))?;
        if observed_sha256 != indexed.sha256 {
            bail!("corpus file digest differs from its authorities: {name}");
        }
        input_bytes =
            input_bytes.checked_add(indexed.size_bytes).context("corpus size overflow")?;
        files_by_sample.entry(file.accession).or_default().push(AdmittedFastqCorpusFile {
            path: path_string(&governed_path)?,
            sha256: indexed.sha256.clone(),
            size_bytes: indexed.size_bytes,
            mate: file.mate,
        });
    }
    let indexed_fastq_names = indexed_files
        .iter()
        .filter(|(_, file)| file.role == "raw-fastq")
        .map(|(name, _)| name.clone())
        .collect::<BTreeSet<_>>();
    if catalog_names != indexed_fastq_names {
        bail!("data index and catalog FASTQ closures differ");
    }

    let mut samples = Vec::with_capacity(sample_layouts.len());
    for (sample_id, input_layout) in sample_layouts {
        let mut files = files_by_sample.remove(&sample_id).unwrap_or_default();
        validate_file_closure(input_layout, &files, &sample_id)?;
        files.sort_by(|left, right| left.mate.cmp(&right.mate).then(left.path.cmp(&right.path)));
        samples.push(AdmittedFastqCorpusSample { sample_id, input_layout, files });
    }
    if !files_by_sample.is_empty() {
        bail!("catalog contains files outside the declared sample closure");
    }

    Ok(AdmittedFastqCorpus {
        corpus_id: corpus_id.to_string(),
        data_index_path: path_string(&data_index_path)?,
        data_index_sha256,
        catalog_path: path_string(&catalog_path)?,
        catalog_sha256,
        sample_count: u64::try_from(samples.len()).context("sample count overflow")?,
        file_count: u64::try_from(catalog_names.len()).context("file count overflow")?,
        input_bytes,
        samples,
    })
}

/// Admit exact read-placement truth paired with a governed composite FASTQ catalog.
///
/// The catalog may contain raw FASTQ and truth rows. This admission selects only
/// `alignment-truth` files while proving that every declared sample has exactly one truth table
/// and that the data index agrees with every selected byte identity.
///
/// # Errors
/// Returns an error when the authority is unsafe, incomplete, ineligible, or digest-drifted.
pub fn admit_alignment_truth_set(
    study_root: &Path,
    data_index_path: &Path,
    truth_set_id: &str,
    expected_sample_count: usize,
) -> Result<AdmittedAlignmentTruthSet> {
    validate_identifier(truth_set_id, "truth set")?;
    let study_root = std::fs::canonicalize(study_root)
        .with_context(|| format!("resolve STUDY root {}", study_root.display()))?;
    let data_index_path = governed_regular_file(&study_root, data_index_path, "data index")?;
    let data_index_bytes = read_authority(&data_index_path, "data index")?;
    let data_index_sha256 = bijux_dna_infra::hash_bytes_sha256(&data_index_bytes);
    let index: BenchmarkDataIndex = serde_json::from_slice(&data_index_bytes)
        .with_context(|| format!("parse data index {}", data_index_path.display()))?;
    validate_benchmark_data_index(&index).context("validate benchmark data index")?;
    let record = unique_eligible_record(index.records, truth_set_id)?;
    let catalog_path = resolve_study_uri(&study_root, &record.catalog_path, "catalog")?;
    let catalog_path = governed_regular_file(&study_root, &catalog_path, "catalog")?;
    let catalog_bytes = read_authority(&catalog_path, "catalog")?;
    let catalog_sha256 = bijux_dna_infra::hash_bytes_sha256(&catalog_bytes);
    if catalog_sha256 != record.catalog_sha256 {
        bail!("alignment truth catalog digest differs from the data index");
    }
    let catalog = parse_data_catalog(&catalog_bytes, &catalog_path)?;
    if !matches!(
        catalog.schema_version.as_str(),
        GOVERNED_DATA_CATALOG_SCHEMA_V1 | GOVERNED_DATA_CATALOG_SCHEMA_V2
    ) || catalog.id != truth_set_id
        || catalog.samples.len() != expected_sample_count
    {
        bail!("alignment truth catalog identity or sample closure is unsupported");
    }

    let indexed_files = index_files(record.files)?;
    let declared_samples = sample_layouts(catalog.samples)?.into_keys().collect::<BTreeSet<_>>();
    let mut truth_names = BTreeSet::new();
    let mut samples = BTreeMap::new();
    let mut input_bytes = 0_u64;
    for file in catalog.files.into_iter().filter(|file| file.kind == "alignment-truth") {
        if file.mate.is_some() {
            bail!("alignment truth is not an admitted non-mate file for {}", file.accession);
        }
        if !declared_samples.contains(&file.accession) {
            bail!("alignment truth names an undeclared sample {}", file.accession);
        }
        validate_sha256(&file.sha256, "alignment truth")?;
        validate_accession_parent(&file.path, &file.accession, "alignment truth")?;
        let name = file_name(&file.path, "alignment truth")?;
        if !name.ends_with(".tsv") || !truth_names.insert(name.clone()) {
            bail!("alignment truth file name is invalid or duplicated: {name}");
        }
        let indexed = indexed_files
            .get(&name)
            .with_context(|| format!("data index lacks alignment truth file {name}"))?;
        if indexed.role != "alignment-truth"
            || indexed.sha256 != file.sha256
            || indexed.size_bytes != file.size_bytes
        {
            bail!("alignment truth catalog and data index disagree for {name}");
        }
        let path = governed_regular_file(
            &study_root,
            &resolve_study_uri(&study_root, &indexed.path, "alignment truth")?,
            "alignment truth",
        )?;
        let observed_size = std::fs::metadata(&path)?.len();
        if observed_size != indexed.size_bytes
            || bijux_dna_infra::hash_file_sha256(&path)? != indexed.sha256
        {
            bail!("alignment truth bytes differ from their authorities: {name}");
        }
        input_bytes = input_bytes.checked_add(observed_size).context("truth size overflow")?;
        let sample_id = file.accession;
        let truth = AdmittedAlignmentTruthSample {
            sample_id: sample_id.clone(),
            truth: AdmittedBamCorpusFile {
                path: path_string(&path)?,
                sha256: indexed.sha256.clone(),
                size_bytes: observed_size,
            },
        };
        if samples.insert(sample_id.clone(), truth).is_some() {
            bail!("alignment truth repeats sample {sample_id}");
        }
    }
    let indexed_truth_names = indexed_files
        .iter()
        .filter(|(_, file)| file.role == "alignment-truth")
        .map(|(name, _)| name.clone())
        .collect::<BTreeSet<_>>();
    if samples.keys().collect::<BTreeSet<_>>() != declared_samples.iter().collect()
        || truth_names != indexed_truth_names
        || samples.len() != expected_sample_count
    {
        bail!("alignment truth does not close over the governed sample set");
    }
    Ok(AdmittedAlignmentTruthSet {
        truth_set_id: truth_set_id.to_string(),
        data_index_path: path_string(&data_index_path)?,
        data_index_sha256,
        catalog_path: path_string(&catalog_path)?,
        catalog_sha256,
        sample_count: u64::try_from(samples.len()).context("truth sample count")?,
        file_count: u64::try_from(truth_names.len()).context("truth file count")?,
        input_bytes,
        samples: samples.into_values().collect(),
    })
}

fn unique_eligible_record(
    records: Vec<BenchmarkDataIndexRecord>,
    id: &str,
) -> Result<BenchmarkDataIndexRecord> {
    let mut matching = records.into_iter().filter(|record| record.id == id);
    let record = matching.next().with_context(|| format!("data index lacks asset {id}"))?;
    if matching.next().is_some()
        || record.benchmark_eligibility != BenchmarkEligibility::Eligible
        || record.integrity_status != DataIntegrityStatus::Verified
    {
        bail!("data index asset is ambiguous or not admitted: {id}");
    }
    Ok(record)
}

fn parse_data_catalog(bytes: &[u8], path: &Path) -> Result<DataCatalog> {
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .with_context(|| format!("parse governed data catalog {}", path.display()))?;
    if value.get("samples").is_some() && value.get("files").is_some() {
        return serde_json::from_value(value)
            .with_context(|| format!("decode data catalog {}", path.display()));
    }
    let document: GovernedDataCatalogDocument = serde_json::from_value(value)
        .with_context(|| format!("decode governed catalog snapshot {}", path.display()))?;
    validate_governed_data_catalog_document(&document)
        .context("validate governed catalog snapshot")?;
    match document {
        GovernedDataCatalogDocument::Preserved(catalog) => {
            serde_json::from_value(catalog.preserved_catalog)
                .with_context(|| format!("decode preserved data catalog {}", path.display()))
        }
        GovernedDataCatalogDocument::Materialized(_) => {
            bail!("native governed data catalog omitted its sample or file closure")
        }
    }
}

fn index_files(
    files: Vec<BenchmarkDataIndexFile>,
) -> Result<BTreeMap<String, BenchmarkDataIndexFile>> {
    let mut indexed = BTreeMap::new();
    for file in files {
        validate_sha256(&file.sha256, "data index file")?;
        let name = file_name(&file.path, "data index file")?;
        if indexed.insert(name.clone(), file).is_some() {
            bail!("data index contains duplicate file name {name}");
        }
    }
    Ok(indexed)
}

fn sample_layouts(
    samples: Vec<DataCatalogSample>,
) -> Result<BTreeMap<String, BenchmarkFastqLayout>> {
    let mut layouts = BTreeMap::new();
    for sample in samples {
        validate_identifier(&sample.accession, "sample")?;
        let layout = parse_layout(&sample.layout)?;
        if layouts.insert(sample.accession.clone(), layout).is_some() {
            bail!("corpus catalog contains duplicate sample {}", sample.accession);
        }
    }
    Ok(layouts)
}

fn parse_layout(layout: &str) -> Result<BenchmarkFastqLayout> {
    match layout {
        "se" => Ok(BenchmarkFastqLayout::SingleEnd),
        "pe" => Ok(BenchmarkFastqLayout::PairedEnd),
        _ => bail!("FASTQ catalog layout is unsupported: {layout}"),
    }
}

fn validate_mate(layout: BenchmarkFastqLayout, mate: Option<u8>, sample_id: &str) -> Result<()> {
    let valid = match layout {
        BenchmarkFastqLayout::SingleEnd => mate.is_none(),
        BenchmarkFastqLayout::PairedEnd => matches!(mate, Some(1 | 2)),
    };
    if !valid {
        bail!("FASTQ mate declaration is invalid for sample {sample_id}");
    }
    Ok(())
}

fn validate_file_closure(
    layout: BenchmarkFastqLayout,
    files: &[AdmittedFastqCorpusFile],
    sample_id: &str,
) -> Result<()> {
    let valid = match layout {
        BenchmarkFastqLayout::SingleEnd => files.len() == 1 && files[0].mate.is_none(),
        BenchmarkFastqLayout::PairedEnd => {
            files.len() == 2
                && files.iter().map(|file| file.mate).collect::<BTreeSet<_>>()
                    == BTreeSet::from([Some(1), Some(2)])
        }
    };
    if !valid {
        bail!("FASTQ file closure disagrees with layout for sample {sample_id}");
    }
    Ok(())
}

fn governed_regular_file(study_root: &Path, path: &Path, label: &str) -> Result<PathBuf> {
    let metadata = std::fs::symlink_metadata(path)
        .with_context(|| format!("inspect {label} {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("{label} is not a regular non-symlink file: {}", path.display());
    }
    let resolved = std::fs::canonicalize(path)
        .with_context(|| format!("resolve {label} {}", path.display()))?;
    if !resolved.starts_with(study_root) {
        bail!("{label} escapes the STUDY root: {}", path.display());
    }
    Ok(resolved)
}

fn resolve_study_uri(study_root: &Path, uri: &str, label: &str) -> Result<PathBuf> {
    let relative = uri
        .strip_prefix("study://")
        .with_context(|| format!("{label} path is not a STUDY URI: {uri}"))?;
    let relative = Path::new(relative);
    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || relative.components().any(|component| !matches!(component, Component::Normal(_)))
    {
        bail!("{label} path is not a normalized STUDY URI: {uri}");
    }
    Ok(study_root.join(relative))
}

fn read_authority(path: &Path, label: &str) -> Result<Vec<u8>> {
    let size = std::fs::metadata(path)
        .with_context(|| format!("inspect {label} {}", path.display()))?
        .len();
    if size > MAX_AUTHORITY_BYTES {
        bail!("{label} exceeds the bounded authority size: {}", path.display());
    }
    std::fs::read(path).with_context(|| format!("read {label} {}", path.display()))
}

fn validate_identifier(value: &str, label: &str) -> Result<()> {
    if value.is_empty()
        || value != value.trim()
        || value.bytes().any(|byte| byte.is_ascii_whitespace())
    {
        bail!("{label} identifier is empty or malformed");
    }
    Ok(())
}

fn validate_sha256(value: &str, label: &str) -> Result<()> {
    if value.len() != 64
        || !value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("{label} SHA-256 identity is malformed");
    }
    Ok(())
}

fn file_name(path: &str, label: &str) -> Result<String> {
    Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .with_context(|| format!("{label} path lacks a UTF-8 file name: {path}"))
}

fn validate_fastq_name(name: &str) -> Result<()> {
    if !name.ends_with(".fastq") && !name.ends_with(".fastq.gz") {
        bail!("corpus file does not have a FASTQ file name: {name}");
    }
    Ok(())
}

fn validate_accession_parent(path: &str, accession: &str, label: &str) -> Result<()> {
    let parent =
        Path::new(path).parent().and_then(Path::file_name).and_then(|value| value.to_str());
    if parent != Some(accession) {
        bail!("{label} path does not belong to accession {accession}: {path}");
    }
    Ok(())
}

fn path_string(path: &Path) -> Result<String> {
    path.to_str().map(str::to_string).context("governed path is not valid UTF-8")
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::parse_data_catalog;

    #[test]
    fn preserved_composite_catalog_retains_fastq_and_alignment_truth_rows() -> anyhow::Result<()> {
        let inner = json!({
            "schema_version": "bijux.data_catalog.v1",
            "id": "alignment-truth-panel",
            "purpose": "controlled alignment truth",
            "access_classification": "controlled",
            "redistribution_classification": "derived-results-only",
            "known_limitations": ["authorization required"],
            "samples": [{"accession": "sample-a", "layout": "pe"}],
            "files": [
                {
                    "accession": "sample-a", "kind": "raw-fastq", "layout": "pe",
                    "mate": 1, "materialization_status": "downloaded",
                    "path": "/study/data/alignment-truth-panel/sample-a/sample-a_R1.fastq.gz",
                    "sha256": "a".repeat(64), "size_bytes": 10
                },
                {
                    "accession": "sample-a", "kind": "raw-fastq", "layout": "pe",
                    "mate": 2, "materialization_status": "downloaded",
                    "path": "/study/data/alignment-truth-panel/sample-a/sample-a_R2.fastq.gz",
                    "sha256": "b".repeat(64), "size_bytes": 10
                },
                {
                    "accession": "sample-a", "kind": "alignment-truth", "layout": "pe",
                    "mate": null, "materialization_status": "downloaded",
                    "path": "/study/data/alignment-truth-panel/sample-a/sample-a.truth.tsv",
                    "sha256": "c".repeat(64), "size_bytes": 3
                }
            ]
        });
        let envelope = json!({
            "schema_version": "bijux.data_catalog.v2",
            "id": "alignment-truth-panel",
            "source_catalog_path": "study://data/catalog/alignment-truth-panel.json",
            "source_catalog_sha256": "d".repeat(64),
            "source_catalog_size_bytes": 1,
            "source_catalog_schema_version": "bijux.data_catalog.v1",
            "preserved_at_utc": "2026-08-11T00:00:00Z",
            "preserved_catalog": inner
        });
        let catalog = parse_data_catalog(
            &serde_json::to_vec(&envelope)?,
            std::path::Path::new("/study/data/catalog/snapshots/source/alignment-truth-panel.json"),
        )?;
        assert_eq!(catalog.samples.len(), 1);
        assert_eq!(catalog.files.iter().filter(|file| file.kind == "raw-fastq").count(), 2);
        assert_eq!(catalog.files.iter().filter(|file| file.kind == "alignment-truth").count(), 1);
        Ok(())
    }
}
