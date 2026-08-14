//! Resolve one admitted inventory selection to an exact local Apptainer image.

use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};

use super::{
    validate_container_inventory_catalog, validate_image_inventory_publication,
    verify_external_artifact_admission, ContainerArtifactDelivery, ContainerCatalog,
    ContainerImageInventoryPublication, ContainerImageState, ExternalArtifactAdmissionStatus,
    ExternalArtifactKind, ImageCandidateDisposition,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentApptainerImage {
    pub tool_id: String,
    pub path: PathBuf,
    pub sha256: String,
    pub size_bytes: u64,
    pub repository_commit: String,
    pub catalog_sha256: String,
}

/// Resolve one image only after binding the inventory to its catalog and repository source lock.
///
/// # Errors
/// Returns an error when the inventory is stale, differs from the catalog, or cannot resolve the
/// requested tool to one current immutable image.
pub fn resolve_catalog_locked_apptainer_image(
    publication: &ContainerImageInventoryPublication,
    catalog_sha256: &str,
    catalog: &ContainerCatalog,
    repository_commit: &str,
    study_root: &Path,
    tool_id: &str,
) -> Result<CurrentApptainerImage> {
    validate_container_inventory_catalog(catalog_sha256, catalog, publication)
        .context("bind image inventory to the container catalog")?;
    if publication.repository_commit != repository_commit {
        bail!(
            "image inventory repository commit is stale: expected {repository_commit}, observed {}",
            publication.repository_commit
        );
    }
    let catalog_tool = catalog
        .tools
        .iter()
        .find(|tool| tool.id == tool_id)
        .with_context(|| format!("container catalog does not declare tool {tool_id}"))?;
    if !catalog_tool.is_redistributable_image() {
        bail!(
            "tool {tool_id} requires admitted {} execution authority",
            catalog_tool.artifact_delivery.as_str()
        );
    }
    resolve_current_apptainer_image(publication, study_root, tool_id)
}

/// Resolve one source-locked Apptainer artifact through its delivery-specific authority.
///
/// Repository-distributable images remain selected by the current image inventory. Licensed
/// external and private/site-delivered SIFs are selected only from the exact commit-addressed
/// admission record under STUDY and are reverified before use.
///
/// # Errors
/// Returns an error when source authorities drift, external access is blocked or unavailable, an
/// admitted artifact is not a SIF, or the selected bytes no longer match their admitted identity.
pub fn resolve_catalog_locked_apptainer_artifact(
    publication: &ContainerImageInventoryPublication,
    catalog_sha256: &str,
    catalog: &ContainerCatalog,
    repository_commit: &str,
    repository_root: &Path,
    study_root: &Path,
    tool_id: &str,
) -> Result<CurrentApptainerImage> {
    validate_container_inventory_catalog(catalog_sha256, catalog, publication)
        .context("bind image inventory to the container catalog")?;
    if publication.repository_commit != repository_commit {
        bail!(
            "image inventory repository commit is stale: expected {repository_commit}, observed {}",
            publication.repository_commit
        );
    }
    let catalog_tool = catalog
        .tools
        .iter()
        .find(|tool| tool.id == tool_id)
        .with_context(|| format!("container catalog does not declare tool {tool_id}"))?;
    if catalog_tool.is_redistributable_image() {
        return resolve_current_apptainer_image(publication, study_root, tool_id);
    }
    if !catalog_tool.permits_external_artifact() || !catalog_tool.external_artifact_supported {
        bail!("tool {tool_id} has no supported Apptainer artifact delivery authority");
    }

    let admission_path = study_root
        .join("containers/evidence/external-artifacts")
        .join(repository_commit)
        .join(format!("{tool_id}.json"));
    let distributable_image_root = study_root.join("containers/current-images");
    let admission = verify_external_artifact_admission(
        repository_root,
        &distributable_image_root,
        &admission_path,
    )
    .with_context(|| format!("verify external artifact admission for {tool_id}"))?;
    if admission.repository_commit != repository_commit
        || admission.catalog.sha256 != catalog_sha256
        || admission.tool_id != tool_id
    {
        bail!("external artifact admission source identity differs for {tool_id}");
    }
    match admission.availability {
        ExternalArtifactAdmissionStatus::BlockedExternalAccess => bail!(
            "external artifact access blocked for {tool_id}: {}",
            admission.blocker_reason.as_deref().unwrap_or("unspecified")
        ),
        ExternalArtifactAdmissionStatus::Unavailable => bail!(
            "external artifact unavailable for {tool_id}: {}",
            admission.blocker_reason.as_deref().unwrap_or("unspecified")
        ),
        ExternalArtifactAdmissionStatus::Admitted => {}
    }
    let artifact = admission.artifact.context("admitted external artifact has no identity")?;
    if artifact.kind != ExternalArtifactKind::ApptainerSif {
        bail!(
            "admitted external artifact for {tool_id} is a binary and requires command-binding authority"
        );
    }
    let declared_path = Path::new(&artifact.path);
    let metadata = fs::symlink_metadata(declared_path)
        .with_context(|| format!("stat admitted external SIF {}", declared_path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("admitted external SIF is not a regular non-symlink file");
    }
    let path = declared_path.canonicalize().with_context(|| {
        format!("canonicalize admitted external SIF {}", declared_path.display())
    })?;
    if path != declared_path {
        bail!("admitted external SIF path is not canonical");
    }

    Ok(CurrentApptainerImage {
        tool_id: admission.tool_id,
        path,
        sha256: artifact.sha256,
        size_bytes: artifact.size_bytes,
        repository_commit: admission.repository_commit,
        catalog_sha256: admission.catalog.sha256,
    })
}

/// Resolve and verify the current retained Apptainer image for one tool.
///
/// # Errors
/// Returns an error when the publication is invalid, the tool is not uniquely current and
/// retained, the selected path escapes STUDY, or the on-disk file differs from inventory evidence.
fn resolve_current_apptainer_image(
    publication: &ContainerImageInventoryPublication,
    study_root: &Path,
    tool_id: &str,
) -> Result<CurrentApptainerImage> {
    validate_image_inventory_publication(publication)
        .context("validate image inventory before current-image resolution")?;
    let tool = publication
        .tools
        .iter()
        .find(|tool| tool.tool == tool_id)
        .with_context(|| format!("image inventory does not declare tool {tool_id}"))?;
    if tool.artifact_delivery != ContainerArtifactDelivery::RedistributableImage
        || tool.state != ContainerImageState::Current
        || !tool.apptainer_supported
    {
        bail!("tool {tool_id} has no current redistributable Apptainer image");
    }
    let selected_sha256 = tool
        .selected_image_sha256
        .as_deref()
        .context("current redistributable-image tool has no selected image digest")?;
    let candidate = tool
        .candidates
        .iter()
        .find(|candidate| candidate.admitted && candidate.sha256 == selected_sha256)
        .context("selected image digest has no admitted candidate")?;
    if !candidate.current_recipe || candidate.disposition != ImageCandidateDisposition::Canonical {
        bail!("selected image is not the current canonical candidate");
    }

    let study_root = study_root
        .canonicalize()
        .with_context(|| format!("canonicalize STUDY root {}", study_root.display()))?;
    let relative = study_relative_path(&candidate.path)?;
    let declared_path = study_root.join(relative);
    let declared_metadata = fs::symlink_metadata(&declared_path)
        .with_context(|| format!("stat selected image {}", declared_path.display()))?;
    if declared_metadata.file_type().is_symlink() || !declared_metadata.is_file() {
        bail!("selected image is not a regular non-symlink file");
    }
    let path = declared_path
        .canonicalize()
        .with_context(|| format!("canonicalize selected image {}", declared_path.display()))?;
    if !path.starts_with(&study_root) {
        bail!("selected image resolves outside STUDY");
    }
    if declared_metadata.len() != candidate.size_bytes {
        bail!("selected image size differs from inventory evidence");
    }
    let sha256 = bijux_dna_infra::hash_file_sha256(&path)
        .with_context(|| format!("hash selected image {}", path.display()))?;
    if sha256 != candidate.sha256 {
        bail!("selected image digest differs from inventory evidence");
    }

    Ok(CurrentApptainerImage {
        tool_id: tool.tool.clone(),
        path,
        sha256,
        size_bytes: candidate.size_bytes,
        repository_commit: publication.repository_commit.clone(),
        catalog_sha256: publication.catalog_sha256.clone(),
    })
}

fn study_relative_path(value: &str) -> Result<&Path> {
    let relative =
        value.strip_prefix("study://").context("selected image path is not rooted at study://")?;
    let path = Path::new(relative);
    if relative.is_empty()
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(component, Component::CurDir | Component::ParentDir | Component::RootDir)
        })
    {
        bail!("selected image path escapes STUDY");
    }
    Ok(path)
}
