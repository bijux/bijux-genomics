//! Recoverable result-layout and superseded-attempt archive transactions.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::Deserialize;

use super::{
    ArchivedResultCategory, ResultFileIdentity, ResultLayoutMove, ResultLayoutPlan,
    ResultLayoutPlanRequest, ResultMovementState, SupersededResultArchivePlan,
    SupersededResultArchivePlanRequest, SupersededResultMove,
    CONTAINER_RESULT_LAYOUT_PLAN_SCHEMA_V1, CONTAINER_RUN_ATTEMPT_LEDGER_SCHEMA_V1,
    CONTAINER_SUPERSEDED_RESULT_ARCHIVE_SCHEMA_V1,
};

#[derive(Debug, Deserialize)]
struct LayoutEvidenceLock {
    run_id: String,
    tool: String,
    source_result: String,
    source_files: Vec<ResultFileIdentity>,
}

#[derive(Debug, Deserialize)]
struct AttemptLedgerView {
    schema_version: String,
    attempts: Vec<AttemptLedgerRowView>,
}

#[derive(Debug, Deserialize)]
struct AttemptLedgerRowView {
    run_id: String,
    tool: String,
    evidence_lock_sha256: String,
    superseded_by: String,
}

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
        let source = result_root.join(source_relative);
        let target = result_root.join(&target_relative);
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
        let source = result_root.join(safe_relative(&movement.source, "source")?);
        let target = result_root.join(safe_relative(&movement.target, "target")?);
        let lock = load_layout_lock(&runs_root.join(&movement.evidence_lock))?;
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
        let source = result_root.join(safe_relative(&movement.source, "source")?);
        let target = result_root.join(safe_relative(&movement.target, "target")?);
        if path_exists(&source)? {
            bail!("nested result still exists: {}", source.display());
        }
        let lock = load_layout_lock(&runs_root.join(&movement.evidence_lock))?;
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
        let source = result_root.join(safe_relative(&movement.source, "source")?);
        let target = result_root.join(safe_relative(&movement.target, "target")?);
        let lock = load_layout_lock(&runs_root.join(&movement.evidence_lock))?;
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
        let candidates =
            [result_root.join(&name), result_root.join(&attempt.tool).join(&attempt.run_id)];
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
    let cache = result_root.join("python-cache");
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
        let source = request.result_root.join(safe_relative(&movement.source, "source")?);
        let target = request.archive_root.join(safe_relative(&movement.target, "target")?);
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
        let source = request.result_root.join(safe_relative(&movement.source, "source")?);
        let target = request.archive_root.join(safe_relative(&movement.target, "target")?);
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
        let source = request.result_root.join(safe_relative(&movement.source, "source")?);
        let target = request.archive_root.join(safe_relative(&movement.target, "target")?);
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

fn load_layout_plan(
    plan_path: &Path,
    runs_root: &Path,
    result_root: &Path,
) -> Result<ResultLayoutPlan> {
    let plan: ResultLayoutPlan = load_json(plan_path)?;
    if plan.schema_version != CONTAINER_RESULT_LAYOUT_PLAN_SCHEMA_V1 {
        bail!("unsupported result-layout plan: {}", plan_path.display());
    }
    if Path::new(&plan.result_root) != absolute_path(result_root)? {
        bail!("plan result root differs from the requested root");
    }
    for movement in &plan.moves {
        let lock_path = runs_root.join(safe_relative(&movement.evidence_lock, "evidence lock")?);
        if !lock_path.is_file()
            || bijux_dna_infra::hash_file_sha256(&lock_path)? != movement.evidence_lock_sha256
        {
            bail!("tracked evidence lock changed: {}", lock_path.display());
        }
        let lock = load_layout_lock(&lock_path)?;
        if inventory_digest(&lock.source_files)? != movement.inventory_sha256 {
            bail!("tracked source inventory changed: {}", lock_path.display());
        }
    }
    Ok(plan)
}

fn load_archive_plan(
    plan_path: &Path,
    request: &SupersededResultArchivePlanRequest,
) -> Result<SupersededResultArchivePlan> {
    let plan: SupersededResultArchivePlan = load_json(plan_path)?;
    if plan.schema_version != CONTAINER_SUPERSEDED_RESULT_ARCHIVE_SCHEMA_V1 {
        bail!("unsupported superseded archive plan: {}", plan_path.display());
    }
    if Path::new(&plan.result_root) != absolute_path(&request.result_root)?
        || Path::new(&plan.archive_root) != absolute_path(&request.archive_root)?
    {
        bail!("archive plan roots differ from requested roots");
    }
    if bijux_dna_infra::hash_file_sha256(&request.attempt_ledger_path)?
        != plan.attempt_ledger_sha256
    {
        bail!("attempt ledger changed after archive planning");
    }
    let finals = final_result_names(&request.runs_root)?;
    if plan.final_results.iter().cloned().collect::<BTreeSet<_>>() != finals {
        bail!("published final run identities changed after planning");
    }
    for movement in &plan.moves {
        if inventory_digest(&movement.files)? != movement.inventory_sha256 {
            bail!("archive plan inventory digest is invalid");
        }
    }
    Ok(plan)
}

fn tracked_locks(runs_root: &Path) -> Result<Vec<(PathBuf, LayoutEvidenceLock)>> {
    let mut records = Vec::new();
    for path in child_directories(runs_root)? {
        let lock_path = path.join("evidence.lock.json");
        if !lock_path.is_file() {
            continue;
        }
        let lock = load_layout_lock(&lock_path)?;
        validate_run_and_tool(&lock.run_id, &lock.tool)?;
        if path.file_name().and_then(|name| name.to_str())
            != Some(format!("{}-{}", lock.run_id, lock.tool).as_str())
        {
            bail!("tracked run identity is invalid: {}", lock_path.display());
        }
        records.push((lock_path, lock));
    }
    if records.is_empty() {
        bail!("no tracked run evidence found under {}", runs_root.display());
    }
    Ok(records)
}

fn load_layout_lock(path: &Path) -> Result<LayoutEvidenceLock> {
    load_json(path).with_context(|| format!("load layout evidence lock {}", path.display()))
}

fn load_attempt_ledger(path: &Path) -> Result<AttemptLedgerView> {
    let ledger: AttemptLedgerView = load_json(path)?;
    if ledger.schema_version != CONTAINER_RUN_ATTEMPT_LEDGER_SCHEMA_V1 {
        bail!("unsupported attempt ledger: {}", path.display());
    }
    Ok(ledger)
}

fn final_result_names(runs_root: &Path) -> Result<BTreeSet<String>> {
    Ok(tracked_locks(runs_root)?
        .into_iter()
        .map(|(_, lock)| format!("{}-{}", lock.run_id, lock.tool))
        .collect())
}

fn directory_inventory(root: &Path) -> Result<Vec<ResultFileIdentity>> {
    let mut files = Vec::new();
    collect_files(root, root, &mut files)?;
    if files.is_empty() {
        bail!("result directory contains no evidence: {}", root.display());
    }
    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(files)
}

fn collect_files(root: &Path, current: &Path, files: &mut Vec<ResultFileIdentity>) -> Result<()> {
    let mut entries = fs::read_dir(current)?
        .map(|entry| entry.map(|value| value.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect_files(root, &path, files)?;
        } else if path.is_file() {
            files.push(ResultFileIdentity {
                path: portable_path(path.strip_prefix(root)?),
                size_bytes: path.metadata()?.len(),
                sha256: bijux_dna_infra::hash_file_sha256(&path)?,
            });
        }
    }
    Ok(())
}

fn validate_inventory(root: &Path, files: &[ResultFileIdentity]) -> Result<()> {
    let mut expected = BTreeSet::new();
    for file in files {
        let relative = safe_relative(&file.path, "inventory path")?;
        if !expected.insert(relative.clone()) {
            bail!("duplicate inventory path for {}: {}", root.display(), relative.display());
        }
        validate_sha256(&file.sha256, "recorded result digest")?;
        let candidate = root.join(relative);
        let metadata = candidate
            .metadata()
            .with_context(|| format!("recorded result file is absent: {}", candidate.display()))?;
        if metadata.len() != file.size_bytes {
            bail!("recorded result size changed: {}", candidate.display());
        }
        if bijux_dna_infra::hash_file_sha256(&candidate)? != file.sha256 {
            bail!("recorded result digest changed: {}", candidate.display());
        }
    }
    let actual = directory_inventory(root)?
        .into_iter()
        .map(|file| PathBuf::from(file.path))
        .collect::<BTreeSet<_>>();
    if actual != expected {
        bail!("result inventory differs for {}", root.display());
    }
    Ok(())
}

fn inventory_digest(files: &[ResultFileIdentity]) -> Result<String> {
    let normalized = files
        .iter()
        .map(|file| {
            BTreeMap::from([
                ("path", serde_json::Value::String(file.path.clone())),
                ("sha256", serde_json::Value::String(file.sha256.clone())),
                ("size_bytes", serde_json::Value::Number(file.size_bytes.into())),
            ])
        })
        .collect::<Vec<_>>();
    Ok(bijux_dna_infra::hash_bytes_sha256(&serde_json::to_vec(&normalized)?))
}

fn remove_empty_child_directories(root: &Path) -> Result<()> {
    for path in child_directories(root)? {
        if fs::read_dir(&path)?.next().is_none() {
            fs::remove_dir(path)?;
        }
    }
    Ok(())
}

fn validate_run_and_tool(run_id: &str, tool: &str) -> Result<()> {
    let valid_run = run_id.strip_prefix("run-").is_some_and(|number| {
        number.len() >= 4
            && !number.starts_with('0')
            && number.bytes().all(|byte| byte.is_ascii_digit())
    });
    let valid_tool = !tool.is_empty()
        && tool
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_');
    if !valid_run || !valid_tool {
        bail!("attempt identity is invalid: {run_id}-{tool}");
    }
    Ok(())
}

fn validate_sha256(value: &str, label: &str) -> Result<()> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("{label} is invalid: {value}");
    }
    Ok(())
}

fn safe_relative(value: &str, label: &str) -> Result<PathBuf> {
    let path = PathBuf::from(value);
    if value.is_empty()
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_))
        })
    {
        bail!("{label} is not a safe relative path: {value}");
    }
    Ok(path)
}

fn absolute_path(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

fn path_exists(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

fn portable_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn child_directories(root: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = fs::read_dir(root)
        .with_context(|| format!("read directory {}", root.display()))?
        .map(|entry| entry.map(|value| value.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    paths.retain(|path| path.is_dir());
    paths.sort();
    Ok(paths)
}

fn load_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path).with_context(|| format!("read JSON {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse JSON {}", path.display()))
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
    bijux_dna_infra::atomic_write_json(path, value)?;
    Ok(())
}
