use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};

use super::super::{
    load_image_inventory_publication, ContainerImageInventoryPublication, ContainerImageState,
    DetachedImageDisposition, EvidenceSourceIdentity, ImageCandidateDisposition,
};
use super::model::{
    ImageStorageMove, ImageStorageMoveCategory, ImageStoragePlan, ImageStoragePlanState,
    IMAGE_STORAGE_PLAN_SCHEMA_V1,
};

pub struct ImageStoragePlanRequest<'a> {
    pub inventory_path: &'a Path,
    pub image_root: &'a Path,
    pub quarantine_root: &'a Path,
    pub study_root: &'a Path,
    pub created_at_utc: &'a str,
}

/// Derive and publish a no-clobber plan for quarantining only unreferenced image material.
///
/// # Errors
/// Returns an error when the inventory is invalid, paths escape the governed roots, file identity
/// has drifted, or the plan destination already exists.
pub fn create_image_storage_plan(
    request: &ImageStoragePlanRequest<'_>,
    output: &Path,
) -> Result<ImageStoragePlan> {
    require_timestamp(request.created_at_utc, "image storage plan")?;
    let inventory = load_image_inventory_publication(request.inventory_path)?;
    let image_root =
        request.image_root.canonicalize().context("canonicalize image-storage root")?;
    let study_root = request.study_root.canonicalize().context("canonicalize study root")?;
    let quarantine_root =
        governed_quarantine_root(&image_root, request.image_root, request.quarantine_root)?;
    let mut records = collect_moves(&inventory, &study_root, &image_root, &quarantine_root)?;
    records.sort_by(|left, right| left.source.cmp(&right.source));
    let plan = ImageStoragePlan {
        schema_version: IMAGE_STORAGE_PLAN_SCHEMA_V1.to_string(),
        state: ImageStoragePlanState::Previewed,
        created_at_utc: request.created_at_utc.to_string(),
        inventory: EvidenceSourceIdentity {
            path: request.inventory_path.display().to_string(),
            sha256: bijux_dna_infra::hash_file_sha256(request.inventory_path)
                .context("hash image-storage inventory")?,
        },
        image_root: image_root.display().to_string(),
        quarantine_root: quarantine_root.display().to_string(),
        file_count: u64::try_from(records.len()).context("count image-storage moves")?,
        size_bytes: records.iter().try_fold(0_u64, |total, row| {
            total.checked_add(row.size_bytes).context("sum image-storage move bytes")
        })?,
        moves: records,
        completed_at_utc: None,
        rolled_back_at_utc: None,
    };
    validate_image_storage_plan(&plan)?;
    bijux_dna_infra::atomic_create_json(output, &plan).context("publish image-storage plan")?;
    Ok(plan)
}

/// Resume or apply every planned move, recording durable progress after each rename.
///
/// # Errors
/// Returns an error when plan or source evidence has drifted, a path escapes its governed root, a
/// target already exists, or a source/target file does not match its immutable identity.
pub fn apply_image_storage_plan(
    plan_path: &Path,
    image_root: &Path,
    observed_at_utc: &str,
) -> Result<ImageStoragePlan> {
    require_timestamp(observed_at_utc, "image storage application")?;
    let mut plan = load_governed_plan(plan_path, image_root)?;
    if !matches!(plan.state, ImageStoragePlanState::Previewed | ImageStoragePlanState::Applying) {
        bail!("image-storage plan cannot be applied from {:?}", plan.state);
    }
    plan.state = ImageStoragePlanState::Applying;
    persist_plan(plan_path, &plan)?;
    let governed_image_root = Path::new(&plan.image_root);
    let governed_quarantine_root = Path::new(&plan.quarantine_root);
    prepare_quarantine_root(governed_image_root, governed_quarantine_root)?;
    for index in 0..plan.moves.len() {
        let row = &plan.moves[index];
        let target = PathBuf::from(&row.target);
        require_lexical_child(&target, governed_quarantine_root, "quarantine target")?;
        if row.applied {
            reject_existing_path(Path::new(&row.source), "applied image source")?;
            require_existing_file_under(
                &row.target,
                governed_quarantine_root,
                "quarantined image",
            )?;
            validate_file_identity(&target, row.size_bytes, &row.sha256)?;
            continue;
        }
        let source_exists = path_exists(Path::new(&row.source));
        let target_exists = path_exists(&target);
        if !source_exists && target_exists {
            require_existing_file_under(
                &row.target,
                governed_quarantine_root,
                "recovered quarantined image",
            )?;
            validate_file_identity(&target, row.size_bytes, &row.sha256)?;
            plan.moves[index].applied = true;
            plan.moves[index].applied_at_utc = Some(observed_at_utc.to_string());
            persist_plan(plan_path, &plan)?;
            continue;
        }
        if source_exists && target_exists {
            bail!("image-storage source and target both exist for a pending move");
        }
        let source = require_existing_file_under(&row.source, governed_image_root, "image source")?;
        validate_file_identity(&source, row.size_bytes, &row.sha256)?;
        prepare_destination_parent(&target, governed_quarantine_root)?;
        reject_existing_path(&target, "quarantine target")?;
        bijux_dna_infra::rename(&source, &target).context("move image into quarantine")?;
        require_existing_file_under(&row.target, governed_quarantine_root, "quarantined image")?;
        validate_file_identity(&target, row.size_bytes, &row.sha256)?;
        plan.moves[index].applied = true;
        plan.moves[index].applied_at_utc = Some(observed_at_utc.to_string());
        persist_plan(plan_path, &plan)?;
    }
    plan.state = ImageStoragePlanState::Applied;
    plan.completed_at_utc = Some(observed_at_utc.to_string());
    persist_plan(plan_path, &plan)?;
    Ok(plan)
}

/// Verify that an applied plan has no remaining sources and exact quarantined targets.
///
/// # Errors
/// Returns an error when plan evidence changed or any source/target state differs from the plan.
pub fn verify_image_storage_plan(plan_path: &Path, image_root: &Path) -> Result<ImageStoragePlan> {
    let plan = load_governed_plan(plan_path, image_root)?;
    if plan.state != ImageStoragePlanState::Applied {
        bail!("image-storage plan is not applied");
    }
    for row in &plan.moves {
        reject_existing_path(Path::new(&row.source), "quarantined source")?;
        require_existing_file_under(
            &row.target,
            Path::new(&plan.quarantine_root),
            "quarantined image",
        )?;
        validate_file_identity(Path::new(&row.target), row.size_bytes, &row.sha256)?;
    }
    Ok(plan)
}

/// Roll back an applied plan in reverse order while preserving checksummed progress.
///
/// # Errors
/// Returns an error when plan evidence changed, a target digest differs, or a source already exists.
pub fn rollback_image_storage_plan(
    plan_path: &Path,
    image_root: &Path,
    observed_at_utc: &str,
) -> Result<ImageStoragePlan> {
    require_timestamp(observed_at_utc, "image storage rollback")?;
    let mut plan = load_governed_plan(plan_path, image_root)?;
    if !matches!(plan.state, ImageStoragePlanState::Applied | ImageStoragePlanState::RollingBack) {
        bail!("image-storage plan cannot be rolled back from {:?}", plan.state);
    }
    plan.state = ImageStoragePlanState::RollingBack;
    persist_plan(plan_path, &plan)?;
    for index in (0..plan.moves.len()).rev() {
        let row = &plan.moves[index];
        let source = PathBuf::from(&row.source);
        let target_path = PathBuf::from(&row.target);
        if !row.applied {
            require_existing_file_under(
                &row.source,
                Path::new(&plan.image_root),
                "restored image",
            )?;
            reject_existing_path(&target_path, "rolled-back quarantine target")?;
            validate_file_identity(&source, row.size_bytes, &row.sha256)?;
            continue;
        }
        let source_exists = path_exists(&source);
        let target_exists = path_exists(&target_path);
        if source_exists && !target_exists {
            require_existing_file_under(
                &row.source,
                Path::new(&plan.image_root),
                "recovered restored image",
            )?;
            validate_file_identity(&source, row.size_bytes, &row.sha256)?;
            plan.moves[index].applied = false;
            plan.moves[index].applied_at_utc = None;
            persist_plan(plan_path, &plan)?;
            continue;
        }
        if source_exists && target_exists {
            bail!("image-storage source and target both exist during rollback");
        }
        let target = require_existing_file_under(
            &row.target,
            Path::new(&plan.quarantine_root),
            "quarantined image",
        )?;
        validate_file_identity(&target, row.size_bytes, &row.sha256)?;
        reject_existing_path(&source, "rollback source")?;
        require_lexical_child(&source, Path::new(&plan.image_root), "rollback source")?;
        prepare_destination_parent(&source, Path::new(&plan.image_root))?;
        bijux_dna_infra::rename(&target, &source).context("restore quarantined image")?;
        require_existing_file_under(&row.source, Path::new(&plan.image_root), "restored image")?;
        validate_file_identity(&source, row.size_bytes, &row.sha256)?;
        plan.moves[index].applied = false;
        plan.moves[index].applied_at_utc = None;
        persist_plan(plan_path, &plan)?;
    }
    plan.state = ImageStoragePlanState::RolledBack;
    plan.rolled_back_at_utc = Some(observed_at_utc.to_string());
    persist_plan(plan_path, &plan)?;
    Ok(plan)
}

/// Load and validate a persisted image-storage plan.
///
/// # Errors
/// Returns an error when the document, state summary, path closure, or source inventory is invalid.
pub fn load_image_storage_plan(path: &Path) -> Result<ImageStoragePlan> {
    let bytes = bijux_dna_infra::read_to_end_bounded(path, 16 * 1024 * 1024)
        .context("read image-storage plan")?;
    let plan: ImageStoragePlan =
        serde_json::from_slice(&bytes).context("parse image-storage plan")?;
    validate_image_storage_plan(&plan)?;
    Ok(plan)
}

fn load_governed_plan(plan_path: &Path, image_root: &Path) -> Result<ImageStoragePlan> {
    let plan = load_image_storage_plan(plan_path)?;
    let requested_root = image_root.canonicalize().context("canonicalize requested image root")?;
    if Path::new(&plan.image_root) != requested_root {
        bail!("image-storage plan root differs from requested root");
    }
    let inventory_path = Path::new(&plan.inventory.path);
    if !inventory_path.is_file()
        || bijux_dna_infra::hash_file_sha256(inventory_path)
            .context("hash image-storage source inventory")?
            != plan.inventory.sha256
    {
        bail!("image-storage source inventory changed");
    }
    load_image_inventory_publication(inventory_path)?;
    Ok(plan)
}

fn collect_moves(
    inventory: &ContainerImageInventoryPublication,
    study_root: &Path,
    image_root: &Path,
    quarantine_root: &Path,
) -> Result<Vec<ImageStorageMove>> {
    let mut moves =
        collect_selected_historical_moves(inventory, study_root, image_root, quarantine_root)?;
    for candidate in inventory.tools.iter().flat_map(|tool| &tool.candidates).filter(|candidate| {
        matches!(candidate.disposition, ImageCandidateDisposition::HistoricalRunDependency)
    }) {
        if candidate.real_run_references == 0 || candidate.run_references.is_empty() {
            bail!("historical image candidate has no real-run references");
        }
        moves.push(move_identity(
            &candidate.path,
            candidate.size_bytes,
            &candidate.sha256,
            ImageStorageMoveCategory::Historical,
            study_root,
            image_root,
            quarantine_root,
        )?);
    }
    for image in &inventory.historical_images {
        if image.disposition != DetachedImageDisposition::HistoricalRunDependency
            || image.real_run_references == 0
            || image.run_references.is_empty()
        {
            bail!("detached historical image has no real-run reference authority");
        }
        moves.push(move_identity(
            &image.path,
            image.size_bytes,
            &image.sha256,
            ImageStorageMoveCategory::Historical,
            study_root,
            image_root,
            quarantine_root,
        )?);
    }
    for candidate in inventory.tools.iter().flat_map(|tool| &tool.candidates).filter(|candidate| {
        matches!(candidate.disposition, ImageCandidateDisposition::UnreferencedObsolete)
    }) {
        if candidate.real_run_references != 0 {
            bail!("obsolete image candidate retains real-run references");
        }
        moves.push(move_identity(
            &candidate.path,
            candidate.size_bytes,
            &candidate.sha256,
            ImageStorageMoveCategory::Obsolete,
            study_root,
            image_root,
            quarantine_root,
        )?);
    }
    for image in inventory
        .unknown_images
        .iter()
        .filter(|image| matches!(image.disposition, DetachedImageDisposition::UnreferencedObsolete))
    {
        if image.real_run_references != 0 {
            bail!("obsolete detached image retains real-run references");
        }
        moves.push(move_identity(
            &image.path,
            image.size_bytes,
            &image.sha256,
            ImageStorageMoveCategory::Obsolete,
            study_root,
            image_root,
            quarantine_root,
        )?);
    }
    for image in &inventory.staged_images {
        moves.push(move_identity(
            &image.path,
            image.size_bytes,
            &image.sha256,
            ImageStorageMoveCategory::Interrupted,
            study_root,
            image_root,
            quarantine_root,
        )?);
    }
    let mut sources = BTreeSet::new();
    let mut targets = BTreeSet::new();
    for row in &moves {
        if !sources.insert(row.source.as_str()) || !targets.insert(row.target.as_str()) {
            bail!("image-storage plan contains duplicate source or target paths");
        }
        reject_existing_path(Path::new(&row.target), "quarantine target")?;
    }
    Ok(moves)
}

fn collect_selected_historical_moves(
    inventory: &ContainerImageInventoryPublication,
    study_root: &Path,
    image_root: &Path,
    quarantine_root: &Path,
) -> Result<Vec<ImageStorageMove>> {
    let mut moves = Vec::new();
    for tool in inventory.tools.iter().filter(|tool| tool.state == ContainerImageState::Historical)
    {
        let selected_sha256 = tool
            .selected_image_sha256
            .as_deref()
            .context("historical tool has no selected image digest")?;
        let mut selected =
            tool.candidates.iter().filter(|candidate| candidate.sha256 == selected_sha256);
        let candidate = selected.next().context("historical selected image is absent")?;
        if selected.next().is_some()
            || !candidate.admitted
            || candidate.current_recipe
            || candidate.disposition != ImageCandidateDisposition::Canonical
        {
            bail!("historical selected image has inconsistent inventory identity");
        }
        let category = match (candidate.real_run_references, candidate.run_references.is_empty()) {
            (0, true) => ImageStorageMoveCategory::Obsolete,
            (count, false) if count > 0 => ImageStorageMoveCategory::Historical,
            _ => bail!("historical selected image has inconsistent run references"),
        };
        moves.push(move_identity(
            &candidate.path,
            candidate.size_bytes,
            &candidate.sha256,
            category,
            study_root,
            image_root,
            quarantine_root,
        )?);
    }
    Ok(moves)
}

#[allow(clippy::too_many_arguments)]
fn move_identity(
    locator: &str,
    size_bytes: u64,
    sha256: &str,
    category: ImageStorageMoveCategory,
    study_root: &Path,
    image_root: &Path,
    quarantine_root: &Path,
) -> Result<ImageStorageMove> {
    let source = resolve_image_locator(locator, study_root)?;
    let source =
        require_existing_file_under(&source.display().to_string(), image_root, "inventory image")?;
    validate_file_identity(&source, size_bytes, sha256)?;
    let file_name = source.file_name().context("inventory image has no file name")?;
    let category_name = match category {
        ImageStorageMoveCategory::Historical => "historical",
        ImageStorageMoveCategory::Obsolete => "obsolete",
        ImageStorageMoveCategory::Interrupted => "interrupted",
    };
    let target = quarantine_root.join(category_name).join(file_name);
    Ok(ImageStorageMove {
        category,
        source: source.display().to_string(),
        target: target.display().to_string(),
        size_bytes,
        sha256: sha256.to_string(),
        applied: false,
        applied_at_utc: None,
    })
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

fn governed_quarantine_root(
    canonical_image_root: &Path,
    requested_image_root: &Path,
    requested_quarantine_root: &Path,
) -> Result<PathBuf> {
    if !requested_image_root.is_absolute() || !requested_quarantine_root.is_absolute() {
        bail!("quarantine root must be absolute");
    }
    require_lexical_child(requested_quarantine_root, requested_image_root, "quarantine root")?;
    let relative = requested_quarantine_root
        .strip_prefix(requested_image_root)
        .context("derive quarantine path below image root")?;
    Ok(canonical_image_root.join(relative))
}

fn prepare_quarantine_root(image_root: &Path, quarantine_root: &Path) -> Result<()> {
    bijux_dna_infra::ensure_dir(quarantine_root).context("create image quarantine root")?;
    let canonical_image_root = image_root.canonicalize().context("canonicalize image root")?;
    let canonical_quarantine =
        quarantine_root.canonicalize().context("canonicalize image quarantine root")?;
    if canonical_quarantine == canonical_image_root
        || !canonical_quarantine.starts_with(&canonical_image_root)
    {
        bail!("quarantine root resolves outside the image root");
    }
    if !bijux_dna_infra::same_filesystem(&canonical_image_root, &canonical_quarantine)
        .context("verify image quarantine filesystem")?
    {
        bail!("quarantine root must share the image-root filesystem");
    }
    Ok(())
}

fn prepare_destination_parent(destination: &Path, root: &Path) -> Result<()> {
    let parent = destination.parent().context("image-storage destination has no parent")?;
    bijux_dna_infra::ensure_dir(parent).context("create image-storage destination parent")?;
    let canonical_root =
        root.canonicalize().context("canonicalize image-storage destination root")?;
    let canonical_parent =
        parent.canonicalize().context("canonicalize image-storage destination parent")?;
    if canonical_parent != canonical_root && !canonical_parent.starts_with(&canonical_root) {
        bail!("image-storage destination parent resolves outside its governed root");
    }
    if !bijux_dna_infra::same_filesystem(&canonical_root, &canonical_parent)
        .context("verify image-storage destination filesystem")?
    {
        bail!("image-storage destination must share its governed-root filesystem");
    }
    Ok(())
}

fn require_existing_file_under(value: &str, root: &Path, label: &str) -> Result<PathBuf> {
    let path = Path::new(value);
    let metadata = path.symlink_metadata().with_context(|| format!("inspect {label} {value}"))?;
    if !metadata.file_type().is_file() {
        bail!("{label} is not a regular file: {value}");
    }
    let canonical = path.canonicalize().with_context(|| format!("canonicalize {label} {value}"))?;
    let canonical_root =
        root.canonicalize().with_context(|| format!("canonicalize {label} root"))?;
    if !canonical.starts_with(&canonical_root) || canonical == canonical_root {
        bail!("{label} is outside {}: {value}", canonical_root.display());
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
    let metadata = path.symlink_metadata().context("inspect image-storage file")?;
    if !metadata.file_type().is_file() || metadata.len() != expected_size {
        bail!("image-storage file type or size changed: {}", path.display());
    }
    let actual = bijux_dna_infra::hash_file_sha256(path).context("hash image-storage file")?;
    if actual != expected_sha256 {
        bail!("image-storage file digest changed: {}", path.display());
    }
    Ok(())
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

fn validate_image_storage_plan(plan: &ImageStoragePlan) -> Result<()> {
    if plan.schema_version != IMAGE_STORAGE_PLAN_SCHEMA_V1 {
        bail!("unsupported image-storage plan schema: {}", plan.schema_version);
    }
    require_timestamp(&plan.created_at_utc, "image storage plan")?;
    let root = Path::new(&plan.image_root);
    let quarantine = Path::new(&plan.quarantine_root);
    require_lexical_child(quarantine, root, "quarantine root")?;
    if plan.file_count != u64::try_from(plan.moves.len()).context("count image-storage moves")?
        || plan.size_bytes
            != plan.moves.iter().try_fold(0_u64, |total, row| {
                total.checked_add(row.size_bytes).context("sum image-storage move bytes")
            })?
    {
        bail!("image-storage plan summary differs from move records");
    }
    let mut sources = BTreeSet::new();
    let mut targets = BTreeSet::new();
    for row in &plan.moves {
        require_lexical_child(Path::new(&row.source), root, "image source")?;
        require_lexical_child(Path::new(&row.target), quarantine, "quarantine target")?;
        if row.size_bytes == 0
            || row.sha256.len() != 64
            || !row.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
            || !sources.insert(row.source.as_str())
            || !targets.insert(row.target.as_str())
            || row.applied != row.applied_at_utc.is_some()
        {
            bail!("image-storage plan contains an invalid move identity");
        }
    }
    match plan.state {
        ImageStoragePlanState::Previewed if plan.moves.iter().any(|row| row.applied) => {
            bail!("previewed image-storage plan contains applied moves");
        }
        ImageStoragePlanState::Applied if plan.moves.iter().any(|row| !row.applied) => {
            bail!("applied image-storage plan contains pending moves");
        }
        ImageStoragePlanState::RolledBack if plan.moves.iter().any(|row| row.applied) => {
            bail!("rolled-back image-storage plan contains applied moves");
        }
        _ => {}
    }
    Ok(())
}

fn persist_plan(path: &Path, plan: &ImageStoragePlan) -> Result<()> {
    validate_image_storage_plan(plan)?;
    bijux_dna_infra::atomic_write_json(path, plan).context("persist image-storage plan")
}

fn require_timestamp(value: &str, label: &str) -> Result<()> {
    if value.trim().is_empty() || !value.ends_with('Z') || !value.contains('T') {
        bail!("{label} timestamp is not UTC RFC 3339");
    }
    Ok(())
}
