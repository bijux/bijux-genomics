//! Owner: container result movement current evidence acquisition and filesystem effects.
use super::super::{
    ResultFileIdentity, ResultLayoutPlan, SupersededResultArchivePlan,
    SupersededResultArchivePlanRequest, CONTAINER_RESULT_LAYOUT_PLAN_SCHEMA_V1,
    CONTAINER_RUN_ATTEMPT_LEDGER_SCHEMA_V1, CONTAINER_SUPERSEDED_RESULT_ARCHIVE_SCHEMA_V1,
};
use super::admission::{
    inventory_digest, portable_path, safe_relative, validate_archive_bindings,
    validate_layout_binding, validate_layout_totals, validate_run_and_tool, validate_sha256,
};
use super::observations::{AttemptLedgerView, LayoutEvidenceLock};
use anyhow::{bail, Context, Result};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn load_layout_plan(
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
    validate_layout_totals(&plan)?;
    for movement in &plan.moves {
        let lock_path = runs_root.join(safe_relative(&movement.evidence_lock, "evidence lock")?);
        if !lock_path.is_file()
            || bijux_dna_infra::hash_file_sha256(&lock_path)? != movement.evidence_lock_sha256
        {
            bail!("tracked evidence lock changed: {}", lock_path.display());
        }
        let lock = load_layout_lock(&lock_path)?;
        validate_layout_binding(movement, &lock)?;
        if inventory_digest(&lock.source_files)? != movement.inventory_sha256 {
            bail!("tracked source inventory changed: {}", lock_path.display());
        }
    }
    Ok(plan)
}

pub(super) fn load_archive_plan(
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
    let ledger = load_attempt_ledger(&request.attempt_ledger_path)?;
    validate_archive_bindings(&plan, &ledger)?;
    for movement in &plan.moves {
        if inventory_digest(&movement.files)? != movement.inventory_sha256 {
            bail!("archive plan inventory digest is invalid");
        }
    }
    Ok(plan)
}

pub(super) fn tracked_locks(runs_root: &Path) -> Result<Vec<(PathBuf, LayoutEvidenceLock)>> {
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

pub(super) fn load_layout_lock(path: &Path) -> Result<LayoutEvidenceLock> {
    load_json(path).with_context(|| format!("load layout evidence lock {}", path.display()))
}

pub(super) fn load_attempt_ledger(path: &Path) -> Result<AttemptLedgerView> {
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

pub(super) fn directory_inventory(root: &Path) -> Result<Vec<ResultFileIdentity>> {
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

pub(super) fn validate_inventory(root: &Path, files: &[ResultFileIdentity]) -> Result<()> {
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

pub(super) fn remove_empty_child_directories(root: &Path) -> Result<()> {
    for path in child_directories(root)? {
        if fs::read_dir(&path)?.next().is_none() {
            fs::remove_dir(path)?;
        }
    }
    Ok(())
}

pub(super) fn absolute_path(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

pub(super) fn path_exists(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

pub(super) fn child_directories(root: &Path) -> Result<Vec<PathBuf>> {
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

pub(super) fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
    bijux_dna_infra::atomic_write_json(path, value)?;
    Ok(())
}
