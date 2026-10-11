//! Owner: container result movement flattening lifecycle.
use super::super::{
    ResultLayoutMove, ResultLayoutPlan, ResultLayoutPlanRequest, ResultMovementState,
    CONTAINER_RESULT_LAYOUT_PLAN_SCHEMA_V1,
};
use super::admission::{inventory_digest, portable_path, safe_relative};
use super::paths::owned_path;
use super::repository::{
    absolute_path, load_layout_lock, load_layout_plan, path_exists, tracked_locks,
    validate_inventory, write_json,
};
use anyhow::{bail, Context, Result};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Derive a non-mutating plan that flattens historical `<tool>/run-<id>` results.
///
/// # Errors
/// Returns an error when tracked locks, source inventories, or move preconditions are invalid.
pub fn derive_result_layout_plan(request: &ResultLayoutPlanRequest) -> Result<ResultLayoutPlan> {
    let result_root = absolute_path(&request.result_root)?;
    let mut moves = Vec::new();
    let mut targets = BTreeSet::new();
    for (lock_path, lock) in tracked_locks(&request.runs_root)? {
        let source_result = safe_relative(&lock.source_result, "source result")?;
        let source_relative =
            source_result.strip_prefix("results/container-validation").with_context(|| {
                format!("source result is outside container validation: {}", lock_path.display())
            })?;
        let target_relative = PathBuf::from(format!("{}-{}", lock.run_id, lock.tool));
        if source_relative == target_relative {
            continue;
        }
        if source_relative != Path::new(&lock.tool).join(&lock.run_id) {
            bail!("historical result layout is not recognized: {}", source_relative.display());
        }
        let source = owned_path(&result_root, &portable_path(source_relative), "source")?;
        let target = owned_path(&result_root, &portable_path(&target_relative), "target")?;
        if !targets.insert(target.clone()) {
            bail!("duplicate flat result target: {}", target.display());
        }
        if !source.is_dir() {
            bail!("historical result directory is absent: {}", source.display());
        }
        if path_exists(&target)? {
            bail!("flat result target already exists: {}", target.display());
        }
        validate_inventory(&source, &lock.source_files)?;
        moves.push(ResultLayoutMove {
            run_id: lock.run_id,
            tool: lock.tool,
            source: portable_path(source_relative),
            target: portable_path(&target_relative),
            file_count: lock.source_files.len(),
            size_bytes: lock.source_files.iter().map(|file| file.size_bytes).sum(),
            inventory_sha256: inventory_digest(&lock.source_files)?,
            evidence_lock: portable_path(lock_path.strip_prefix(&request.runs_root)?),
            evidence_lock_sha256: bijux_dna_infra::hash_file_sha256(&lock_path)?,
            applied: false,
            applied_at_utc: None,
        });
    }
    Ok(ResultLayoutPlan {
        schema_version: CONTAINER_RESULT_LAYOUT_PLAN_SCHEMA_V1.to_string(),
        state: ResultMovementState::Previewed,
        created_at_utc: request.created_at_utc.clone(),
        result_root: result_root.display().to_string(),
        run_count: moves.len(),
        file_count: moves.iter().map(|movement| movement.file_count).sum(),
        size_bytes: moves.iter().map(|movement| movement.size_bytes).sum(),
        moves,
        removed_empty_directories: Vec::new(),
        completed_at_utc: None,
        rolled_back_at_utc: None,
    })
}

/// Apply or resume a checked result-layout plan.
///
/// # Errors
/// Returns an error when the plan is stale, a source changed, or an atomic rename fails.
pub fn apply_result_layout(
    plan_path: &Path,
    runs_root: &Path,
    result_root: &Path,
    observed_at_utc: &str,
) -> Result<usize> {
    let mut plan = load_layout_plan(plan_path, runs_root, result_root)?;
    if !matches!(plan.state, ResultMovementState::Previewed | ResultMovementState::Applying) {
        bail!("plan cannot be applied from state {:?}", plan.state);
    }
    plan.state = ResultMovementState::Applying;
    write_json(plan_path, &plan)?;
    for index in 0..plan.moves.len() {
        let movement = plan.moves[index].clone();
        let source = owned_path(result_root, &movement.source, "source")?;
        let target = owned_path(result_root, &movement.target, "target")?;
        let lock =
            load_layout_lock(&owned_path(runs_root, &movement.evidence_lock, "evidence lock")?)?;
        if movement.applied {
            validate_inventory(&target, &lock.source_files)?;
            continue;
        }
        if !source.is_dir() || path_exists(&target)? {
            bail!("result move precondition failed: {} -> {}", source.display(), target.display());
        }
        validate_inventory(&source, &lock.source_files)?;
        fs::rename(&source, &target)
            .with_context(|| format!("move result {} -> {}", source.display(), target.display()))?;
        validate_inventory(&target, &lock.source_files)?;
        plan.moves[index].applied = true;
        plan.moves[index].applied_at_utc = Some(observed_at_utc.to_string());
        write_json(plan_path, &plan)?;
    }
    let mut removed = Vec::new();
    let tool_roots = plan
        .moves
        .iter()
        .map(|movement| result_root.join(&movement.source).parent().map(Path::to_path_buf))
        .collect::<Option<BTreeSet<_>>>()
        .context("layout source has no parent")?;
    for tool_root in tool_roots {
        if tool_root.is_dir() && fs::read_dir(&tool_root)?.next().is_none() {
            fs::remove_dir(&tool_root)?;
            removed.push(portable_path(tool_root.strip_prefix(result_root)?));
        }
    }
    plan.removed_empty_directories = removed;
    plan.state = ResultMovementState::Applied;
    plan.completed_at_utc = Some(observed_at_utc.to_string());
    write_json(plan_path, &plan)?;
    Ok(plan.moves.len())
}

/// Verify an applied flat result layout against every recorded byte identity.
///
/// # Errors
/// Returns an error when state, source absence, target inventory, or lock identity differs.
pub fn verify_result_layout(
    plan_path: &Path,
    runs_root: &Path,
    result_root: &Path,
) -> Result<usize> {
    let plan = load_layout_plan(plan_path, runs_root, result_root)?;
    if plan.state != ResultMovementState::Applied {
        bail!("plan is not applied: {:?}", plan.state);
    }
    for movement in &plan.moves {
        let source = owned_path(result_root, &movement.source, "source")?;
        let target = owned_path(result_root, &movement.target, "target")?;
        if path_exists(&source)? {
            bail!("nested result still exists: {}", source.display());
        }
        let lock =
            load_layout_lock(&owned_path(runs_root, &movement.evidence_lock, "evidence lock")?)?;
        validate_inventory(&target, &lock.source_files)?;
    }
    Ok(plan.moves.len())
}

/// Roll back an applied flat result layout in reverse order.
///
/// # Errors
/// Returns an error when an archive changed, a source already exists, or a rename fails.
pub fn rollback_result_layout(
    plan_path: &Path,
    runs_root: &Path,
    result_root: &Path,
    observed_at_utc: &str,
) -> Result<usize> {
    let mut plan = load_layout_plan(plan_path, runs_root, result_root)?;
    if plan.state != ResultMovementState::Applied {
        bail!("plan cannot be rolled back from state {:?}", plan.state);
    }
    for index in (0..plan.moves.len()).rev() {
        let movement = plan.moves[index].clone();
        let source = owned_path(result_root, &movement.source, "source")?;
        let target = owned_path(result_root, &movement.target, "target")?;
        let lock =
            load_layout_lock(&owned_path(runs_root, &movement.evidence_lock, "evidence lock")?)?;
        validate_inventory(&target, &lock.source_files)?;
        if path_exists(&source)? {
            bail!("rollback source already exists: {}", source.display());
        }
        if let Some(parent) = source.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(&target, &source)?;
        plan.moves[index].applied = false;
        plan.moves[index].applied_at_utc = None;
        write_json(plan_path, &plan)?;
    }
    plan.state = ResultMovementState::RolledBack;
    plan.completed_at_utc = None;
    plan.rolled_back_at_utc = Some(observed_at_utc.to_string());
    write_json(plan_path, &plan)?;
    Ok(plan.moves.len())
}
