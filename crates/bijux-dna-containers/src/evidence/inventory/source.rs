use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::evidence::{BuildContextIdentity, BuildEvidenceStatus, ContainerBuildEvidence};

pub const RAW_INVENTORY_SCHEMA_V1: &str = "bijux.container.image_inventory_scan.v1";
pub const RAW_INVENTORY_SCHEMA_V2: &str = "bijux.container.image_inventory_scan.v2";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageInventoryDerivationRequest {
    pub repository_root: PathBuf,
    pub study_root: PathBuf,
    pub evidence_roots: Vec<ImageInventoryEvidenceRoots>,
    pub run_evidence_root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageInventoryEvidenceRoots {
    pub image_root: PathBuf,
    pub build_evidence_root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageInventoryEvidenceRootIdentity {
    pub image_root: String,
    pub build_evidence_root: String,
}

pub(super) fn validate_evidence_roots(request: &ImageInventoryDerivationRequest) -> Result<()> {
    if request.evidence_roots.is_empty() {
        bail!("image inventory requires at least one paired image and build-evidence root");
    }
    let study_root = request
        .study_root
        .canonicalize()
        .with_context(|| format!("canonicalize study root {}", request.study_root.display()))?;
    let mut evidence_root_pairs = BTreeSet::new();
    let mut build_evidence_roots = BTreeSet::new();
    for roots in &request.evidence_roots {
        let image_root = roots
            .image_root
            .canonicalize()
            .with_context(|| format!("canonicalize image root {}", roots.image_root.display()))?;
        let build_evidence_root = roots.build_evidence_root.canonicalize().with_context(|| {
            format!("canonicalize build-evidence root {}", roots.build_evidence_root.display())
        })?;
        if !image_root.starts_with(&study_root) || !build_evidence_root.starts_with(&study_root) {
            bail!("image inventory evidence roots must be contained by the governed study root");
        }
        if image_root == build_evidence_root {
            bail!("image and build-evidence roots must be distinct");
        }
        if !evidence_root_pairs.insert((image_root, build_evidence_root.clone())) {
            bail!("image inventory contains a duplicate evidence-root pair");
        }
        if !build_evidence_roots.insert(build_evidence_root) {
            bail!("image inventory contains a duplicate build-evidence root");
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageInventoryScan {
    #[schemars(schema_with = "image_inventory_scan_schema_version")]
    pub schema_version: String,
    pub repository_commit: String,
    pub catalog_sha256: String,
    #[serde(default)]
    #[schemars(required)]
    pub evidence_roots: Vec<ImageInventoryEvidenceRootIdentity>,
    pub images: Vec<ImageInventoryScannedImage>,
    pub staged_images: Vec<ImageInventoryScannedImage>,
    pub build_evidence: Vec<ImageInventorySourceFile>,
    pub run_references: BTreeMap<String, Vec<String>>,
}

fn image_inventory_scan_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": RAW_INVENTORY_SCHEMA_V2,
    })
}

pub(super) fn evidence_root_identities(
    request: &ImageInventoryDerivationRequest,
) -> Result<Vec<ImageInventoryEvidenceRootIdentity>> {
    let study_root = request
        .study_root
        .canonicalize()
        .with_context(|| format!("canonicalize study root {}", request.study_root.display()))?;
    request
        .evidence_roots
        .iter()
        .map(|roots| {
            Ok(ImageInventoryEvidenceRootIdentity {
                image_root: portable_study_root(&roots.image_root, &study_root)?,
                build_evidence_root: portable_study_root(&roots.build_evidence_root, &study_root)?,
            })
        })
        .collect()
}

fn portable_study_root(path: &Path, study_root: &Path) -> Result<String> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("canonicalize evidence root {}", path.display()))?;
    let relative = canonical.strip_prefix(study_root).with_context(|| {
        format!("evidence root {} is outside governed study root", canonical.display())
    })?;
    Ok(format!("study://{}", relative.to_string_lossy()))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageInventoryScannedImage {
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageInventorySourceFile {
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
    #[serde(default)]
    definition: Option<BuildDefinitionIdentity>,
    #[serde(default)]
    definition_sha256: Option<String>,
    #[serde(default)]
    build_context: Option<Vec<BuildContextIdentity>>,
    #[serde(default)]
    image: Option<BuildImageIdentity>,
    #[serde(default)]
    image_path: Option<String>,
    #[serde(default)]
    image_sha256: Option<String>,
    #[serde(default)]
    image_size_bytes: Option<u64>,
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

struct NormalizedBuildIdentity {
    definition: BuildDefinitionIdentity,
    image: BuildImageIdentity,
}

impl BuildEvidenceDocument {
    fn normalize_identity(&mut self) -> Result<NormalizedBuildIdentity> {
        match (
            self.definition.take(),
            self.image.take(),
            self.definition_sha256.take(),
            self.image_path.take(),
            self.image_sha256.take(),
            self.image_size_bytes.take(),
        ) {
            (Some(definition), Some(image), None, None, None, None) => {
                Ok(NormalizedBuildIdentity { definition, image })
            }
            (None, None, Some(definition_sha256), image_path, Some(image_sha256), Some(size)) => {
                Ok(NormalizedBuildIdentity {
                    definition: BuildDefinitionIdentity { path: None, sha256: definition_sha256 },
                    image: BuildImageIdentity {
                        path: image_path,
                        sha256: image_sha256,
                        size_bytes: size,
                    },
                })
            }
            _ => bail!(
                "build evidence must contain exactly one complete nested or flattened identity"
            ),
        }
    }
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
    pub identities: Vec<ImageInventorySourceFile>,
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
) -> Result<Vec<(PathBuf, ImageInventoryScannedImage)>> {
    let mut images = Vec::new();
    let mut scanned_roots = BTreeSet::new();
    for roots in &request.evidence_roots {
        let image_root = roots
            .image_root
            .canonicalize()
            .with_context(|| format!("canonicalize image root {}", roots.image_root.display()))?;
        if !scanned_roots.insert(image_root.clone()) {
            continue;
        }
        for path in regular_files_with_suffix(&roots.image_root, ".sif")? {
            let metadata = fs::metadata(&path)
                .with_context(|| format!("inspect container image {}", path.display()))?;
            let sha256 = bijux_dna_infra::hash_file_sha256(&path)
                .with_context(|| format!("hash container image {}", path.display()))?;
            images.push((
                path.clone(),
                ImageInventoryScannedImage {
                    path: portable_evidence_path(&path, request)?,
                    sha256,
                    size_bytes: metadata.len(),
                },
            ));
        }
    }
    images.sort_by(|left, right| left.1.path.cmp(&right.1.path));
    let mut digests = BTreeSet::new();
    if let Some(duplicate) = images
        .iter()
        .map(|(_, image)| image.sha256.as_str())
        .find(|digest| !digests.insert((*digest).to_string()))
    {
        bail!("image inventory source roots repeat image digest {duplicate}");
    }
    Ok(images)
}

pub(super) fn scan_staged_images(
    request: &ImageInventoryDerivationRequest,
) -> Result<Vec<ImageInventoryScannedImage>> {
    let mut images = Vec::new();
    let mut scanned_roots = BTreeSet::new();
    for roots in &request.evidence_roots {
        let image_root = roots
            .image_root
            .canonicalize()
            .with_context(|| format!("canonicalize image root {}", roots.image_root.display()))?;
        if !scanned_roots.insert(image_root.clone()) {
            continue;
        }
        let root = roots.image_root.join(".building");
        if !root.is_dir() {
            continue;
        }
        for path in regular_files_with_suffix(&root, ".sif")? {
            let metadata = fs::metadata(&path)
                .with_context(|| format!("inspect staged image {}", path.display()))?;
            images.push(ImageInventoryScannedImage {
                path: portable_evidence_path(&path, request)?,
                sha256: bijux_dna_infra::hash_file_sha256(&path)
                    .with_context(|| format!("hash staged image {}", path.display()))?,
                size_bytes: metadata.len(),
            });
        }
    }
    images.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(images)
}

pub(super) fn load_build_evidence(
    request: &ImageInventoryDerivationRequest,
) -> Result<LoadedBuildEvidence> {
    let mut by_digest = BTreeMap::<String, Vec<IndexedBuildEvidence>>::new();
    let mut identities = Vec::new();
    for roots in &request.evidence_roots {
        for path in campaign_build_evidence_files(&roots.build_evidence_root)? {
            let bytes = fs::read(&path)
                .with_context(|| format!("read container build evidence {}", path.display()))?;
            let mut document: BuildEvidenceDocument = serde_json::from_slice(&bytes)
                .with_context(|| format!("parse container build evidence {}", path.display()))?;
            if document.schema_version != "bijux.container.build_evidence.v1" {
                bail!("unsupported container build evidence schema in {}", path.display());
            }
            let identity = document.normalize_identity().with_context(|| {
                format!("normalize container build evidence {}", path.display())
            })?;
            validate_sha256(&identity.definition.sha256, "build-evidence definition")?;
            validate_sha256(&identity.image.sha256, "build-evidence image")?;
            if request.evidence_roots.len() > 1 && identity.image.path.is_none() {
                bail!(
                    "composite image inventory requires path-bound build evidence in {}",
                    path.display()
                );
            }
            if let Some(image_path) = identity.image.path.as_deref() {
                validate_build_image_root(
                    image_path,
                    &identity.image.sha256,
                    identity.image.size_bytes,
                    roots,
                    request,
                )
                .with_context(|| {
                    format!("validate build-evidence image authority in {}", path.display())
                })?;
            }
            let image_name = identity
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
                definition_path: identity.definition.path,
                image_name,
                image_size_bytes: identity.image.size_bytes,
                evidence: ContainerBuildEvidence {
                    status: document.status,
                    definition_sha256: identity.definition.sha256,
                    repository_commit: document.repository_commit,
                    build_context: document.build_context,
                },
            };
            by_digest.entry(identity.image.sha256).or_default().push(indexed);
            identities.push(ImageInventorySourceFile {
                path: portable_evidence_path(&path, request)?,
                sha256: bijux_dna_infra::hash_bytes_sha256(&bytes),
            });
        }
    }
    identities.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(LoadedBuildEvidence { by_digest, identities })
}

fn validate_build_image_root(
    value: &str,
    expected_sha256: &str,
    expected_size_bytes: u64,
    roots: &ImageInventoryEvidenceRoots,
    request: &ImageInventoryDerivationRequest,
) -> Result<()> {
    let path = if let Some(relative) = value.strip_prefix("study://") {
        request.study_root.join(relative)
    } else if let Some(relative) = value.strip_prefix("repository://") {
        request.repository_root.join(relative)
    } else {
        let path = PathBuf::from(value);
        if !path.is_absolute() {
            bail!("build-evidence image path must be absolute or use a governed URI scheme");
        }
        path
    };
    let image_root = roots.image_root.canonicalize().with_context(|| {
        format!("canonicalize paired image root {}", roots.image_root.display())
    })?;
    let study_root = request
        .study_root
        .canonicalize()
        .with_context(|| format!("canonicalize study root {}", request.study_root.display()))?;
    if !path
        .try_exists()
        .with_context(|| format!("inspect build-evidence image authority {}", path.display()))?
    {
        // Activation and governed storage reconciliation may retire the original campaign link
        // while preserving the same immutable image in an active or composite roster. Candidate
        // admission still requires an exact digest, size, filename, tool, version, and recipe
        // match, and composite scans reject repeated image digests across their image roots.
        let relative = path.strip_prefix(&request.study_root).with_context(|| {
            format!("retired build-evidence image path is outside {}", request.study_root.display())
        })?;
        if relative.components().any(|component| !matches!(component, Component::Normal(_))) {
            bail!("retired build-evidence image path is not a normalized STUDY path");
        }
        let mut existing_ancestor =
            path.parent().context("build-evidence image path has no parent")?;
        while !existing_ancestor.try_exists().with_context(|| {
            format!("inspect retired build-evidence ancestor {}", existing_ancestor.display())
        })? {
            existing_ancestor = existing_ancestor
                .parent()
                .context("retired build-evidence image path has no existing ancestor")?;
        }
        let existing_ancestor = existing_ancestor.canonicalize().with_context(|| {
            format!(
                "canonicalize retired build-evidence image ancestor {}",
                existing_ancestor.display()
            )
        })?;
        if !existing_ancestor.starts_with(&study_root) {
            bail!("retired build-evidence image path is outside the governed study root");
        }
        return Ok(());
    }
    let image = path
        .canonicalize()
        .with_context(|| format!("canonicalize build-evidence image {}", path.display()))?;
    if image.starts_with(&image_root) {
        return Ok(());
    }
    if !image.starts_with(&study_root) {
        bail!("build-evidence image is outside the governed study root");
    }
    let file_name = image.file_name().context("build-evidence image has no filename")?;
    let paired_image = image_root.join(file_name);
    let paired_image = paired_image.canonicalize().with_context(|| {
        format!(
            "build-evidence image is outside its paired image root and no activated image exists at {}",
            paired_image.display()
        )
    })?;
    if !paired_image.starts_with(&image_root) || !paired_image.is_file() {
        bail!("activated build-evidence image is outside its paired image root");
    }
    let metadata = fs::metadata(&paired_image)
        .with_context(|| format!("inspect activated image {}", paired_image.display()))?;
    if metadata.len() != expected_size_bytes
        || bijux_dna_infra::hash_file_sha256(&paired_image)
            .with_context(|| format!("hash activated image {}", paired_image.display()))?
            != expected_sha256
    {
        bail!("activated image does not match build-evidence identity");
    }
    Ok(())
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
            .push(portable_evidence_path(&lock_path, request)?);
    }
    for paths in references.values_mut() {
        paths.sort();
        paths.dedup();
    }
    Ok(references)
}

pub(super) fn portable_evidence_path(
    path: &Path,
    request: &ImageInventoryDerivationRequest,
) -> Result<String> {
    for (scheme, root) in
        [("study", request.study_root.as_path()), ("repository", request.repository_root.as_path())]
    {
        if let Ok(relative) = path.strip_prefix(root) {
            return Ok(format!("{scheme}://{}", relative.to_string_lossy()));
        }
    }
    bail!("evidence path {} is outside governed study and repository roots", path.display())
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

fn campaign_build_evidence_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = regular_files_with_suffix(root, ".build.json")?;
    for entry in fs::read_dir(root)
        .with_context(|| format!("read build-evidence root {}", root.display()))?
    {
        let entry =
            entry.with_context(|| format!("read build-evidence entry under {}", root.display()))?;
        if entry.file_type().context("inspect build-evidence entry")?.is_dir()
            && entry.file_name().to_string_lossy().starts_with("selected-")
        {
            paths.extend(regular_files_with_suffix(&entry.path(), ".build.json")?);
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
