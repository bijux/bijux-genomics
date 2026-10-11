//! Owner: bijux-dna-bench
//! Admission of governed BAM corpora into benchmark execution.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{AdmittedBamCorpus, AdmittedBamCorpusFile};
use serde::Deserialize;

const DATA_INDEX_SCHEMA: &str = "bijux.data_index.v1";
const DATA_CATALOG_SCHEMA: &str = "bijux.data_catalog.v1";
const BAM_ROLE: &str = "coordinate-sorted-deduplicated-bam";
const BAI_ROLE: &str = "bam-index";
const MAX_AUTHORITY_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Debug, Deserialize)]
struct DataIndex {
    schema_version: String,
    records: Vec<DataIndexRecord>,
}

#[derive(Debug, Deserialize)]
struct DataIndexRecord {
    id: String,
    benchmark_eligibility: String,
    integrity_status: String,
    catalog_path: String,
    catalog_sha256: String,
    files: Vec<AuthorityFile>,
}

#[derive(Debug, Deserialize)]
struct AuthorityFile {
    path: String,
    role: String,
    sha256: String,
    size_bytes: u64,
}

#[derive(Debug, Deserialize)]
struct DataCatalog {
    schema_version: String,
    id: String,
    run_accession: String,
    files: Vec<CatalogFile>,
}

#[derive(Debug, Deserialize)]
struct CatalogFile {
    path: String,
    kind: String,
    sha256: String,
    size_bytes: u64,
}

/// Admit one coordinate-sorted BAM and its index from governed STUDY authorities.
///
/// # Errors
/// Returns an error when identities, schemas, eligibility, file roles, paths, sizes, or digests
/// differ across the index, catalog, and materialized files.
pub fn admit_bam_benchmark_corpus(
    study_root: &Path,
    data_index_path: &Path,
    corpus_id: &str,
) -> Result<AdmittedBamCorpus> {
    validate_identifier(corpus_id, "corpus")?;
    let study_root = std::fs::canonicalize(study_root)
        .with_context(|| format!("resolve STUDY root {}", study_root.display()))?;
    if !study_root.is_dir() {
        bail!("STUDY root is not a directory: {}", study_root.display());
    }

    let data_index_path = governed_regular_file(&study_root, data_index_path, "data index")?;
    let data_index_bytes = read_authority(&data_index_path, "data index")?;
    let data_index_sha256 = bijux_dna_infra::hash_bytes_sha256(&data_index_bytes);
    let index: DataIndex = serde_json::from_slice(&data_index_bytes)
        .with_context(|| format!("parse data index {}", data_index_path.display()))?;
    if index.schema_version != DATA_INDEX_SCHEMA {
        bail!("data index schema is unsupported: {}", index.schema_version);
    }
    let mut records = index.records.into_iter().filter(|record| record.id == corpus_id);
    let record = records.next().with_context(|| format!("data index lacks corpus {corpus_id}"))?;
    if records.next().is_some() {
        bail!("data index corpus identity is not unique: {corpus_id}");
    }
    if record.benchmark_eligibility != "eligible" || record.integrity_status != "verified" {
        bail!("BAM corpus is not eligible with verified integrity: {corpus_id}");
    }
    validate_sha256(&record.catalog_sha256, "catalog")?;

    let catalog_path = resolve_study_uri(&study_root, &record.catalog_path, "catalog")?;
    let catalog_path = governed_regular_file(&study_root, &catalog_path, "catalog")?;
    let catalog_bytes = read_authority(&catalog_path, "catalog")?;
    let catalog_sha256 = bijux_dna_infra::hash_bytes_sha256(&catalog_bytes);
    if catalog_sha256 != record.catalog_sha256 {
        bail!("BAM corpus catalog digest differs from the data index");
    }
    let catalog: DataCatalog = serde_json::from_slice(&catalog_bytes)
        .with_context(|| format!("parse BAM corpus catalog {}", catalog_path.display()))?;
    if catalog.schema_version != DATA_CATALOG_SCHEMA || catalog.id != corpus_id {
        bail!("BAM corpus catalog identity is unsupported");
    }
    validate_identifier(&catalog.run_accession, "BAM sample")?;

    let indexed = files_by_role(record.files, "data index")?;
    let catalog_files = catalog_files_by_role(catalog.files)?;
    if indexed.keys().collect::<BTreeSet<_>>() != catalog_files.keys().collect::<BTreeSet<_>>() {
        bail!("BAM data index and catalog file-role closures differ");
    }
    for (role, index_file) in &indexed {
        let catalog_file = catalog_files
            .get(role)
            .with_context(|| format!("BAM catalog lacks indexed role {role}"))?;
        if file_name(&index_file.path, "data index file")?
            != file_name(&catalog_file.path, "catalog file")?
            || index_file.sha256 != catalog_file.sha256
            || index_file.size_bytes != catalog_file.size_bytes
        {
            bail!("BAM catalog and data index disagree for role {role}");
        }
    }

    let bam = admit_materialized_file(&study_root, required_role(&indexed, BAM_ROLE)?, BAM_ROLE)?;
    let bam_index =
        admit_materialized_file(&study_root, required_role(&indexed, BAI_ROLE)?, BAI_ROLE)?;
    validate_bam_pair(&bam, &bam_index, &catalog.run_accession)?;
    let input_bytes =
        bam.size_bytes.checked_add(bam_index.size_bytes).context("BAM corpus size overflow")?;

    Ok(AdmittedBamCorpus {
        corpus_id: corpus_id.to_string(),
        sample_id: catalog.run_accession,
        data_index_path: path_string(&data_index_path)?,
        data_index_sha256,
        catalog_path: path_string(&catalog_path)?,
        catalog_sha256,
        bam,
        bam_index,
        input_bytes,
    })
}

fn files_by_role(
    files: Vec<AuthorityFile>,
    authority: &str,
) -> Result<BTreeMap<String, AuthorityFile>> {
    let mut indexed = BTreeMap::new();
    for file in files {
        validate_sha256(&file.sha256, authority)?;
        if indexed.insert(file.role.clone(), file).is_some() {
            bail!("{authority} contains a duplicate BAM file role");
        }
    }
    Ok(indexed)
}

fn catalog_files_by_role(files: Vec<CatalogFile>) -> Result<BTreeMap<String, CatalogFile>> {
    let mut catalog = BTreeMap::new();
    for file in files {
        validate_sha256(&file.sha256, "catalog file")?;
        if catalog.insert(file.kind.clone(), file).is_some() {
            bail!("BAM catalog contains a duplicate file role");
        }
    }
    Ok(catalog)
}

fn required_role<'a>(
    files: &'a BTreeMap<String, AuthorityFile>,
    role: &str,
) -> Result<&'a AuthorityFile> {
    files.get(role).with_context(|| format!("BAM corpus lacks required role {role}"))
}

fn admit_materialized_file(
    study_root: &Path,
    file: &AuthorityFile,
    role: &str,
) -> Result<AdmittedBamCorpusFile> {
    let path = resolve_study_uri(study_root, &file.path, role)?;
    let path = governed_regular_file(study_root, &path, role)?;
    let size = std::fs::metadata(&path)
        .with_context(|| format!("inspect BAM corpus file {}", path.display()))?
        .len();
    if size != file.size_bytes {
        bail!("BAM corpus file size differs for role {role}");
    }
    let sha256 = bijux_dna_infra::hash_file_sha256(&path)
        .with_context(|| format!("hash BAM corpus file {}", path.display()))?;
    if sha256 != file.sha256 {
        bail!("BAM corpus file digest differs for role {role}");
    }
    Ok(AdmittedBamCorpusFile { path: path_string(&path)?, sha256, size_bytes: size })
}

fn validate_bam_pair(
    bam: &AdmittedBamCorpusFile,
    bam_index: &AdmittedBamCorpusFile,
    sample_id: &str,
) -> Result<()> {
    let expected_bam = format!("{sample_id}.sorted.bam");
    let expected_bai = format!("{expected_bam}.bai");
    if file_name(&bam.path, BAM_ROLE)? != expected_bam
        || file_name(&bam_index.path, BAI_ROLE)? != expected_bai
    {
        bail!("BAM and BAI names do not close over sample {sample_id}");
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

fn path_string(path: &Path) -> Result<String> {
    path.to_str().map(str::to_string).context("governed path is not valid UTF-8")
}
