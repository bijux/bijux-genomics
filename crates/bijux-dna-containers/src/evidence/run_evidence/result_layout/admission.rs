//! Owner: container result movement pure identity, path and aggregate admission.
use super::super::{
    ArchivedResultCategory, ResultFileIdentity, ResultLayoutMove, ResultLayoutPlan,
    SupersededResultArchivePlan,
};
use super::observations::{AttemptLedgerView, LayoutEvidenceLock};
use anyhow::{bail, Context, Result};
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

pub(super) fn inventory_digest(files: &[ResultFileIdentity]) -> Result<String> {
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

pub(super) fn validate_run_and_tool(run_id: &str, tool: &str) -> Result<()> {
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

pub(super) fn validate_sha256(value: &str, label: &str) -> Result<()> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("{label} is invalid: {value}");
    }
    Ok(())
}

pub(super) fn safe_relative(value: &str, label: &str) -> Result<PathBuf> {
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

pub(super) fn portable_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn checked_totals(mut rows: impl Iterator<Item = (usize, u64)>) -> Result<(usize, u64)> {
    rows.try_fold((0usize, 0u64), |(files, bytes), (next_files, next_bytes)| {
        Ok((
            files.checked_add(next_files).context("movement file count overflow")?,
            bytes.checked_add(next_bytes).context("movement byte count overflow")?,
        ))
    })
}

fn file_totals(files: &[ResultFileIdentity]) -> Result<(usize, u64)> {
    if files.is_empty() {
        bail!("movement inventory is empty");
    }
    let mut paths = BTreeSet::new();
    for file in files {
        let path = safe_relative(&file.path, "inventory path")?;
        if !paths.insert(path) {
            bail!("duplicate movement inventory path");
        }
        validate_sha256(&file.sha256, "movement file digest")?;
    }
    checked_totals(files.iter().map(|file| (1, file.size_bytes)))
}

pub(super) fn validate_layout_totals(plan: &ResultLayoutPlan) -> Result<()> {
    let totals = checked_totals(plan.moves.iter().map(|row| (row.file_count, row.size_bytes)))?;
    if plan.run_count != plan.moves.len() || totals != (plan.file_count, plan.size_bytes) {
        bail!("result-layout aggregate counts differ from movements");
    }
    let mut sources = BTreeSet::new();
    let mut targets = BTreeSet::new();
    for row in &plan.moves {
        if !sources.insert(&row.source) || !targets.insert(&row.target) {
            bail!("duplicate result-layout source or target");
        }
    }
    Ok(())
}

pub(super) fn validate_layout_binding(
    row: &ResultLayoutMove,
    lock: &LayoutEvidenceLock,
) -> Result<()> {
    validate_run_and_tool(&lock.run_id, &lock.tool)?;
    let source = format!("{}/{}", lock.tool, lock.run_id);
    let target = format!("{}-{}", lock.run_id, lock.tool);
    if row.run_id != lock.run_id
        || row.tool != lock.tool
        || row.source != source
        || row.target != target
        || row.evidence_lock != format!("{target}/evidence.lock.json")
        || lock.source_result != format!("results/container-validation/{source}")
    {
        bail!("result-layout movement identity or destination differs from tracked evidence");
    }
    if file_totals(&lock.source_files)? != (row.file_count, row.size_bytes) {
        bail!("result-layout movement counts differ from tracked inventory");
    }
    Ok(())
}

pub(super) fn validate_archive_bindings(
    plan: &SupersededResultArchivePlan,
    ledger: &AttemptLedgerView,
) -> Result<()> {
    let totals = checked_totals(plan.moves.iter().map(|row| (row.file_count, row.size_bytes)))?;
    if plan.move_count != plan.moves.len() || totals != (plan.file_count, plan.size_bytes) {
        bail!("archive aggregate counts differ from movements");
    }
    let finals = plan.final_results.iter().cloned().collect::<BTreeSet<_>>();
    if finals.len() != plan.final_results.len() {
        bail!("duplicate archive final identity");
    }
    let mut attempts = BTreeMap::new();
    for attempt in &ledger.attempts {
        validate_run_and_tool(&attempt.run_id, &attempt.tool)?;
        validate_sha256(&attempt.evidence_lock_sha256, "attempt lock digest")?;
        let name = format!("{}-{}", attempt.run_id, attempt.tool);
        if attempts.insert(name, attempt).is_some() {
            bail!("duplicate archive ledger attempt");
        }
        if !finals.contains(&format!("{}-{}", attempt.superseded_by, attempt.tool)) {
            bail!("archive replacement is not a published final result");
        }
    }
    let mut sources = BTreeSet::new();
    let mut targets = BTreeSet::new();
    let mut observed_attempts = BTreeSet::new();
    for row in &plan.moves {
        if !sources.insert(&row.source) || !targets.insert(&row.target) {
            bail!("duplicate archive source or target");
        }
        if file_totals(&row.files)? != (row.file_count, row.size_bytes) {
            bail!("archive movement counts differ from inventory");
        }
        match row.category {
            ArchivedResultCategory::SupersededAttempt => {
                let attempt =
                    attempts.get(&row.target).context("archive movement has no ledger attempt")?;
                let historical = format!("{}/{}", attempt.tool, attempt.run_id);
                if row.run_id.as_deref() != Some(attempt.run_id.as_str())
                    || row.tool.as_deref() != Some(attempt.tool.as_str())
                    || row.evidence_lock_sha256.as_deref()
                        != Some(attempt.evidence_lock_sha256.as_str())
                    || row.superseded_by.as_deref() != Some(attempt.superseded_by.as_str())
                    || (row.source != row.target && row.source != historical)
                    || finals.contains(&row.target)
                {
                    bail!("archive movement identity or destination differs from ledger");
                }
                observed_attempts.insert(row.target.clone());
            }
            ArchivedResultCategory::GeneratedCache => {
                if row.source != "python-cache"
                    || row.target != "python-cache"
                    || row.run_id.is_some()
                    || row.tool.is_some()
                    || row.evidence_lock_sha256.is_some()
                    || row.superseded_by.is_some()
                {
                    bail!("archive cache movement contains an unrelated identity or destination");
                }
            }
        }
    }
    if observed_attempts != attempts.keys().cloned().collect::<BTreeSet<_>>() {
        bail!("archive movements omit ledger attempts");
    }
    Ok(())
}
