use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::evidence::{BuildContextIdentity, BuildEvidenceStatus, ContainerBuildEvidence};

pub const RAW_INVENTORY_SCHEMA_V1: &str = "bijux.container.image_inventory_scan.v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageInventoryDerivationRequest {
    pub repository_root: PathBuf,
    pub study_root: PathBuf,
    pub image_root: PathBuf,
    pub build_evidence_root: PathBuf,
    pub run_evidence_root: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct RawInventoryIdentity {
    pub schema_version: &'static str,
    pub repository_commit: String,
    pub catalog_sha256: String,
    pub images: Vec<RawImageIdentity>,
    pub build_evidence: Vec<RawFileIdentity>,
    pub run_references: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct RawImageIdentity {
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct RawFileIdentity {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Deserialize)]
struct BuildEvidenceDocument {
    schema_version: String,
    tool: String,
    version: String,
    status: BuildEvidenceStatus,
    repository_commit: String,
    definition: BuildDefinitionIdentity,
    #[serde(default)]
    build_context: Option<Vec<BuildContextIdentity>>,
    image: BuildImageIdentity,
}

#[derive(Debug, Clone, Deserialize)]
struct BuildDefinitionIdentity {
    #[serde(default)]
    path: Option<String>,
    sha256: String,
}

#[derive(Debug, Clone, Deserialize)]
struct BuildImageIdentity {
    #[serde(default)]
    path: Option<String>,
    sha256: String,
    size_bytes: u64,
}

#[derive(Debug, Clone)]
pub(super) struct IndexedBuildEvidence {
    pub tool: String,
    pub version: String,
    pub definition_path: Option<String>,
    pub image_name: Option<String>,
    pub image_size_bytes: u64,
    pub evidence: ContainerBuildEvidence,
}

pub(super) struct LoadedBuildEvidence {
    pub by_digest: BTreeMap<String, Vec<IndexedBuildEvidence>>,
    pub identities: Vec<RawFileIdentity>,
}

#[derive(Debug, Deserialize)]
struct RunEvidenceLock {
    schema_version: String,
    image: RunImageIdentity,
}

#[derive(Debug, Deserialize)]
struct RunImageIdentity {
    sha256: String,
}

pub(super) fn scan_images(
    request: &ImageInventoryDerivationRequest,
) -> Result<Vec<(PathBuf, RawImageIdentity)>> {
    let mut images = Vec::new();
    for path in regular_files_with_suffix(&request.image_root, ".sif")? {
        let metadata = fs::metadata(&path)
            .with_context(|| format!("inspect container image {}", path.display()))?;
        let sha256 = bijux_dna_infra::hash_file_sha256(&path)
            .with_context(|| format!("hash container image {}", path.display()))?;
        images.push((
            path.clone(),
            RawImageIdentity {
                path: portable_study_path(&path, &request.study_root)?,
                sha256,
                size_bytes: metadata.len(),
            },
        ));
    }
    Ok(images)
}

pub(super) fn load_build_evidence(
    request: &ImageInventoryDerivationRequest,
) -> Result<LoadedBuildEvidence> {
    let mut by_digest = BTreeMap::<String, Vec<IndexedBuildEvidence>>::new();
    let mut identities = Vec::new();
    for path in regular_files_with_suffix(&request.build_evidence_root, ".build.json")? {
        let bytes = fs::read(&path)
            .with_context(|| format!("read container build evidence {}", path.display()))?;
        let document: BuildEvidenceDocument = serde_json::from_slice(&bytes)
            .with_context(|| format!("parse container build evidence {}", path.display()))?;
        if document.schema_version != "bijux.container.build_evidence.v1" {
            bail!("unsupported container build evidence schema in {}", path.display());
        }
        validate_sha256(&document.image.sha256, "build-evidence image")?;
        let image_name = document
            .image
            .path
            .as_deref()
            .map(Path::new)
            .map(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .context("build-evidence image path has no UTF-8 filename")
                    .map(str::to_string)
            })
            .transpose()?;
        let indexed = IndexedBuildEvidence {
            tool: document.tool,
            version: document.version,
            definition_path: document.definition.path,
            image_name,
            image_size_bytes: document.image.size_bytes,
            evidence: ContainerBuildEvidence {
                status: document.status,
                definition_sha256: document.definition.sha256,
                repository_commit: document.repository_commit,
                build_context: document.build_context,
            },
        };
        by_digest.entry(document.image.sha256).or_default().push(indexed);
        identities.push(RawFileIdentity {
            path: portable_study_path(&path, &request.study_root)?,
            sha256: bijux_dna_infra::hash_bytes_sha256(&bytes),
        });
    }
    Ok(LoadedBuildEvidence { by_digest, identities })
}

pub(super) fn load_run_references(
    request: &ImageInventoryDerivationRequest,
) -> Result<BTreeMap<String, Vec<String>>> {
    let mut references = BTreeMap::<String, Vec<String>>::new();
    for entry in fs::read_dir(&request.run_evidence_root).with_context(|| {
        format!("read container run evidence root {}", request.run_evidence_root.display())
    })? {
        let entry = entry.context("read container run evidence entry")?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !entry.file_type().context("inspect container run evidence entry")?.is_dir()
            || !name.starts_with("run-")
        {
            continue;
        }
        let lock_path = entry.path().join("evidence.lock.json");
        if !lock_path.is_file() {
            continue;
        }
        let bytes = fs::read(&lock_path)
            .with_context(|| format!("read container run evidence {}", lock_path.display()))?;
        let lock: RunEvidenceLock = serde_json::from_slice(&bytes)
            .with_context(|| format!("parse container run evidence {}", lock_path.display()))?;
        if lock.schema_version != "bijux.container.run_evidence_lock.v1" {
            bail!("unsupported container run evidence schema in {}", lock_path.display());
        }
        validate_sha256(&lock.image.sha256, "run image")?;
        references
            .entry(lock.image.sha256)
            .or_default()
            .push(portable_study_path(&lock_path, &request.study_root)?);
    }
    for paths in references.values_mut() {
        paths.sort();
        paths.dedup();
    }
    Ok(references)
}

pub(super) fn portable_study_path(path: &Path, study_root: &Path) -> Result<String> {
    let relative = path.strip_prefix(study_root).with_context(|| {
        format!(
            "execution evidence path {} is outside study root {}",
            path.display(),
            study_root.display()
        )
    })?;
    Ok(format!("study://{}", relative.to_string_lossy()))
}

fn regular_files_with_suffix(root: &Path, suffix: &str) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for entry in
        fs::read_dir(root).with_context(|| format!("read evidence root {}", root.display()))?
    {
        let entry =
            entry.with_context(|| format!("read evidence entry under {}", root.display()))?;
        if entry.file_type().context("inspect evidence entry")?.is_file()
            && entry.file_name().to_string_lossy().ends_with(suffix)
        {
            paths.push(entry.path());
        }
    }
    paths.sort();
    Ok(paths)
}

fn validate_sha256(value: &str, label: &str) -> Result<()> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("{label} identity is not a SHA-256 digest: {value}");
    }
    Ok(())
}

pub(super) fn referenced_digests(references: &BTreeMap<String, Vec<String>>) -> BTreeSet<&str> {
    references.keys().map(String::as_str).collect()
}
