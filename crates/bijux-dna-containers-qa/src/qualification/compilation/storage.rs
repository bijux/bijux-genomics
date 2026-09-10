//! Filesystem identity checks and frozen-repository binding for qualification compilation.
use super::binding::validate_input_declaration;
use crate::qualification::specification::QualificationSpecificationSourceV1;
use crate::qualification::specification::{
    admission::{require_safe_component, require_safe_relative, validate_specification_header},
    QualificationSpecificationCompilationRequest, QualificationSpecificationInputV1,
    QUALIFICATION_SPECIFICATION_SOURCE_SCHEMA_V1,
};
use anyhow::{bail, Context, Result};
use bijux_dna_containers::api::{
    validate_container_governance, verify_frozen_repository, ContainerGovernance,
    ContainerImageState, ImageCandidateDisposition,
};
use std::fs;
use std::path::{Path, PathBuf};

/// Bind a qualification source to a governed commit-addressed immutable repository.
///
/// # Errors
/// Rejects inconsistent governance or a repository that is not an admitted frozen snapshot.
pub fn bind_qualification_source_to_frozen_repository(
    source: &QualificationSpecificationSourceV1,
    governance: &ContainerGovernance,
    frozen_repository: &Path,
) -> Result<QualificationSpecificationSourceV1> {
    validate_container_governance(governance)?;
    if source.schema_version != QUALIFICATION_SPECIFICATION_SOURCE_SCHEMA_V1 {
        bail!("unsupported qualification specification source schema");
    }
    let study_root = source
        .governed_study_root
        .canonicalize()
        .context("canonicalize qualification source STUDY root")?;
    let governed_study_root =
        governance.paths.study_root.canonicalize().context("canonicalize governed STUDY root")?;
    if study_root != governed_study_root {
        bail!("qualification source STUDY root differs from site governance");
    }
    let declared_result_relative = source
        .result_root
        .strip_prefix(&source.governed_study_root)
        .context("qualification result root is outside its declared STUDY root")?;
    require_safe_relative(declared_result_relative, "qualification result root")?;
    let input_relatives = source
        .inputs
        .iter()
        .map(|input| {
            let relative = input
                .path
                .strip_prefix(&source.governed_study_root)
                .context("qualification input is outside its declared STUDY root")?;
            require_safe_relative(relative, "qualification input")?;
            Ok(relative.to_path_buf())
        })
        .collect::<Result<Vec<_>>>()?;
    let frozen_root = governance
        .paths
        .frozen_repositories
        .canonicalize()
        .context("canonicalize governed frozen-repository root")?;
    let governed_results_root = governance
        .paths
        .results_root
        .canonicalize()
        .context("canonicalize governed qualification results root")?;
    let result_name = format!("{}-{}", source.run_id, source.tool_id);
    require_safe_component(&result_name, "qualification result identity")?;
    let repository =
        frozen_repository.canonicalize().context("canonicalize qualification frozen repository")?;
    if repository.parent() != Some(frozen_root.as_path())
        || repository.file_name().and_then(|name| name.to_str())
            != Some(source.repository_commit.as_str())
    {
        bail!("qualification source repository is not frozen by commit identity");
    }
    verify_frozen_repository(&repository, &source.repository_commit)
        .context("verify qualification source frozen repository")?;
    let mut bound = source.clone();
    bound.governed_study_root.clone_from(&governed_study_root);
    bound.repository_path = repository;
    bound.result_root = governed_results_root.join("container-validation").join(result_name);
    for (input, relative) in bound.inputs.iter_mut().zip(input_relatives) {
        input.path = governed_study_root.join(relative);
    }
    Ok(bound)
}

pub(super) fn require_source_header(
    request: &QualificationSpecificationCompilationRequest,
    source: &QualificationSpecificationSourceV1,
) -> Result<()> {
    validate_specification_header(source)?;
    if !request.repository_root.is_dir() {
        bail!("qualification compiler repository root is unavailable");
    }
    Ok(())
}

pub(super) fn resolve_current_image(
    inventory: &bijux_dna_containers::api::ContainerImageInventoryPublication,
    source: &QualificationSpecificationSourceV1,
) -> Result<(PathBuf, String)> {
    let tool = inventory
        .tools
        .iter()
        .find(|tool| tool.tool == source.tool_id)
        .context("qualification tool is absent from image inventory")?;
    if tool.state != ContainerImageState::Current {
        bail!("qualification tool has no canonical current image");
    }
    let digest = tool
        .selected_image_sha256
        .as_deref()
        .context("qualification tool has no selected image identity")?;
    let candidates = tool
        .candidates
        .iter()
        .filter(|candidate| {
            candidate.sha256 == digest
                && candidate.admitted
                && candidate.current_recipe
                && candidate.disposition == ImageCandidateDisposition::Canonical
        })
        .collect::<Vec<_>>();
    let [candidate] = candidates.as_slice() else {
        bail!("qualification tool does not resolve to exactly one admitted canonical image");
    };
    let image_path = resolve_governed_path(&source.governed_study_root, &candidate.path)?;
    let active_image_root = source.governed_study_root.join("containers/apptainer");
    if image_path.parent() != Some(active_image_root.as_path())
        || image_path.extension().and_then(|value| value.to_str()) != Some("sif")
    {
        bail!("qualification image is not a canonical SIF under the active image root");
    }
    let metadata = fs::symlink_metadata(&image_path)
        .with_context(|| format!("inspect qualification image {}", image_path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("qualification image is not a regular file under the active image root");
    }
    let canonical_active_root =
        active_image_root.canonicalize().context("canonicalize qualification active image root")?;
    let canonical_image = image_path.canonicalize().context("canonicalize qualification image")?;
    if canonical_image.parent() != Some(canonical_active_root.as_path()) {
        bail!("qualification image resolves outside the active image root");
    }
    if hash(&canonical_image)? != digest {
        bail!("qualification image bytes differ from the selected inventory identity");
    }
    Ok((canonical_image, digest.to_string()))
}

pub(super) fn source_path(root: &Path, tool_id: &str) -> Option<PathBuf> {
    ["yml", "yaml", "json"]
        .into_iter()
        .map(|extension| root.join(format!("{tool_id}.{extension}")))
        .find(|path| path.is_file())
}

pub(super) fn hash(path: &Path) -> Result<String> {
    bijux_dna_infra::hash_file_sha256(path).with_context(|| format!("hash {}", path.display()))
}

fn resolve_governed_path(study_root: &Path, value: &str) -> Result<PathBuf> {
    let relative = value
        .strip_prefix("study://")
        .context("qualification image inventory path is not STUDY-governed")?;
    let relative = Path::new(relative);
    require_safe_relative(relative, "qualification image inventory path")?;
    Ok(study_root.join(relative))
}

pub(super) fn validate_input(
    input: &QualificationSpecificationInputV1,
    study_root: &Path,
) -> Result<()> {
    validate_input_declaration(input, study_root)?;
    if bijux_dna_infra::hash_path_sha256(&input.path)
        .with_context(|| format!("hash qualification input {}", input.path.display()))?
        != input.sha256
    {
        bail!("qualification input bytes differ from the declared identity: {}", input.id);
    }
    Ok(())
}
