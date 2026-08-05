//! Resolve one admitted inventory selection to an exact local Apptainer image.

use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};

use super::{
    validate_image_inventory_publication, ContainerImageInventoryPublication, ContainerImageState,
    ContainerSupportDisposition, ImageCandidateDisposition,
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

/// Resolve and verify the current retained Apptainer image for one tool.
///
/// # Errors
/// Returns an error when the publication is invalid, the tool is not uniquely current and
/// retained, the selected path escapes STUDY, or the on-disk file differs from inventory evidence.
pub fn resolve_current_apptainer_image(
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
    if tool.support_disposition != ContainerSupportDisposition::Retained
        || tool.state != ContainerImageState::Current
        || !tool.apptainer_supported
    {
        bail!("tool {tool_id} has no current retained Apptainer image");
    }
    let selected_sha256 = tool
        .selected_image_sha256
        .as_deref()
        .context("current retained tool has no selected image digest")?;
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
