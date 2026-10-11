//! Validation for finalization progress ledgers.

use std::collections::{BTreeMap, BTreeSet};

use crate::diagnostics::BenchError;
use crate::model::{
    BenchmarkProgressAttemptStatus, BenchmarkProgressExecutionUnitStatus, BenchmarkProgressLedger,
    BenchmarkProgressTaskStatus,
};

use super::BENCHMARK_PROGRESS_LEDGER_SCHEMA_V1;

/// Validate one complete progress ledger and its cross-record frontier references.
///
/// # Errors
/// Returns an error when identities, status-specific evidence, references, or frontier partitions
/// are malformed or inconsistent.
pub fn validate_benchmark_progress_ledger(
    ledger: &BenchmarkProgressLedger,
) -> Result<(), BenchError> {
    if ledger.schema_version != BENCHMARK_PROGRESS_LEDGER_SCHEMA_V1 {
        return Err(invalid("benchmark progress ledger schema mismatch"));
    }
    if ledger.cycle_id.trim().len() < 12 {
        return Err(invalid("benchmark progress ledger cycle_id is too short"));
    }
    validate_git_sha(&ledger.source.local_head, "local_head")?;
    validate_git_sha(&ledger.source.remote_head, "remote_head")?;
    validate_sha256(&ledger.source.catalog_sha256, "catalog_sha256")?;
    validate_sha256(&ledger.source.benchmark_registry_sha256, "benchmark_registry_sha256")?;

    let task_statuses = validate_tasks(ledger)?;
    let unit_statuses = validate_execution_units(ledger)?;
    validate_attempts(ledger)?;
    validate_commits_and_checks(ledger)?;
    validate_external_waits(ledger)?;
    validate_frontier(ledger, &task_statuses, &unit_statuses)
}

fn validate_tasks(
    ledger: &BenchmarkProgressLedger,
) -> Result<BTreeMap<&str, BenchmarkProgressTaskStatus>, BenchError> {
    let mut statuses = BTreeMap::new();
    for task in &ledger.tasks {
        require_text(&task.task_id, "task_id")?;
        require_text(&task.next_action, "task next_action")?;
        if statuses.insert(task.task_id.as_str(), task.status).is_some() {
            return Err(invalid("benchmark progress ledger duplicates a task_id"));
        }
        validate_unique_text(&task.evidence, "task evidence")?;
        if task.status == BenchmarkProgressTaskStatus::Verified && task.evidence.is_empty() {
            return Err(invalid("verified benchmark progress task has no evidence"));
        }
    }
    Ok(statuses)
}

fn validate_execution_units(
    ledger: &BenchmarkProgressLedger,
) -> Result<BTreeMap<&str, BenchmarkProgressExecutionUnitStatus>, BenchError> {
    let mut statuses = BTreeMap::new();
    for unit in &ledger.execution_units {
        require_text(&unit.unit_id, "execution unit_id")?;
        require_text(&unit.run_id, "execution run_id")?;
        if statuses.insert(unit.unit_id.as_str(), unit.status).is_some() {
            return Err(invalid("benchmark progress ledger duplicates an execution unit_id"));
        }
        validate_unique_text(&unit.prerequisites, "execution prerequisites")?;
        validate_unique_text(&unit.attempt_ids, "execution attempt_ids")?;
        if unit.prerequisites.iter().any(|value| value == &unit.unit_id) {
            return Err(invalid("benchmark progress execution unit depends on itself"));
        }
    }
    Ok(statuses)
}

fn validate_attempts(ledger: &BenchmarkProgressLedger) -> Result<(), BenchError> {
    let mut attempts = BTreeMap::new();
    for attempt in &ledger.attempts {
        require_text(&attempt.attempt_id, "attempt_id")?;
        require_text(&attempt.run_id, "attempt run_id")?;
        validate_git_sha(&attempt.source_commit, "attempt source_commit")?;
        let key = (attempt.run_id.as_str(), attempt.attempt_id.as_str());
        if attempts.insert(key, attempt.status).is_some() {
            return Err(invalid("benchmark progress ledger duplicates an attempt identity"));
        }
        validate_attempt_state(attempt)?;
    }
    for unit in &ledger.execution_units {
        for attempt_id in &unit.attempt_ids {
            if !attempts.contains_key(&(unit.run_id.as_str(), attempt_id.as_str())) {
                return Err(invalid("execution unit references an unknown attempt"));
            }
        }
    }
    Ok(())
}

fn validate_attempt_state(
    attempt: &crate::model::BenchmarkProgressAttempt,
) -> Result<(), BenchError> {
    if matches!(
        attempt.status,
        BenchmarkProgressAttemptStatus::Submitted
            | BenchmarkProgressAttemptStatus::Pending
            | BenchmarkProgressAttemptStatus::Running
            | BenchmarkProgressAttemptStatus::Collectible
    ) && attempt.slurm_job_id.as_deref().is_none_or(str::is_empty)
    {
        return Err(invalid("scheduler-active progress attempt has no Slurm job identity"));
    }
    if let Some(job_id) = &attempt.slurm_job_id {
        if job_id.is_empty() || !job_id.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(invalid("benchmark progress attempt Slurm job identity is invalid"));
        }
    }
    if matches!(
        attempt.status,
        BenchmarkProgressAttemptStatus::Passed
            | BenchmarkProgressAttemptStatus::Failed
            | BenchmarkProgressAttemptStatus::Superseded
    ) && attempt.evidence_root.as_deref().is_none_or(str::is_empty)
    {
        return Err(invalid("terminal benchmark progress attempt has no evidence root"));
    }
    if attempt.status == BenchmarkProgressAttemptStatus::Failed
        && attempt.failure_class.as_deref().is_none_or(str::is_empty)
    {
        return Err(invalid("failed benchmark progress attempt has no failure class"));
    }
    Ok(())
}

fn validate_commits_and_checks(ledger: &BenchmarkProgressLedger) -> Result<(), BenchError> {
    let mut checks = BTreeSet::new();
    for check in &ledger.checks {
        require_text(&check.id, "check id")?;
        require_text(&check.log, "check log")?;
        if check.duration_seconds.is_some_and(|value| !value.is_finite() || value < 0.0) {
            return Err(invalid("benchmark progress check duration is invalid"));
        }
        if !checks.insert(check.id.as_str()) {
            return Err(invalid("benchmark progress ledger duplicates a check"));
        }
    }
    let mut commits = BTreeSet::new();
    for commit in &ledger.commits {
        validate_git_sha(&commit.sha, "commit sha")?;
        require_text(&commit.intent, "commit intent")?;
        validate_unique_text(&commit.checks, "commit checks")?;
        if !commits.insert(commit.sha.as_str()) {
            return Err(invalid("benchmark progress ledger duplicates a commit"));
        }
        if commit.checks.iter().any(|check| !checks.contains(check.as_str())) {
            return Err(invalid("benchmark progress commit references an unknown check"));
        }
    }
    Ok(())
}

fn validate_external_waits(ledger: &BenchmarkProgressLedger) -> Result<(), BenchError> {
    let mut waits = BTreeSet::new();
    for wait in &ledger.external_waits {
        require_text(&wait.id, "external wait id")?;
        require_text(&wait.reason, "external wait reason")?;
        if !wait.independent_work_continues {
            return Err(invalid("external wait stops independent progress"));
        }
        if !waits.insert(wait.id.as_str()) {
            return Err(invalid("benchmark progress ledger duplicates an external wait"));
        }
    }
    Ok(())
}

fn validate_frontier(
    ledger: &BenchmarkProgressLedger,
    task_statuses: &BTreeMap<&str, BenchmarkProgressTaskStatus>,
    unit_statuses: &BTreeMap<&str, BenchmarkProgressExecutionUnitStatus>,
) -> Result<(), BenchError> {
    validate_task_frontier(&ledger.frontier.ready_task_ids, task_statuses)?;
    let groups = [
        (
            ledger.frontier.ready_unit_ids.as_slice(),
            &[BenchmarkProgressExecutionUnitStatus::Ready][..],
            "ready",
        ),
        (
            ledger.frontier.submitted_unit_ids.as_slice(),
            &[
                BenchmarkProgressExecutionUnitStatus::Submitted,
                BenchmarkProgressExecutionUnitStatus::Running,
            ][..],
            "submitted",
        ),
        (
            ledger.frontier.collectible_unit_ids.as_slice(),
            &[BenchmarkProgressExecutionUnitStatus::Collectible][..],
            "collectible",
        ),
    ];
    let mut partition = BTreeSet::new();
    for (ids, allowed, label) in groups {
        validate_unit_frontier(ids, allowed, label, unit_statuses, &mut partition)?;
    }
    Ok(())
}

fn validate_unit_frontier<'a>(
    ids: &'a [String],
    allowed: &[BenchmarkProgressExecutionUnitStatus],
    label: &str,
    statuses: &BTreeMap<&str, BenchmarkProgressExecutionUnitStatus>,
    partition: &mut BTreeSet<&'a str>,
) -> Result<(), BenchError> {
    validate_unique_text(ids, label)?;
    for id in ids {
        let status = statuses
            .get(id.as_str())
            .ok_or_else(|| invalid(format!("{label} frontier references an unknown unit")))?;
        if !allowed.contains(status) || !partition.insert(id.as_str()) {
            return Err(invalid(format!("{label} frontier status or partition is invalid")));
        }
    }
    Ok(())
}

fn validate_task_frontier(
    ids: &[String],
    statuses: &BTreeMap<&str, BenchmarkProgressTaskStatus>,
) -> Result<(), BenchError> {
    validate_unique_text(ids, "ready task frontier")?;
    for id in ids {
        let status = statuses
            .get(id.as_str())
            .ok_or_else(|| invalid("ready task frontier references an unknown task"))?;
        if !matches!(
            status,
            BenchmarkProgressTaskStatus::Ready | BenchmarkProgressTaskStatus::InProgress
        ) {
            return Err(invalid("ready task frontier contains a non-ready task"));
        }
    }
    Ok(())
}

fn validate_unique_text(values: &[String], field: &str) -> Result<(), BenchError> {
    let mut unique = BTreeSet::new();
    for value in values {
        require_text(value, field)?;
        if !unique.insert(value.as_str()) {
            return Err(invalid(format!("benchmark progress ledger duplicates {field}")));
        }
    }
    Ok(())
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return Err(invalid(format!("benchmark progress ledger {field} is blank")));
    }
    Ok(())
}

fn validate_git_sha(value: &str, field: &str) -> Result<(), BenchError> {
    if value.len() != 40 || !value.bytes().all(is_lower_hex) {
        return Err(invalid(format!("benchmark progress ledger {field} is malformed")));
    }
    Ok(())
}

fn validate_sha256(value: &str, field: &str) -> Result<(), BenchError> {
    if value.len() != 64 || !value.bytes().all(is_lower_hex) {
        return Err(invalid(format!("benchmark progress ledger {field} is malformed")));
    }
    Ok(())
}

fn is_lower_hex(byte: u8) -> bool {
    byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)
}

fn invalid(message: impl Into<String>) -> BenchError {
    BenchError::InvalidPolicy(message.into())
}
