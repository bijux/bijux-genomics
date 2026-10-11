//! Owner: container result movement superseded-attempt archive lifecycle.
use super::super::{
    ArchivedResultCategory, ResultMovementState, SupersededResultArchivePlan,
    SupersededResultArchivePlanRequest, SupersededResultMove,
    CONTAINER_SUPERSEDED_RESULT_ARCHIVE_SCHEMA_V1,
};
use super::admission::{inventory_digest, portable_path, validate_run_and_tool, validate_sha256};
use super::paths::owned_path;
use super::repository::{
    absolute_path, child_directories, directory_inventory, load_archive_plan, load_attempt_ledger,
    path_exists, remove_empty_child_directories, tracked_locks, validate_inventory, write_json,
};
use anyhow::{bail, Result};
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

/// Derive a no-clobber archive plan for superseded attempts and generated Python cache.
///
/// # Errors
/// Returns an error when ledger lineage, active finals, source uniqueness, or inventories fail.
pub fn derive_superseded_result_archive_plan(
    request: &SupersededResultArchivePlanRequest,
) -> Result<SupersededResultArchivePlan> {
    let result_root = absolute_path(&request.result_root)?;
    let archive_root = absolute_path(&request.archive_root)?;
    if path_exists(&archive_root)? {
        bail!("archive target already exists: {}", archive_root.display());
    }
    let ledger = load_attempt_ledger(&request.attempt_ledger_path)?;
    let locks = tracked_locks(&request.runs_root)?;
    let final_by_id = locks
        .iter()
        .map(|(_, lock)| (lock.run_id.as_str(), lock.tool.as_str()))
        .collect::<BTreeMap<_, _>>();
    let final_results =
        final_by_id.iter().map(|(run_id, tool)| format!("{run_id}-{tool}")).collect::<Vec<_>>();
    let mut moves = Vec::new();
    let mut seen = BTreeSet::new();
    for attempt in ledger.attempts {
        validate_run_and_tool(&attempt.run_id, &attempt.tool)?;
        if final_by_id.get(attempt.superseded_by.as_str()) != Some(&attempt.tool.as_str()) {
            bail!(
                "attempt replacement is not a published {} result: {}",
                attempt.tool,
                attempt.superseded_by
            );
        }
        validate_sha256(&attempt.evidence_lock_sha256, "attempt lock digest")?;
        let name = format!("{}-{}", attempt.run_id, attempt.tool);
        if !seen.insert(name.clone()) {
            bail!("duplicate attempt identity: {name}");
        }
        let candidates = [
            owned_path(&result_root, &name, "source")?,
            owned_path(&result_root, &format!("{}/{}", attempt.tool, attempt.run_id), "source")?,
        ];
        let sources = candidates.iter().filter(|path| path.is_dir()).collect::<Vec<_>>();
        if sources.len() != 1 {
            bail!("expected one result directory for {name}, found {}", sources.len());
        }
        let source = sources[0];
        let files = directory_inventory(source)?;
        moves.push(SupersededResultMove {
            category: ArchivedResultCategory::SupersededAttempt,
            run_id: Some(attempt.run_id),
            tool: Some(attempt.tool),
            source: portable_path(source.strip_prefix(&result_root)?),
            target: name,
            file_count: files.len(),
            size_bytes: files.iter().map(|file| file.size_bytes).sum(),
            inventory_sha256: inventory_digest(&files)?,
            files,
            evidence_lock_sha256: Some(attempt.evidence_lock_sha256),
            superseded_by: Some(attempt.superseded_by),
            applied: false,
            applied_at_utc: None,
        });
    }
    let cache = owned_path(&result_root, "python-cache", "cache")?;
    if cache.is_dir() {
        let files = directory_inventory(&cache)?;
        moves.push(SupersededResultMove {
            category: ArchivedResultCategory::GeneratedCache,
            run_id: None,
            tool: None,
            source: "python-cache".to_string(),
            target: "python-cache".to_string(),
            file_count: files.len(),
            size_bytes: files.iter().map(|file| file.size_bytes).sum(),
            inventory_sha256: inventory_digest(&files)?,
            files,
            evidence_lock_sha256: None,
            superseded_by: None,
            applied: false,
            applied_at_utc: None,
        });
    }
    Ok(SupersededResultArchivePlan {
        schema_version: CONTAINER_SUPERSEDED_RESULT_ARCHIVE_SCHEMA_V1.to_string(),
        state: ResultMovementState::Previewed,
        created_at_utc: request.created_at_utc.clone(),
        result_root: result_root.display().to_string(),
        archive_root: archive_root.display().to_string(),
        attempt_ledger_sha256: bijux_dna_infra::hash_file_sha256(&request.attempt_ledger_path)?,
        final_results,
        move_count: moves.len(),
        file_count: moves.iter().map(|movement| movement.file_count).sum(),
        size_bytes: moves.iter().map(|movement| movement.size_bytes).sum(),
        moves,
        completed_at_utc: None,
        rolled_back_at_utc: None,
    })
}

/// Apply or resume a superseded-result archive plan.
///
/// # Errors
/// Returns an error when provenance or bytes drift, a target exists, or a rename fails.
pub fn apply_superseded_result_archive(
    plan_path: &Path,
    request: &SupersededResultArchivePlanRequest,
    observed_at_utc: &str,
) -> Result<usize> {
    let mut plan = load_archive_plan(plan_path, request)?;
    if !matches!(plan.state, ResultMovementState::Previewed | ResultMovementState::Applying) {
        bail!("archive cannot be applied from {:?}", plan.state);
    }
    fs::create_dir_all(&request.archive_root)?;
    plan.state = ResultMovementState::Applying;
    write_json(plan_path, &plan)?;
    for index in 0..plan.moves.len() {
        let movement = plan.moves[index].clone();
        let source = owned_path(&request.result_root, &movement.source, "source")?;
        let target = owned_path(&request.archive_root, &movement.target, "target")?;
        if movement.applied {
            validate_inventory(&target, &movement.files)?;
            continue;
        }
        if !source.is_dir() || path_exists(&target)? {
            bail!("archive move precondition failed: {} -> {}", source.display(), target.display());
        }
        validate_inventory(&source, &movement.files)?;
        fs::rename(&source, &target)?;
        validate_inventory(&target, &movement.files)?;
        plan.moves[index].applied = true;
        plan.moves[index].applied_at_utc = Some(observed_at_utc.to_string());
        write_json(plan_path, &plan)?;
    }
    remove_empty_child_directories(&request.result_root)?;
    plan.state = ResultMovementState::Applied;
    plan.completed_at_utc = Some(observed_at_utc.to_string());
    write_json(plan_path, &plan)?;
    verify_superseded_result_archive(plan_path, request)?;
    Ok(plan.moves.len())
}

/// Verify archived attempts and the exact active final-result roster.
///
/// # Errors
/// Returns an error when provenance, archive bytes, source absence, or active finals differ.
pub fn verify_superseded_result_archive(
    plan_path: &Path,
    request: &SupersededResultArchivePlanRequest,
) -> Result<usize> {
    let plan = load_archive_plan(plan_path, request)?;
    if plan.state != ResultMovementState::Applied {
        bail!("archive is not applied: {:?}", plan.state);
    }
    for movement in &plan.moves {
        let source = owned_path(&request.result_root, &movement.source, "source")?;
        let target = owned_path(&request.archive_root, &movement.target, "target")?;
        if path_exists(&source)? {
            bail!("superseded result remains active: {}", source.display());
        }
        validate_inventory(&target, &movement.files)?;
    }
    let active = child_directories(&request.result_root)?
        .into_iter()
        .filter_map(|path| path.file_name().and_then(|name| name.to_str()).map(str::to_string))
        .filter(|name| !name.starts_with('.'))
        .collect::<BTreeSet<_>>();
    let expected = plan.final_results.iter().cloned().collect::<BTreeSet<_>>();
    if active != expected {
        bail!(
            "active results differ from published finals; extra={:?}, missing={:?}",
            active.difference(&expected).collect::<Vec<_>>(),
            expected.difference(&active).collect::<Vec<_>>()
        );
    }
    Ok(plan.moves.len())
}

/// Roll back an applied superseded-result archive in reverse order.
///
/// # Errors
/// Returns an error when archive bytes changed, a source exists, or restoration fails.
pub fn rollback_superseded_result_archive(
    plan_path: &Path,
    request: &SupersededResultArchivePlanRequest,
    observed_at_utc: &str,
) -> Result<usize> {
    let mut plan = load_archive_plan(plan_path, request)?;
    if plan.state != ResultMovementState::Applied {
        bail!("archive cannot be rolled back from {:?}", plan.state);
    }
    for index in (0..plan.moves.len()).rev() {
        let movement = plan.moves[index].clone();
        let source = owned_path(&request.result_root, &movement.source, "source")?;
        let target = owned_path(&request.archive_root, &movement.target, "target")?;
        validate_inventory(&target, &movement.files)?;
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
    fs::remove_dir(&request.archive_root)?;
    plan.state = ResultMovementState::RolledBack;
    plan.completed_at_utc = None;
    plan.rolled_back_at_utc = Some(observed_at_utc.to_string());
    write_json(plan_path, &plan)?;
    Ok(plan.moves.len())
}
