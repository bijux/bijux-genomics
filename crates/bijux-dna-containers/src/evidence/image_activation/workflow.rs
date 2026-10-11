use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};

use super::super::{
    load_image_inventory_publication, ContainerArtifactDelivery,
    ContainerImageInventoryPublication, ContainerImageState, EvidenceSourceIdentity,
    ImageCandidateDisposition,
};
use super::model::{
    ImageActivationPlan, ImageActivationPlanState, ImageActivationPublication,
    IMAGE_ACTIVATION_PLAN_SCHEMA_V1,
};

pub struct ImageActivationPlanRequest<'a> {
    pub inventory_path: &'a Path,
    pub study_root: &'a Path,
    pub source_root: &'a Path,
    pub active_root: &'a Path,
    pub created_at_utc: &'a str,
}

/// Derive and publish a no-clobber plan for activating canonical images by hard link.
///
/// # Errors
/// Returns an error when inventory selection is incomplete, roots are unsafe, source identity has
/// drifted, or any active destination already exists.
pub fn create_image_activation_plan(
    request: &ImageActivationPlanRequest<'_>,
    output: &Path,
) -> Result<ImageActivationPlan> {
    require_timestamp(request.created_at_utc, "image activation plan")?;
    let inventory = load_image_inventory_publication(request.inventory_path)?;
    let study_root = request.study_root.canonicalize().context("canonicalize study root")?;
    let source_root = request.source_root.canonicalize().context("canonicalize source root")?;
    let active_root = request.active_root.canonicalize().context("canonicalize active root")?;
    require_distinct_roots(&study_root, &source_root, &active_root)?;
    if !bijux_dna_infra::same_filesystem(&source_root, &active_root)
        .context("verify activation roots share a filesystem")?
    {
        bail!("image activation requires source and active roots on one filesystem");
    }
    let publications = collect_publications(&inventory, &study_root, &source_root, &active_root)?;
    let plan = ImageActivationPlan {
        schema_version: IMAGE_ACTIVATION_PLAN_SCHEMA_V1.to_string(),
        state: ImageActivationPlanState::Previewed,
        created_at_utc: request.created_at_utc.to_string(),
        inventory: EvidenceSourceIdentity {
            path: request.inventory_path.display().to_string(),
            sha256: bijux_dna_infra::hash_file_sha256(request.inventory_path)
                .context("hash image-activation inventory")?,
        },
        source_root: source_root.display().to_string(),
        active_root: active_root.display().to_string(),
        file_count: u64::try_from(publications.len()).context("count image publications")?,
        size_bytes: publications.iter().try_fold(0_u64, |total, row| {
            total.checked_add(row.size_bytes).context("sum image publication bytes")
        })?,
        publications,
        completed_at_utc: None,
        rolled_back_at_utc: None,
    };
    validate_image_activation_plan(&plan)?;
    bijux_dna_infra::atomic_create_json(output, &plan).context("publish image-activation plan")?;
    Ok(plan)
}

/// Apply or resume an activation plan and persist progress after every image publication.
///
/// # Errors
/// Returns an error when evidence drifts, a destination conflicts, or hard-link publication fails.
pub fn apply_image_activation_plan(
    plan_path: &Path,
    source_root: &Path,
    active_root: &Path,
    observed_at_utc: &str,
) -> Result<ImageActivationPlan> {
    require_timestamp(observed_at_utc, "image activation application")?;
    let mut plan = load_governed_plan(plan_path, source_root, active_root)?;
    if !matches!(
        plan.state,
        ImageActivationPlanState::Previewed | ImageActivationPlanState::Applying
    ) {
        bail!("image-activation plan cannot be applied from {:?}", plan.state);
    }
    plan.state = ImageActivationPlanState::Applying;
    persist_plan(plan_path, &plan)?;
    for index in 0..plan.publications.len() {
        let row = &plan.publications[index];
        let source = Path::new(&row.source);
        let target = Path::new(&row.target);
        require_existing_file_under(source, Path::new(&plan.source_root), "activation source")?;
        validate_file_identity(source, row.size_bytes, &row.sha256)?;
        if row.applied {
            require_existing_file_under(target, Path::new(&plan.active_root), "active image")?;
            validate_file_identity(target, row.size_bytes, &row.sha256)?;
            require_same_file(source, target)?;
            continue;
        }
        if path_exists(target) {
            require_existing_file_under(
                target,
                Path::new(&plan.active_root),
                "recovered active image",
            )?;
            validate_file_identity(target, row.size_bytes, &row.sha256)?;
            require_same_file(source, target)?;
        } else {
            fs::hard_link(source, target)
                .with_context(|| format!("activate image {}", target.display()))?;
            require_existing_file_under(target, Path::new(&plan.active_root), "active image")?;
            validate_file_identity(target, row.size_bytes, &row.sha256)?;
            require_same_file(source, target)?;
        }
        plan.publications[index].applied = true;
        plan.publications[index].applied_at_utc = Some(observed_at_utc.to_string());
        persist_plan(plan_path, &plan)?;
    }
    plan.state = ImageActivationPlanState::Applied;
    plan.completed_at_utc = Some(observed_at_utc.to_string());
    persist_plan(plan_path, &plan)?;
    Ok(plan)
}

/// Verify exact source and active identities for every applied publication.
///
/// # Errors
/// Returns an error when the plan is not applied or any path, digest, size, or hard-link identity
/// differs from the persisted plan.
pub fn verify_image_activation_plan(
    plan_path: &Path,
    source_root: &Path,
    active_root: &Path,
) -> Result<ImageActivationPlan> {
    let plan = load_governed_plan(plan_path, source_root, active_root)?;
    if plan.state != ImageActivationPlanState::Applied {
        bail!("image-activation plan is not applied");
    }
    for row in &plan.publications {
        let source = require_existing_file_under(
            Path::new(&row.source),
            Path::new(&plan.source_root),
            "activation source",
        )?;
        let target = require_existing_file_under(
            Path::new(&row.target),
            Path::new(&plan.active_root),
            "active image",
        )?;
        validate_file_identity(&source, row.size_bytes, &row.sha256)?;
        validate_file_identity(&target, row.size_bytes, &row.sha256)?;
        require_same_file(&source, &target)?;
    }
    Ok(plan)
}

/// Remove only links created by an applied plan while retaining immutable staging sources.
///
/// # Errors
/// Returns an error when source evidence changed, an active link conflicts, or removal fails.
pub fn rollback_image_activation_plan(
    plan_path: &Path,
    source_root: &Path,
    active_root: &Path,
    observed_at_utc: &str,
) -> Result<ImageActivationPlan> {
    require_timestamp(observed_at_utc, "image activation rollback")?;
    let mut plan = load_governed_plan(plan_path, source_root, active_root)?;
    if !matches!(
        plan.state,
        ImageActivationPlanState::Applied | ImageActivationPlanState::RollingBack
    ) {
        bail!("image-activation plan cannot be rolled back from {:?}", plan.state);
    }
    plan.state = ImageActivationPlanState::RollingBack;
    persist_plan(plan_path, &plan)?;
    for index in (0..plan.publications.len()).rev() {
        let row = &plan.publications[index];
        let source = require_existing_file_under(
            Path::new(&row.source),
            Path::new(&plan.source_root),
            "activation source",
        )?;
        validate_file_identity(&source, row.size_bytes, &row.sha256)?;
        let target = Path::new(&row.target);
        if row.applied && path_exists(target) {
            require_existing_file_under(target, Path::new(&plan.active_root), "active image")?;
            validate_file_identity(target, row.size_bytes, &row.sha256)?;
            require_same_file(&source, target)?;
            fs::remove_file(target)
                .with_context(|| format!("remove active image link {}", target.display()))?;
        } else if !row.applied && path_exists(target) {
            bail!("rolled-back activation target exists: {}", target.display());
        }
        plan.publications[index].applied = false;
        plan.publications[index].applied_at_utc = None;
        persist_plan(plan_path, &plan)?;
    }
    plan.state = ImageActivationPlanState::RolledBack;
    plan.rolled_back_at_utc = Some(observed_at_utc.to_string());
    persist_plan(plan_path, &plan)?;
    Ok(plan)
}

/// Load and validate a persisted image-activation plan.
///
/// # Errors
/// Returns an error when the document or lifecycle invariants are invalid.
pub fn load_image_activation_plan(path: &Path) -> Result<ImageActivationPlan> {
    let bytes = bijux_dna_infra::read_to_end_bounded(path, 16 * 1024 * 1024)
        .context("read image-activation plan")?;
    let plan: ImageActivationPlan =
        serde_json::from_slice(&bytes).context("parse image-activation plan")?;
    validate_image_activation_plan(&plan)?;
    Ok(plan)
}

fn collect_publications(
    inventory: &ContainerImageInventoryPublication,
    study_root: &Path,
    source_root: &Path,
    active_root: &Path,
) -> Result<Vec<ImageActivationPublication>> {
    let mut publications = Vec::new();
    for tool in &inventory.tools {
        if tool.artifact_delivery != ContainerArtifactDelivery::RedistributableImage {
            continue;
        }
        if tool.state != ContainerImageState::Current {
            bail!(
                "redistributable-image tool {} is not current in activation inventory",
                tool.tool
            );
        }
        let selected = tool.selected_image_sha256.as_deref().with_context(|| {
            format!("redistributable-image tool {} has no selected image", tool.tool)
        })?;
        let candidates = tool
            .candidates
            .iter()
            .filter(|candidate| {
                candidate.disposition == ImageCandidateDisposition::Canonical
                    && candidate.sha256 == selected
            })
            .collect::<Vec<_>>();
        let [candidate] = candidates.as_slice() else {
            bail!(
                "redistributable-image tool {} does not have exactly one selected canonical image",
                tool.tool
            );
        };
        let source = resolve_image_locator(&candidate.path, study_root)?;
        let source = require_existing_file_under(&source, source_root, "activation source")?;
        validate_file_identity(&source, candidate.size_bytes, &candidate.sha256)?;
        let file_name = source.file_name().context("activation source has no file name")?;
        let target = active_root.join(file_name);
        reject_existing_path(&target, "active image target")?;
        publications.push(ImageActivationPublication {
            tool: tool.tool.clone(),
            source: source.display().to_string(),
            target: target.display().to_string(),
            size_bytes: candidate.size_bytes,
            sha256: candidate.sha256.clone(),
            applied: false,
            applied_at_utc: None,
        });
    }
    publications.sort_by(|left, right| left.tool.cmp(&right.tool));
    let mut tools = BTreeSet::new();
    let mut sources = BTreeSet::new();
    let mut targets = BTreeSet::new();
    for row in &publications {
        if !tools.insert(row.tool.as_str())
            || !sources.insert(row.source.as_str())
            || !targets.insert(row.target.as_str())
        {
            bail!("image-activation plan contains duplicate tool, source, or target identity");
        }
    }
    if publications.is_empty() {
        bail!("image-activation inventory selects no retained images");
    }
    Ok(publications)
}

fn load_governed_plan(
    plan_path: &Path,
    source_root: &Path,
    active_root: &Path,
) -> Result<ImageActivationPlan> {
    let plan = load_image_activation_plan(plan_path)?;
    let requested_source =
        source_root.canonicalize().context("canonicalize requested source root")?;
    let requested_active =
        active_root.canonicalize().context("canonicalize requested active root")?;
    if Path::new(&plan.source_root) != requested_source
        || Path::new(&plan.active_root) != requested_active
    {
        bail!("image-activation plan roots differ from requested roots");
    }
    let inventory_path = Path::new(&plan.inventory.path);
    if !inventory_path.is_file()
        || bijux_dna_infra::hash_file_sha256(inventory_path)
            .context("hash image-activation source inventory")?
            != plan.inventory.sha256
    {
        bail!("image-activation source inventory changed");
    }
    load_image_inventory_publication(inventory_path)?;
    Ok(plan)
}

fn require_distinct_roots(study_root: &Path, source_root: &Path, active_root: &Path) -> Result<()> {
    if source_root == active_root
        || !source_root.starts_with(study_root)
        || !active_root.starts_with(study_root)
        || source_root.starts_with(active_root)
        || active_root.starts_with(source_root)
    {
        bail!("activation roots must be non-overlapping descendants of the study root");
    }
    Ok(())
}

fn resolve_image_locator(locator: &str, study_root: &Path) -> Result<PathBuf> {
    if let Some(relative) = locator.strip_prefix("study://") {
        let relative = Path::new(relative);
        if relative.is_absolute()
            || relative.components().any(|component| {
                matches!(
                    component,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            bail!("image locator escapes the study root: {locator}");
        }
        return Ok(study_root.join(relative));
    }
    let path = PathBuf::from(locator);
    if !path.is_absolute() {
        bail!("image locator is neither study-relative nor absolute: {locator}");
    }
    Ok(path)
}

fn require_existing_file_under(path: &Path, root: &Path, label: &str) -> Result<PathBuf> {
    let metadata =
        path.symlink_metadata().with_context(|| format!("inspect {label} {}", path.display()))?;
    if !metadata.file_type().is_file() {
        bail!("{label} is not a regular file: {}", path.display());
    }
    let canonical =
        path.canonicalize().with_context(|| format!("canonicalize {label} {}", path.display()))?;
    let canonical_root =
        root.canonicalize().with_context(|| format!("canonicalize {label} root"))?;
    if canonical == canonical_root || !canonical.starts_with(&canonical_root) {
        bail!("{label} is outside {}: {}", canonical_root.display(), path.display());
    }
    Ok(canonical)
}

fn require_lexical_child(path: &Path, root: &Path, label: &str) -> Result<()> {
    if !path.is_absolute()
        || !root.is_absolute()
        || path == root
        || !path.starts_with(root)
        || path.components().any(|component| matches!(component, Component::ParentDir))
    {
        bail!("{label} is outside {}: {}", root.display(), path.display());
    }
    Ok(())
}

fn validate_file_identity(path: &Path, expected_size: u64, expected_sha256: &str) -> Result<()> {
    let metadata = path.symlink_metadata().context("inspect image-activation file")?;
    if !metadata.file_type().is_file() || metadata.len() != expected_size {
        bail!("image-activation file type or size changed: {}", path.display());
    }
    let actual = bijux_dna_infra::hash_file_sha256(path).context("hash image-activation file")?;
    if actual != expected_sha256 {
        bail!("image-activation file digest changed: {}", path.display());
    }
    Ok(())
}

#[cfg(unix)]
fn require_same_file(source: &Path, target: &Path) -> Result<()> {
    use std::os::unix::fs::MetadataExt;

    let source_metadata = source.metadata().context("inspect activation source inode")?;
    let target_metadata = target.metadata().context("inspect activation target inode")?;
    if source_metadata.dev() != target_metadata.dev()
        || source_metadata.ino() != target_metadata.ino()
    {
        bail!("active image is not the planned hard link: {}", target.display());
    }
    Ok(())
}

#[cfg(not(unix))]
fn require_same_file(_source: &Path, _target: &Path) -> Result<()> {
    bail!("image activation requires Unix hard-link identity support")
}

fn reject_existing_path(path: &Path, label: &str) -> Result<()> {
    if path_exists(path) {
        bail!("{label} already exists: {}", path.display());
    }
    Ok(())
}

fn path_exists(path: &Path) -> bool {
    path.exists() || path.symlink_metadata().is_ok()
}

fn validate_image_activation_plan(plan: &ImageActivationPlan) -> Result<()> {
    if plan.schema_version != IMAGE_ACTIVATION_PLAN_SCHEMA_V1 {
        bail!("unsupported image-activation plan schema: {}", plan.schema_version);
    }
    require_timestamp(&plan.created_at_utc, "image activation plan")?;
    let source_root = Path::new(&plan.source_root);
    let active_root = Path::new(&plan.active_root);
    if source_root == active_root || !source_root.is_absolute() || !active_root.is_absolute() {
        bail!("image-activation roots must be distinct absolute paths");
    }
    if plan.file_count != u64::try_from(plan.publications.len()).context("count publications")?
        || plan.size_bytes
            != plan.publications.iter().try_fold(0_u64, |total, row| {
                total.checked_add(row.size_bytes).context("sum publication bytes")
            })?
    {
        bail!("image-activation plan summary differs from publications");
    }
    let mut tools = BTreeSet::new();
    let mut sources = BTreeSet::new();
    let mut targets = BTreeSet::new();
    for row in &plan.publications {
        require_lexical_child(Path::new(&row.source), source_root, "activation source")?;
        require_lexical_child(Path::new(&row.target), active_root, "active image target")?;
        if row.tool.is_empty()
            || row.size_bytes == 0
            || row.sha256.len() != 64
            || !row.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
            || !tools.insert(row.tool.as_str())
            || !sources.insert(row.source.as_str())
            || !targets.insert(row.target.as_str())
            || row.applied != row.applied_at_utc.is_some()
        {
            bail!("image-activation plan contains an invalid publication identity");
        }
    }
    match plan.state {
        ImageActivationPlanState::Previewed if plan.publications.iter().any(|row| row.applied) => {
            bail!("previewed image-activation plan contains applied publications");
        }
        ImageActivationPlanState::Applied if plan.publications.iter().any(|row| !row.applied) => {
            bail!("applied image-activation plan contains pending publications");
        }
        ImageActivationPlanState::RolledBack if plan.publications.iter().any(|row| row.applied) => {
            bail!("rolled-back image-activation plan contains applied publications");
        }
        _ => {}
    }
    Ok(())
}

fn persist_plan(path: &Path, plan: &ImageActivationPlan) -> Result<()> {
    validate_image_activation_plan(plan)?;
    bijux_dna_infra::atomic_write_json(path, plan).context("persist image-activation plan")
}

fn require_timestamp(value: &str, label: &str) -> Result<()> {
    if value.trim().is_empty() || !value.ends_with('Z') || !value.contains('T') {
        bail!("{label} timestamp is not UTC RFC 3339");
    }
    Ok(())
}
