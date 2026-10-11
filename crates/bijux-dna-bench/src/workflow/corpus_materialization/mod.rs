//! Owner: bijux-dna-bench
//! Governed FASTQ corpus materialization and atomic admission workflow.

use std::fs;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

use anyhow::{bail, Context, Result};
pub use bijux_dna_bench_model::CorpusMaterializationPublication;
use bijux_dna_bench_model::{
    validate_corpus_acquisition_plan, validate_governed_data_catalog, CorpusAcquisitionPlan,
    GovernedDataCatalog, BENCHMARK_CORPUS_MATERIALIZATION_PUBLICATION_SCHEMA_V1,
};
use bijux_dna_db_ena::{
    download_verified_files, verify_existing_files, VerifiedDownloadDisposition,
    VerifiedDownloadRequest,
};
use chrono::{DateTime, FixedOffset};

mod catalog;
mod fastq;

const MAX_ACQUISITION_PLAN_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorpusMaterializationRequest {
    pub plan_path: PathBuf,
    pub data_root: PathBuf,
    pub repository_commit: String,
    pub maximum_parallelism: usize,
    pub retries: usize,
    pub timeout_seconds: u64,
    pub admitted_at_utc: String,
}

#[derive(Debug, Clone)]
struct MaterializationTarget {
    sample_index: usize,
    file_index: usize,
    path: PathBuf,
}

/// Materialize, validate, and atomically admit one checksum-pinned real FASTQ corpus.
///
/// Existing FASTQ bytes are re-hashed by the ENA transfer owner. A catalog is published only
/// after every compressed stream and FASTQ record passes and all files are made read-only.
///
/// # Errors
/// Returns an error for unsafe paths, invalid plans, transfer drift, malformed FASTQ, existing
/// catalog authority, or publication failure.
pub fn materialize_fastq_corpus(
    request: &CorpusMaterializationRequest,
) -> Result<CorpusMaterializationPublication> {
    validate_request(request)?;
    let data_root = canonical_directory(&request.data_root, "governed data root")?;
    let plan_path = canonical_file_under(&request.plan_path, &data_root, "acquisition plan")?;
    let bytes = bijux_dna_infra::read_to_end_bounded(&plan_path, MAX_ACQUISITION_PLAN_BYTES)
        .with_context(|| format!("read acquisition plan {}", plan_path.display()))?;
    let plan: CorpusAcquisitionPlan = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse acquisition plan {}", plan_path.display()))?;
    validate_corpus_acquisition_plan(&plan).context("validate corpus acquisition plan")?;

    let lock_path = data_root
        .join("evidence/materialization-locks")
        .join(format!("{}.lock", plan.portfolio_id));
    let _lock = bijux_dna_infra::FileLock::acquire(&lock_path, Duration::from_secs(30))
        .context("acquire corpus materialization lock")?;
    let targets = materialization_targets(&data_root, &plan);
    let catalog_path =
        catalog_snapshot_path(&data_root, &request.repository_commit, &plan.portfolio_id);
    if let Some(catalog) = load_existing_catalog(&catalog_path)? {
        return revalidate_existing_catalog(
            request,
            &plan,
            &plan_path,
            &catalog_path,
            catalog,
            &targets,
        );
    }
    let download_requests = download_requests(request, &plan, &targets)?;
    let transfers = download_verified_files(&download_requests, request.maximum_parallelism)
        .context("transfer checksum-pinned corpus files")?;
    ensure_transfer_order(&targets, &transfers)?;
    let fastq_paths = targets.iter().map(|target| target.path.clone()).collect::<Vec<_>>();
    let fastq = fastq::validate_fastq_files(&fastq_paths, request.maximum_parallelism)?;
    for path in &fastq_paths {
        make_read_only(path)?;
    }

    let relative_plan = plan_path
        .strip_prefix(&data_root)
        .context("acquisition plan is outside governed data root")?;
    let source_record = format!("study://data/{}", portable_relative_path(relative_plan)?);
    let source_record_sha256 =
        bijux_dna_infra::hash_file_sha256(&plan_path).context("hash acquisition plan")?;
    let catalog = catalog::build_catalog(
        &plan,
        &targets,
        &transfers,
        &fastq,
        source_record,
        source_record_sha256,
        request.admitted_at_utc.clone(),
    );
    validate_governed_data_catalog(&catalog).context("validate governed data catalog")?;
    bijux_dna_infra::atomic_create_json(&catalog_path, &catalog)
        .with_context(|| format!("publish admitted data catalog {}", catalog_path.display()))?;
    let persisted: GovernedDataCatalog = serde_json::from_slice(
        &bijux_dna_infra::read_to_end_bounded(&catalog_path, MAX_ACQUISITION_PLAN_BYTES)?,
    )?;
    if persisted != catalog {
        bail!("persisted governed data catalog differs from admitted catalog");
    }
    let catalog_sha256 =
        bijux_dna_infra::hash_file_sha256(&catalog_path).context("hash admitted catalog")?;
    let downloaded_files = count_disposition(&transfers, VerifiedDownloadDisposition::Downloaded)?;
    let verified_existing_files =
        count_disposition(&transfers, VerifiedDownloadDisposition::VerifiedExisting)?;
    Ok(CorpusMaterializationPublication {
        schema_version: BENCHMARK_CORPUS_MATERIALIZATION_PUBLICATION_SCHEMA_V1.to_string(),
        catalog_path,
        catalog_sha256,
        downloaded_files,
        verified_existing_files,
        catalog,
    })
}

fn load_existing_catalog(path: &Path) -> Result<Option<GovernedDataCatalog>> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(error)
                .with_context(|| format!("inspect admitted catalog {}", path.display()))
        }
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("admitted data catalog is not a regular non-symlink file: {}", path.display());
    }
    let canonical = path
        .canonicalize()
        .with_context(|| format!("canonicalize admitted catalog {}", path.display()))?;
    if canonical != path {
        bail!("admitted data catalog path is not canonical: {}", path.display());
    }
    let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_ACQUISITION_PLAN_BYTES)
        .with_context(|| format!("read admitted catalog {}", path.display()))?;
    let catalog = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse admitted catalog {}", path.display()))?;
    Ok(Some(catalog))
}

fn revalidate_existing_catalog(
    request: &CorpusMaterializationRequest,
    plan: &CorpusAcquisitionPlan,
    plan_path: &Path,
    catalog_path: &Path,
    catalog: GovernedDataCatalog,
    targets: &[MaterializationTarget],
) -> Result<CorpusMaterializationPublication> {
    validate_governed_data_catalog(&catalog).context("validate existing governed data catalog")?;
    for target in targets {
        let metadata = fs::symlink_metadata(&target.path)
            .with_context(|| format!("inspect admitted FASTQ {}", target.path.display()))?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            bail!("admitted FASTQ is not a regular non-symlink file: {}", target.path.display());
        }
        let canonical = target
            .path
            .canonicalize()
            .with_context(|| format!("canonicalize admitted FASTQ {}", target.path.display()))?;
        if canonical != target.path {
            bail!("admitted FASTQ path is not canonical: {}", target.path.display());
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            if metadata.permissions().mode() & 0o222 != 0 {
                bail!("admitted FASTQ remains writable: {}", target.path.display());
            }
        }
    }
    let requests = download_requests(request, plan, targets)?;
    let transfers = verify_existing_files(&requests, request.maximum_parallelism)
        .context("reverify checksum-pinned corpus files")?;
    ensure_transfer_order(targets, &transfers)?;
    let paths = targets.iter().map(|target| target.path.clone()).collect::<Vec<_>>();
    let fastq = fastq::validate_fastq_files(&paths, request.maximum_parallelism)?;
    let relative_plan = plan_path
        .strip_prefix(&request.data_root)
        .context("acquisition plan is outside governed data root")?;
    let source_record = format!("study://data/{}", portable_relative_path(relative_plan)?);
    let source_record_sha256 =
        bijux_dna_infra::hash_file_sha256(plan_path).context("hash acquisition plan")?;
    catalog::validate_existing_catalog(
        &catalog,
        plan,
        targets,
        &fastq,
        &transfers,
        &source_record,
        &source_record_sha256,
    )?;
    let catalog_sha256 =
        bijux_dna_infra::hash_file_sha256(catalog_path).context("hash admitted catalog")?;
    Ok(CorpusMaterializationPublication {
        schema_version: BENCHMARK_CORPUS_MATERIALIZATION_PUBLICATION_SCHEMA_V1.to_string(),
        catalog_path: catalog_path.to_path_buf(),
        catalog_sha256,
        downloaded_files: 0,
        verified_existing_files: count_disposition(
            &transfers,
            VerifiedDownloadDisposition::VerifiedExisting,
        )?,
        catalog,
    })
}

fn validate_request(request: &CorpusMaterializationRequest) -> Result<()> {
    if request.repository_commit.len() != 40
        || !request
            .repository_commit
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("repository commit must be a lowercase 40-character Git object id");
    }
    if request.maximum_parallelism == 0 || request.maximum_parallelism > 64 {
        bail!("corpus materialization parallelism must be between 1 and 64");
    }
    if request.retries > 10 {
        bail!("corpus materialization retries must not exceed 10");
    }
    if request.timeout_seconds == 0 || request.timeout_seconds > 3600 {
        bail!("corpus materialization timeout must be between 1 and 3600 seconds");
    }
    if request.data_root == request.plan_path {
        bail!("acquisition plan cannot be the governed data root");
    }
    let admitted_at = DateTime::<FixedOffset>::parse_from_rfc3339(&request.admitted_at_utc)
        .context("corpus admission timestamp is not RFC 3339")?;
    if admitted_at.offset().local_minus_utc() != 0 {
        bail!("corpus admission timestamp must use UTC");
    }
    Ok(())
}

fn catalog_snapshot_path(data_root: &Path, repository_commit: &str, portfolio_id: &str) -> PathBuf {
    data_root.join("catalog/snapshots").join(repository_commit).join(format!("{portfolio_id}.json"))
}

fn materialization_targets(
    data_root: &Path,
    plan: &CorpusAcquisitionPlan,
) -> Vec<MaterializationTarget> {
    plan.samples
        .iter()
        .enumerate()
        .flat_map(|(sample_index, sample)| {
            sample.files.iter().enumerate().map(move |(file_index, file)| MaterializationTarget {
                sample_index,
                file_index,
                path: data_root
                    .join("corpora")
                    .join(&plan.portfolio_id)
                    .join("raw")
                    .join(&sample.accession)
                    .join(&file.name),
            })
        })
        .collect()
}

fn download_requests(
    request: &CorpusMaterializationRequest,
    plan: &CorpusAcquisitionPlan,
    targets: &[MaterializationTarget],
) -> Result<Vec<VerifiedDownloadRequest>> {
    targets
        .iter()
        .map(|target| {
            let source = &plan.samples[target.sample_index].files[target.file_index];
            Ok(VerifiedDownloadRequest {
                source_url: source.source_url.clone(),
                output: target.path.clone(),
                expected_size_bytes: source.size_bytes,
                expected_md5: source.source_md5.clone(),
                retries: request.retries,
                timeout_seconds: request.timeout_seconds,
            })
        })
        .collect()
}

fn ensure_transfer_order(
    targets: &[MaterializationTarget],
    transfers: &[bijux_dna_db_ena::VerifiedDownloadOutcome],
) -> Result<()> {
    if targets.len() != transfers.len()
        || targets.iter().zip(transfers).any(|(target, transfer)| target.path != transfer.output)
    {
        bail!("verified transfer outcomes differ from materialization target order");
    }
    Ok(())
}

fn canonical_directory(path: &Path, label: &str) -> Result<PathBuf> {
    let metadata = fs::symlink_metadata(path).with_context(|| format!("inspect {label}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("{label} is not a regular non-symlink directory");
    }
    let canonical = path.canonicalize().with_context(|| format!("canonicalize {label}"))?;
    if canonical != path {
        bail!("{label} path is not canonical");
    }
    Ok(canonical)
}

fn canonical_file_under(path: &Path, root: &Path, label: &str) -> Result<PathBuf> {
    let metadata = fs::symlink_metadata(path).with_context(|| format!("inspect {label}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("{label} is not a regular non-symlink file");
    }
    let canonical = path.canonicalize().with_context(|| format!("canonicalize {label}"))?;
    canonical.strip_prefix(root).with_context(|| format!("{label} escapes governed root"))?;
    Ok(canonical)
}

fn portable_relative_path(path: &Path) -> Result<String> {
    if path.components().any(|component| !matches!(component, Component::Normal(_))) {
        bail!("governed relative path contains a non-normal component");
    }
    Ok(path.to_string_lossy().replace('\\', "/"))
}

fn make_read_only(path: &Path) -> Result<()> {
    let mut permissions = fs::metadata(path)
        .with_context(|| format!("inspect admitted FASTQ {}", path.display()))?
        .permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        permissions.set_mode(0o444);
    }
    #[cfg(not(unix))]
    permissions.set_readonly(true);
    fs::set_permissions(path, permissions)
        .with_context(|| format!("make admitted FASTQ read-only {}", path.display()))
}

fn count_disposition(
    transfers: &[bijux_dna_db_ena::VerifiedDownloadOutcome],
    disposition: VerifiedDownloadDisposition,
) -> Result<u64> {
    u64::try_from(transfers.iter().filter(|value| value.disposition == disposition).count())
        .context("materialization disposition count overflows")
}
