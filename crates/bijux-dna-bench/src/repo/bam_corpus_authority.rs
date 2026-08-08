//! Owner: bijux-dna-bench
//! Admission of governed BAM corpora into benchmark execution.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_benchmark_data_index, AdmittedBamCorpus, AdmittedBamCorpusFile, BenchmarkDataIndex,
    BenchmarkDataIndexFile, BenchmarkDataIndexRecord, BenchmarkEligibility, DataIntegrityStatus,
    GOVERNED_DATA_CATALOG_SCHEMA_V1, GOVERNED_DATA_CATALOG_SCHEMA_V2,
};
use serde::Deserialize;

const BAM_ROLE: &str = "coordinate-sorted-deduplicated-bam";
const BAI_ROLE: &str = "bam-index";
const MAX_AUTHORITY_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedBamGenotypingCorpus {
    pub bam: AdmittedBamCorpus,
    pub reference: AdmittedBamCorpusFile,
    pub reference_index: AdmittedBamCorpusFile,
    pub reference_catalog: AdmittedBamCorpusFile,
    pub known_sites: AdmittedBamCorpusFile,
    pub known_sites_index: AdmittedBamCorpusFile,
    pub candidate_regions: AdmittedBamCorpusFile,
    pub known_sites_catalog: AdmittedBamCorpusFile,
    pub constrained_alleles: AdmittedBamCorpusFile,
    pub genotyping_inputs_catalog: AdmittedBamCorpusFile,
    pub bam_input_bytes: u64,
    pub reference_input_bytes: u64,
    pub database_input_bytes: u64,
    pub input_bytes: u64,
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

#[derive(Debug, Deserialize)]
struct GenericDataCatalog {
    schema_version: String,
    id: String,
    files: Vec<GenericCatalogFile>,
}

#[derive(Debug, Deserialize)]
struct GenericCatalogFile {
    path: Option<String>,
    kind: Option<String>,
    sha256: Option<String>,
    size_bytes: Option<u64>,
    governed_path: Option<String>,
    governed_sha256: Option<String>,
    governed_size_bytes: Option<u64>,
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
    let index: BenchmarkDataIndex = serde_json::from_slice(&data_index_bytes)
        .with_context(|| format!("parse data index {}", data_index_path.display()))?;
    validate_benchmark_data_index(&index).context("validate benchmark data index")?;
    let mut records = index.records.into_iter().filter(|record| record.id == corpus_id);
    let record = records.next().with_context(|| format!("data index lacks corpus {corpus_id}"))?;
    if records.next().is_some() {
        bail!("data index corpus identity is not unique: {corpus_id}");
    }
    if record.benchmark_eligibility != BenchmarkEligibility::Eligible
        || record.integrity_status != DataIntegrityStatus::Verified
    {
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
    if !matches!(
        catalog.schema_version.as_str(),
        GOVERNED_DATA_CATALOG_SCHEMA_V1 | GOVERNED_DATA_CATALOG_SCHEMA_V2
    ) || catalog.id != corpus_id
    {
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

/// Admit the complete reference and candidate-site closure used by BAM genotyping.
///
/// # Errors
/// Returns an error when any governed index record, catalog, materialized file, size, or digest
/// differs from the exact run-2001 input contract.
pub fn admit_bam_genotyping_corpus(
    study_root: &Path,
    data_index_path: &Path,
) -> Result<AdmittedBamGenotypingCorpus> {
    let bam =
        admit_bam_benchmark_corpus(study_root, data_index_path, "ena-prjeb22390-err2112767-bam")?;
    let study_root = std::fs::canonicalize(study_root).context("resolve genotyping STUDY root")?;
    let data_index_path = governed_regular_file(&study_root, data_index_path, "data index")?;
    let index: BenchmarkDataIndex =
        serde_json::from_slice(&read_authority(&data_index_path, "data index")?)
            .context("parse genotyping data index")?;
    validate_benchmark_data_index(&index).context("validate genotyping data index")?;

    let reference_record = unique_record(&index, "equcab3-gcf-002863925.1")?;
    let known_sites_record = unique_record(&index, "ensembl-113-equcab3-refseq-known-sites-10mb")?;
    let alleles_record =
        unique_record(&index, "ensembl-113-equcab3-refseq-genotyping-inputs-10mb")?;
    let reference_catalog = admit_record_catalog(&study_root, reference_record)?;
    let known_sites_catalog = admit_record_catalog(&study_root, known_sites_record)?;
    let genotyping_inputs_catalog = admit_record_catalog(&study_root, alleles_record)?;
    let reference = admit_record_role(&study_root, reference_record, "catalog.files.0")?;
    let reference_index = admit_record_role(&study_root, reference_record, "catalog.files.1")?;
    let known_sites =
        admit_record_role(&study_root, known_sites_record, "normalized-known-sites-vcf")?;
    let known_sites_index =
        admit_record_role(&study_root, known_sites_record, "known-sites-vcf-index")?;
    let candidate_regions =
        admit_record_role(&study_root, known_sites_record, "zero-based-candidate-regions")?;
    let constrained_alleles =
        admit_record_role(&study_root, alleles_record, "bcftools-constrained-calling-alleles")?;
    let bam_input_bytes = checked_sum(&[bam.bam.size_bytes, bam.bam_index.size_bytes])?;
    let reference_input_bytes = checked_sum(&[reference.size_bytes, reference_index.size_bytes])?;
    let database_input_bytes = checked_sum(&[
        known_sites.size_bytes,
        known_sites_index.size_bytes,
        candidate_regions.size_bytes,
        constrained_alleles.size_bytes,
    ])?;
    let input_bytes = checked_sum(&[bam_input_bytes, reference_input_bytes, database_input_bytes])?;

    Ok(AdmittedBamGenotypingCorpus {
        bam,
        reference,
        reference_index,
        reference_catalog,
        known_sites,
        known_sites_index,
        candidate_regions,
        known_sites_catalog,
        constrained_alleles,
        genotyping_inputs_catalog,
        bam_input_bytes,
        reference_input_bytes,
        database_input_bytes,
        input_bytes,
    })
}

fn unique_record<'a>(
    index: &'a BenchmarkDataIndex,
    id: &str,
) -> Result<&'a BenchmarkDataIndexRecord> {
    let mut matches = index.records.iter().filter(|record| record.id == id);
    let record = matches.next().with_context(|| format!("data index lacks record {id}"))?;
    if matches.next().is_some() {
        bail!("data index record identity is not unique: {id}");
    }
    if record.benchmark_eligibility != BenchmarkEligibility::Eligible
        || record.integrity_status != DataIntegrityStatus::Verified
    {
        bail!("genotyping input is not eligible with verified integrity: {id}");
    }
    Ok(record)
}

fn admit_record_catalog(
    study_root: &Path,
    record: &BenchmarkDataIndexRecord,
) -> Result<AdmittedBamCorpusFile> {
    validate_sha256(&record.catalog_sha256, "catalog")?;
    let path = resolve_study_uri(study_root, &record.catalog_path, "catalog")?;
    let path = governed_regular_file(study_root, &path, "catalog")?;
    let bytes = read_authority(&path, "catalog")?;
    let sha256 = bijux_dna_infra::hash_bytes_sha256(&bytes);
    if sha256 != record.catalog_sha256 {
        bail!("genotyping catalog digest differs from the data index: {}", record.id);
    }
    let catalog: GenericDataCatalog = serde_json::from_slice(&bytes).context("parse catalog")?;
    if !matches!(
        catalog.schema_version.as_str(),
        GOVERNED_DATA_CATALOG_SCHEMA_V1 | GOVERNED_DATA_CATALOG_SCHEMA_V2
    ) || catalog.id != record.id
    {
        bail!("genotyping catalog identity is unsupported: {}", record.id);
    }
    validate_record_catalog_closure(study_root, record, &catalog.files)?;
    Ok(AdmittedBamCorpusFile {
        path: path_string(&path)?,
        sha256,
        size_bytes: u64::try_from(bytes.len()).context("catalog size does not fit u64")?,
    })
}

fn validate_record_catalog_closure(
    study_root: &Path,
    record: &BenchmarkDataIndexRecord,
    catalog_files: &[GenericCatalogFile],
) -> Result<()> {
    if record.files.len() != catalog_files.len() {
        bail!("genotyping catalog and data index file closures differ: {}", record.id);
    }
    let mut matched = BTreeSet::new();
    for index_file in &record.files {
        validate_sha256(&index_file.sha256, "data index file")?;
        let index_path = governed_regular_file(
            study_root,
            &resolve_study_uri(study_root, &index_file.path, &index_file.role)?,
            &index_file.role,
        )?;
        let mut candidates = catalog_files.iter().enumerate().filter_map(|(position, file)| {
            catalog_file_identity(file).ok().and_then(|(path, sha256, size_bytes)| {
                let path = resolve_catalog_path(study_root, path, &index_file.role).ok()?;
                (path == index_path
                    && sha256 == index_file.sha256
                    && size_bytes == index_file.size_bytes
                    && file.kind.as_deref().is_none_or(|kind| kind == index_file.role))
                .then_some(position)
            })
        });
        let position = candidates.next().with_context(|| {
            format!("genotyping catalog and data index disagree for role {}", index_file.role)
        })?;
        if candidates.next().is_some() || !matched.insert(position) {
            bail!("genotyping catalog file identity is ambiguous for role {}", index_file.role);
        }
    }
    if matched.len() != catalog_files.len() {
        bail!("genotyping catalog contains unmatched file identities: {}", record.id);
    }
    Ok(())
}

fn catalog_file_identity(file: &GenericCatalogFile) -> Result<(&str, &str, u64)> {
    match (
        file.path.as_deref(),
        file.sha256.as_deref(),
        file.size_bytes,
        file.governed_path.as_deref(),
        file.governed_sha256.as_deref(),
        file.governed_size_bytes,
    ) {
        (Some(path), Some(sha256), Some(size_bytes), None, None, None) => {
            validate_sha256(sha256, "catalog file")?;
            Ok((path, sha256, size_bytes))
        }
        (None, None, None, Some(path), Some(sha256), Some(size_bytes)) => {
            validate_sha256(sha256, "catalog file")?;
            Ok((path, sha256, size_bytes))
        }
        _ => bail!("genotyping catalog file identity is incomplete or ambiguous"),
    }
}

fn resolve_catalog_path(study_root: &Path, value: &str, label: &str) -> Result<PathBuf> {
    let path = if value.starts_with("study://") {
        resolve_study_uri(study_root, value, label)?
    } else {
        let path = Path::new(value);
        if !path.is_absolute() {
            bail!("catalog file path is neither absolute nor a STUDY URI: {value}");
        }
        path.to_path_buf()
    };
    governed_regular_file(study_root, &path, label)
}

fn admit_record_role(
    study_root: &Path,
    record: &BenchmarkDataIndexRecord,
    role: &str,
) -> Result<AdmittedBamCorpusFile> {
    let mut matches = record.files.iter().filter(|file| file.role == role);
    let file = matches.next().with_context(|| format!("{} lacks role {role}", record.id))?;
    if matches.next().is_some() {
        bail!("{} contains duplicate role {role}", record.id);
    }
    admit_materialized_file(study_root, file, role)
}

fn checked_sum(values: &[u64]) -> Result<u64> {
    values
        .iter()
        .try_fold(0_u64, |sum, value| sum.checked_add(*value))
        .context("genotyping input size overflow")
}

fn files_by_role(
    files: Vec<BenchmarkDataIndexFile>,
    authority: &str,
) -> Result<BTreeMap<String, BenchmarkDataIndexFile>> {
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
    files: &'a BTreeMap<String, BenchmarkDataIndexFile>,
    role: &str,
) -> Result<&'a BenchmarkDataIndexFile> {
    files.get(role).with_context(|| format!("BAM corpus lacks required role {role}"))
}

fn admit_materialized_file(
    study_root: &Path,
    file: &BenchmarkDataIndexFile,
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
