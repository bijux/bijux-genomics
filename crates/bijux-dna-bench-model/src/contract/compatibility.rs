//! Fail-closed readers and lossless upgrades for historical benchmark lifecycle records.

use std::collections::BTreeSet;

use chrono::DateTime;
use serde_json::Value;

use crate::diagnostics::BenchError;
use crate::model::{
    BenchmarkAttemptDocument, BenchmarkAttemptIdentity, BenchmarkAttemptRecord,
    BenchmarkAttemptRecordV2, BenchmarkAttemptTransition, BenchmarkAttemptV1MigrationContext,
    BenchmarkCycleDocument, BenchmarkCycleStatus, BenchmarkLifecycleStatus, BenchmarkRunIdentity,
};

use super::{
    validate_attempt_identity, validate_attempt_record_v2, validate_cycle_status_v2,
    BENCHMARK_ATTEMPT_SCHEMA_V1, BENCHMARK_ATTEMPT_SCHEMA_V2, BENCHMARK_CYCLE_STATUS_SCHEMA_V1,
    BENCHMARK_CYCLE_STATUS_SCHEMA_V2,
};

/// Parse and validate either supported persisted attempt version.
///
/// # Errors
/// Returns an error for malformed JSON, unknown schema versions, unknown fields or states, and
/// records that violate their version's invariants.
pub fn parse_benchmark_attempt_document(
    bytes: &[u8],
) -> Result<BenchmarkAttemptDocument, BenchError> {
    let value: Value = serde_json::from_slice(bytes)
        .map_err(|error| invalid(format!("cannot parse benchmark attempt JSON: {error}")))?;
    match schema_version(&value)? {
        BENCHMARK_ATTEMPT_SCHEMA_V1 => {
            let record: BenchmarkAttemptRecord = serde_json::from_value(value)
                .map_err(|error| invalid(format!("cannot parse benchmark attempt v1: {error}")))?;
            validate_attempt_record_v1(&record)?;
            Ok(BenchmarkAttemptDocument::V1(record))
        }
        BENCHMARK_ATTEMPT_SCHEMA_V2 => {
            let record: BenchmarkAttemptRecordV2 = serde_json::from_value(value)
                .map_err(|error| invalid(format!("cannot parse benchmark attempt v2: {error}")))?;
            validate_attempt_record_v2(&record)?;
            Ok(BenchmarkAttemptDocument::V2(record))
        }
        version => Err(invalid(format!("unsupported benchmark attempt schema: {version}"))),
    }
}

/// Parse and validate either supported persisted cycle-status version.
///
/// # Errors
/// Returns an error for malformed JSON, unknown schema versions, unknown fields or states, and
/// cycle records that violate their version's invariants.
pub fn parse_benchmark_cycle_document(bytes: &[u8]) -> Result<BenchmarkCycleDocument, BenchError> {
    let value: Value = serde_json::from_slice(bytes)
        .map_err(|error| invalid(format!("cannot parse benchmark cycle JSON: {error}")))?;
    match schema_version(&value)? {
        BENCHMARK_CYCLE_STATUS_SCHEMA_V1 => {
            let cycle: BenchmarkCycleStatus = serde_json::from_value(value)
                .map_err(|error| invalid(format!("cannot parse benchmark cycle v1: {error}")))?;
            validate_cycle_status_v1(&cycle)?;
            Ok(BenchmarkCycleDocument::V1(cycle))
        }
        BENCHMARK_CYCLE_STATUS_SCHEMA_V2 => {
            let cycle = serde_json::from_value(value)
                .map_err(|error| invalid(format!("cannot parse benchmark cycle v2: {error}")))?;
            validate_cycle_status_v2(&cycle)?;
            Ok(BenchmarkCycleDocument::V2(cycle))
        }
        version => Err(invalid(format!("unsupported benchmark cycle schema: {version}"))),
    }
}

/// Validate a historical flat attempt record without pretending it has v2 provenance.
///
/// # Errors
/// Returns an error when its schema, identity, state-specific reason, job, or evidence paths are
/// malformed.
pub fn validate_attempt_record_v1(record: &BenchmarkAttemptRecord) -> Result<(), BenchError> {
    if record.schema_version != BENCHMARK_ATTEMPT_SCHEMA_V1 {
        return Err(invalid("benchmark attempt v1 schema mismatch"));
    }
    let identity = legacy_identity(record)?;
    validate_attempt_identity(&identity)?;
    if record
        .supersedes_attempt_id
        .as_deref()
        .is_some_and(|attempt_id| attempt_id == record.attempt_id)
    {
        return Err(invalid("benchmark attempt cannot supersede itself"));
    }
    if let Some(attempt_id) = &record.supersedes_attempt_id {
        validate_legacy_attempt_id(attempt_id)?;
    }
    match (record.status, record.blocked_reason.as_deref()) {
        (BenchmarkLifecycleStatus::BlockedExternal, Some(reason)) if reason.trim().len() >= 10 => {}
        (BenchmarkLifecycleStatus::BlockedExternal, _) => {
            return Err(invalid("blocked benchmark attempt v1 requires a substantive reason"));
        }
        (_, None) => {}
        (_, Some(_)) => {
            return Err(invalid("nonblocked benchmark attempt v1 carries a blocked reason"));
        }
    }
    let mut jobs = BTreeSet::new();
    for job in &record.jobs {
        if job.scheduler != "slurm"
            || job.job_id.is_empty()
            || !job.job_id.bytes().all(|byte| byte.is_ascii_digit())
            || !jobs.insert((job.scheduler.as_str(), job.job_id.as_str()))
        {
            return Err(invalid("benchmark attempt v1 has malformed or duplicate Slurm jobs"));
        }
    }
    let mut paths = BTreeSet::new();
    for path in &record.evidence_paths {
        validate_study_uri(path)?;
        if !paths.insert(path.as_str()) {
            return Err(invalid("benchmark attempt v1 duplicates an evidence path"));
        }
    }
    Ok(())
}

/// Validate a historical cycle snapshot and all contained flat attempts.
///
/// # Errors
/// Returns an error when cycle identity, timestamp, attempts, or attempt uniqueness is invalid.
pub fn validate_cycle_status_v1(cycle: &BenchmarkCycleStatus) -> Result<(), BenchError> {
    if cycle.schema_version != BENCHMARK_CYCLE_STATUS_SCHEMA_V1 {
        return Err(invalid("benchmark cycle status v1 schema mismatch"));
    }
    validate_component(&cycle.campaign_id, "cycle campaign_id")?;
    validate_utc_timestamp(&cycle.generated_at, "cycle generated_at")?;
    if cycle.attempts.is_empty() {
        return Err(invalid("benchmark cycle v1 contains no attempts"));
    }
    let mut identities = BTreeSet::new();
    for attempt in &cycle.attempts {
        validate_attempt_record_v1(attempt)?;
        if attempt.campaign_id != cycle.campaign_id {
            return Err(invalid("benchmark attempt v1 campaign differs from its cycle"));
        }
        if !identities.insert((
            attempt.run_id.as_str(),
            attempt.stage_id.as_str(),
            attempt.tool_id.as_str(),
            attempt.attempt_id.as_str(),
        )) {
            return Err(invalid("benchmark cycle v1 duplicates an attempt identity"));
        }
    }
    Ok(())
}

/// Upgrade one historical flat attempt into an append-only v2 record.
///
/// The caller must provide evidence absent from v1. The migration fails rather than inventing
/// timestamps, terminal reasons, checksums, or scheduler accounting.
///
/// # Errors
/// Returns an error when the legacy record or migration evidence is invalid or cannot represent
/// the v1 state losslessly as a valid v2 transition chain.
pub fn migrate_benchmark_attempt_v1(
    record: &BenchmarkAttemptRecord,
    context: &BenchmarkAttemptV1MigrationContext,
) -> Result<BenchmarkAttemptRecordV2, BenchError> {
    validate_attempt_record_v1(record)?;
    let identity = legacy_identity(record)?;
    let supplied_paths =
        context.evidence.iter().map(|evidence| evidence.uri.as_str()).collect::<BTreeSet<_>>();
    let legacy_paths = record.evidence_paths.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if supplied_paths != legacy_paths {
        return Err(invalid(
            "benchmark attempt v1 migration evidence does not match legacy evidence paths",
        ));
    }

    let mut migrated = BenchmarkAttemptRecordV2::planned(identity, context.observed_at_utc.clone());
    migrated.supersedes_attempt_id.clone_from(&record.supersedes_attempt_id);
    if record.status == BenchmarkLifecycleStatus::Planned {
        require_unused_context(context)?;
        validate_attempt_record_v2(&migrated)?;
        return Ok(migrated);
    }

    push_transition(&mut migrated, BenchmarkLifecycleStatus::Ready, context, None, false)?;
    if record.status == BenchmarkLifecycleStatus::Ready {
        require_unused_context(context)?;
        validate_attempt_record_v2(&migrated)?;
        return Ok(migrated);
    }

    migrate_after_ready(record, context, &mut migrated)?;
    validate_attempt_record_v2(&migrated)?;
    Ok(migrated)
}

fn migrate_after_ready(
    record: &BenchmarkAttemptRecord,
    context: &BenchmarkAttemptV1MigrationContext,
    migrated: &mut BenchmarkAttemptRecordV2,
) -> Result<(), BenchError> {
    match record.status {
        BenchmarkLifecycleStatus::BlockedExternal | BenchmarkLifecycleStatus::Superseded => {
            migrated.transitions.pop();
            let reason = if record.status == BenchmarkLifecycleStatus::BlockedExternal {
                if context.terminal_reason.is_some() {
                    return Err(invalid(
                        "benchmark attempt v1 blocked migration must use the retained blocked reason",
                    ));
                }
                record.blocked_reason.clone()
            } else {
                context.terminal_reason.clone()
            };
            push_transition(migrated, record.status, context, reason, true)?;
        }
        BenchmarkLifecycleStatus::Submitted
        | BenchmarkLifecycleStatus::Running
        | BenchmarkLifecycleStatus::Collecting
        | BenchmarkLifecycleStatus::Passed
        | BenchmarkLifecycleStatus::Failed
        | BenchmarkLifecycleStatus::Cancelled => {
            if record.jobs.is_empty() {
                return Err(invalid(
                    "benchmark attempt v1 submitted or terminal state cannot migrate without a Slurm job",
                ));
            }
            push_transition(migrated, BenchmarkLifecycleStatus::Submitted, context, None, false)?;
            migrated
                .transitions
                .last_mut()
                .ok_or_else(|| invalid("benchmark migration lost submitted transition"))?
                .jobs
                .clone_from(&record.jobs);
            if record.status == BenchmarkLifecycleStatus::Submitted {
                require_unused_terminal_context(context)?;
                attach_evidence_to_last(migrated, context)?;
            } else if matches!(
                record.status,
                BenchmarkLifecycleStatus::Failed | BenchmarkLifecycleStatus::Cancelled
            ) {
                push_transition(
                    migrated,
                    record.status,
                    context,
                    context.terminal_reason.clone(),
                    true,
                )?;
            } else {
                push_transition(migrated, BenchmarkLifecycleStatus::Running, context, None, false)?;
                if record.status == BenchmarkLifecycleStatus::Running {
                    require_unused_terminal_context(context)?;
                    attach_evidence_to_last(migrated, context)?;
                } else {
                    if context.terminal_reason.is_some() {
                        return Err(invalid(
                            "benchmark attempt v1 successful migration supplied a failure reason",
                        ));
                    }
                    push_transition(
                        migrated,
                        BenchmarkLifecycleStatus::Collecting,
                        context,
                        None,
                        true,
                    )?;
                    if record.status == BenchmarkLifecycleStatus::Passed {
                        push_transition(
                            migrated,
                            BenchmarkLifecycleStatus::Passed,
                            context,
                            None,
                            false,
                        )?;
                    }
                }
            }
        }
        BenchmarkLifecycleStatus::Planned | BenchmarkLifecycleStatus::Ready => unreachable!(),
    }
    Ok(())
}

fn attach_evidence_to_last(
    record: &mut BenchmarkAttemptRecordV2,
    context: &BenchmarkAttemptV1MigrationContext,
) -> Result<(), BenchError> {
    record
        .transitions
        .last_mut()
        .ok_or_else(|| invalid("benchmark migration lost its current transition"))?
        .evidence
        .clone_from(&context.evidence);
    Ok(())
}

fn legacy_identity(
    record: &BenchmarkAttemptRecord,
) -> Result<BenchmarkAttemptIdentity, BenchError> {
    let (domain, _) = record
        .stage_id
        .split_once('.')
        .ok_or_else(|| invalid("benchmark attempt v1 stage identity lacks its domain"))?;
    Ok(BenchmarkAttemptIdentity {
        campaign_id: record.campaign_id.clone(),
        run: BenchmarkRunIdentity {
            run_id: record.run_id.clone(),
            domain: domain.to_string(),
            stage_id: record.stage_id.clone(),
        },
        case_id: None,
        attempt_id: record.attempt_id.clone(),
        tool_id: record.tool_id.clone(),
    })
}

fn push_transition(
    record: &mut BenchmarkAttemptRecordV2,
    status: BenchmarkLifecycleStatus,
    context: &BenchmarkAttemptV1MigrationContext,
    reason: Option<String>,
    attach_provenance: bool,
) -> Result<(), BenchError> {
    let previous = record.current_status();
    let sequence = u32::try_from(record.transitions.len() + 1)
        .map_err(|_| invalid("benchmark migration transition sequence overflow"))?;
    record.transitions.push(BenchmarkAttemptTransition {
        sequence,
        from_status: previous,
        to_status: status,
        observed_at_utc: context.observed_at_utc.clone(),
        reason,
        jobs: Vec::new(),
        accounting: if attach_provenance { context.accounting.clone() } else { Vec::new() },
        evidence: if attach_provenance { context.evidence.clone() } else { Vec::new() },
    });
    Ok(())
}

fn require_unused_context(context: &BenchmarkAttemptV1MigrationContext) -> Result<(), BenchError> {
    require_unused_terminal_context(context)?;
    if !context.evidence.is_empty() {
        return Err(invalid(
            "benchmark attempt v1 migration supplied evidence for an unexecuted state",
        ));
    }
    Ok(())
}

fn require_unused_terminal_context(
    context: &BenchmarkAttemptV1MigrationContext,
) -> Result<(), BenchError> {
    if !context.accounting.is_empty() || context.terminal_reason.is_some() {
        return Err(invalid(
            "benchmark attempt v1 migration supplied terminal context for a nonterminal state",
        ));
    }
    Ok(())
}

fn schema_version(value: &Value) -> Result<&str, BenchError> {
    value
        .as_object()
        .and_then(|object| object.get("schema_version"))
        .and_then(Value::as_str)
        .ok_or_else(|| invalid("benchmark document requires a string schema_version"))
}

fn validate_legacy_attempt_id(value: &str) -> Result<(), BenchError> {
    let Some(number) = value.strip_prefix("attempt-") else {
        return Err(invalid("benchmark attempt v1 supersession identity is malformed"));
    };
    if number.is_empty() || !number.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(invalid("benchmark attempt v1 supersession identity is malformed"));
    }
    Ok(())
}

fn validate_component(value: &str, field: &str) -> Result<(), BenchError> {
    if value.is_empty()
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
    {
        return Err(invalid(format!("benchmark {field} is not a normalized component")));
    }
    Ok(())
}

fn validate_utc_timestamp(value: &str, field: &str) -> Result<(), BenchError> {
    let timestamp = DateTime::parse_from_rfc3339(value)
        .map_err(|_| invalid(format!("benchmark {field} is not RFC 3339")))?;
    if timestamp.offset().local_minus_utc() != 0 {
        return Err(invalid(format!("benchmark {field} is not UTC")));
    }
    Ok(())
}

fn validate_study_uri(uri: &str) -> Result<(), BenchError> {
    let Some(relative) = uri.strip_prefix("study://") else {
        return Err(invalid("benchmark attempt v1 evidence must use a study URI"));
    };
    if relative.is_empty()
        || relative.starts_with('/')
        || relative.ends_with('/')
        || relative.contains(['\\', '?', '#', '%'])
        || relative.split('/').any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        return Err(invalid("benchmark attempt v1 evidence study URI is not normalized"));
    }
    Ok(())
}

fn invalid(message: impl Into<String>) -> BenchError {
    BenchError::InvalidPolicy(message.into())
}
