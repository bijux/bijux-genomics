//! Validation for benchmark attempt lifecycle records.

use std::collections::BTreeSet;

use chrono::{DateTime, FixedOffset};

use crate::diagnostics::BenchError;
use crate::model::{
    BenchmarkAttemptArchive, BenchmarkAttemptDisposition, BenchmarkAttemptRecordV2,
    BenchmarkAttemptTransition, BenchmarkCycleStatusV2, BenchmarkJobReference,
    BenchmarkLifecycleStatus, BenchmarkRunIdentity,
};

use super::{
    BENCHMARK_ATTEMPT_ARCHIVE_SCHEMA_V1, BENCHMARK_ATTEMPT_SCHEMA_V2,
    BENCHMARK_CYCLE_STATUS_SCHEMA_V2,
};

/// Derive the permanent `run-<id>-<domain>-<stage>` benchmark directory name.
///
/// # Errors
/// Returns an error when run, domain, or stage identity is malformed or inconsistent.
pub fn benchmark_run_directory_name(identity: &BenchmarkRunIdentity) -> Result<String, BenchError> {
    validate_run_identity(identity)?;
    Ok(format!("{}-{}", identity.run_id, slug(&identity.stage_id)))
}

/// Append one validated lifecycle event without rewriting prior transitions.
///
/// # Errors
/// Returns an error when the existing record is invalid or the transition violates sequence,
/// identity, evidence, scheduler, timestamp, or state-machine requirements.
pub fn append_attempt_transition(
    record: &mut BenchmarkAttemptRecordV2,
    transition: BenchmarkAttemptTransition,
) -> Result<(), BenchError> {
    validate_attempt_record_v2(record)?;
    let mut candidate = record.clone();
    candidate.transitions.push(transition);
    validate_attempt_record_v2(&candidate)?;
    *record = candidate;
    Ok(())
}

/// Validate an immutable v2 attempt record and its complete append-only transition chain.
///
/// # Errors
/// Returns an error when identity, lineage, transition order, scheduler jobs, evidence, or terminal
/// state is malformed.
pub fn validate_attempt_record_v2(record: &BenchmarkAttemptRecordV2) -> Result<(), BenchError> {
    if record.schema_version != BENCHMARK_ATTEMPT_SCHEMA_V2 {
        return Err(invalid("benchmark attempt v2 schema mismatch"));
    }
    validate_component(&record.identity.campaign_id, "campaign_id")?;
    validate_run_identity(&record.identity.run)?;
    validate_attempt_id(&record.identity.attempt_id)?;
    validate_component(&record.identity.tool_id, "tool_id")?;
    if record
        .supersedes_attempt_id
        .as_deref()
        .is_some_and(|attempt| attempt == record.identity.attempt_id)
    {
        return Err(invalid("benchmark attempt cannot supersede itself"));
    }
    if let Some(attempt_id) = &record.supersedes_attempt_id {
        validate_attempt_id(attempt_id)?;
    }
    if record.transitions.is_empty() {
        return Err(invalid("benchmark attempt has no lifecycle transitions"));
    }

    let mut previous_status = None;
    let mut previous_time = None;
    let mut jobs = BTreeSet::new();
    let mut evidence = BTreeSet::new();
    for (index, transition) in record.transitions.iter().enumerate() {
        let expected_sequence = u32::try_from(index + 1)
            .map_err(|_| invalid("benchmark transition sequence overflow"))?;
        if transition.sequence != expected_sequence || transition.from_status != previous_status {
            return Err(invalid("benchmark transition sequence or prior status is discontinuous"));
        }
        validate_attempt_transition(transition)?;
        if let Some(from_status) = previous_status {
            if !allowed_transition(from_status, transition.to_status) {
                return Err(invalid("benchmark lifecycle transition is not allowed"));
            }
        } else if transition.to_status != BenchmarkLifecycleStatus::Planned {
            return Err(invalid("benchmark attempt must begin in planned status"));
        }
        let observed_at = parse_timestamp(&transition.observed_at_utc, "observed_at_utc")?;
        if previous_time.is_some_and(|prior| observed_at < prior) {
            return Err(invalid("benchmark transition timestamps are not monotonic"));
        }
        previous_time = Some(observed_at);
        for job in &transition.jobs {
            validate_job(job)?;
            if !jobs.insert((job.scheduler.as_str(), job.job_id.as_str())) {
                return Err(invalid("benchmark attempt duplicates a scheduler job"));
            }
        }
        for path in &transition.evidence_paths {
            validate_evidence_path(path)?;
            if !evidence.insert(path.as_str()) {
                return Err(invalid("benchmark attempt duplicates an evidence path"));
            }
        }
        previous_status = Some(transition.to_status);
    }

    let status = previous_status.ok_or_else(|| invalid("benchmark attempt has no status"))?;
    if requires_scheduler_job(status) && jobs.is_empty() {
        return Err(invalid("benchmark lifecycle status requires a Slurm job identity"));
    }
    if status == BenchmarkLifecycleStatus::Passed && evidence.is_empty() {
        return Err(invalid("passed benchmark attempt has no evidence path"));
    }
    Ok(())
}

/// Validate a v2 cycle snapshot while preserving multiple immutable attempts per tool and run.
///
/// # Errors
/// Returns an error for invalid cycle identity, attempts, lineage, duplicate attempts, or multiple
/// active attempts for the same run and tool.
pub fn validate_cycle_status_v2(cycle: &BenchmarkCycleStatusV2) -> Result<(), BenchError> {
    if cycle.schema_version != BENCHMARK_CYCLE_STATUS_SCHEMA_V2 {
        return Err(invalid("benchmark cycle status v2 schema mismatch"));
    }
    validate_component(&cycle.campaign_id, "cycle campaign_id")?;
    let generated_at = parse_timestamp(&cycle.generated_at_utc, "generated_at_utc")?;
    if cycle.attempts.is_empty() {
        return Err(invalid("benchmark cycle contains no attempts"));
    }

    let mut attempt_ids = BTreeSet::new();
    let mut active = BTreeSet::new();
    for attempt in &cycle.attempts {
        validate_attempt_record_v2(attempt)?;
        if attempt.identity.campaign_id != cycle.campaign_id {
            return Err(invalid("benchmark attempt campaign differs from cycle campaign"));
        }
        if !attempt_ids.insert(attempt.identity.attempt_id.as_str()) {
            return Err(invalid("benchmark cycle duplicates an attempt identity"));
        }
        let last = attempt
            .transitions
            .last()
            .ok_or_else(|| invalid("benchmark attempt has no terminal transition"))?;
        let observed_at = parse_timestamp(&last.observed_at_utc, "observed_at_utc")?;
        if observed_at > generated_at {
            return Err(invalid("benchmark cycle predates an attempt transition"));
        }
        if !is_terminal(last.to_status)
            && !active
                .insert((attempt.identity.run.run_id.as_str(), attempt.identity.tool_id.as_str()))
        {
            return Err(invalid(
                "benchmark cycle has multiple active attempts for one run and tool",
            ));
        }
    }
    for attempt in &cycle.attempts {
        let Some(superseded_id) = attempt.supersedes_attempt_id.as_deref() else {
            continue;
        };
        let superseded = cycle
            .attempts
            .iter()
            .find(|candidate| candidate.identity.attempt_id == superseded_id)
            .ok_or_else(|| invalid("benchmark attempt supersedes an unknown attempt"))?;
        if superseded.identity.run != attempt.identity.run
            || superseded.identity.tool_id != attempt.identity.tool_id
        {
            return Err(invalid("benchmark attempt supersession crosses run or tool identity"));
        }
        if !superseded.current_status().is_some_and(is_terminal) {
            return Err(invalid("benchmark attempt supersedes a nonterminal attempt"));
        }
    }
    Ok(())
}

/// Validate one transition independent of its surrounding chain.
///
/// # Errors
/// Returns an error when sequence, timestamp, reason, scheduler job, or evidence fields are invalid.
pub fn validate_attempt_transition(
    transition: &BenchmarkAttemptTransition,
) -> Result<(), BenchError> {
    if transition.sequence == 0 {
        return Err(invalid("benchmark transition sequence starts at one"));
    }
    parse_timestamp(&transition.observed_at_utc, "observed_at_utc")?;
    let reason_required = matches!(
        transition.to_status,
        BenchmarkLifecycleStatus::Failed
            | BenchmarkLifecycleStatus::Superseded
            | BenchmarkLifecycleStatus::BlockedExternal
    );
    match (reason_required, transition.reason.as_deref()) {
        (true, Some(reason)) if reason.trim().len() >= 10 => {}
        (true, _) => return Err(invalid("benchmark terminal failure status requires a reason")),
        (false, None) => {}
        (false, Some(_)) => {
            return Err(invalid("benchmark nonfailure transition carries a reason"))
        }
    }
    for job in &transition.jobs {
        validate_job(job)?;
    }
    for path in &transition.evidence_paths {
        validate_evidence_path(path)?;
    }
    Ok(())
}

fn validate_run_identity(identity: &BenchmarkRunIdentity) -> Result<(), BenchError> {
    let Some(run_number) = identity.run_id.strip_prefix("run-") else {
        return Err(invalid("benchmark run identity must use run-<id>"));
    };
    if run_number.len() != 4 {
        return Err(invalid("benchmark run identity must use four canonical digits"));
    }
    let run_number = run_number
        .parse::<u16>()
        .map_err(|_| invalid("benchmark run identity has a nonnumeric id"))?;
    if !(2000..3000).contains(&run_number) {
        return Err(invalid("benchmark run identity is outside run-2000..2999"));
    }
    if !matches!(identity.domain.as_str(), "bam" | "fastq" | "vcf") {
        return Err(invalid("benchmark run identity has an unsupported domain"));
    }
    let Some((domain, stage)) = identity.stage_id.split_once('.') else {
        return Err(invalid("benchmark stage identity must include its domain"));
    };
    if domain != identity.domain || stage.contains('.') || !is_snake_component(stage) {
        return Err(invalid("benchmark stage identity is malformed or differs from its domain"));
    }
    Ok(())
}

fn validate_attempt_id(attempt_id: &str) -> Result<(), BenchError> {
    let Some(number) = attempt_id.strip_prefix("attempt-") else {
        return Err(invalid("benchmark attempt identity must use attempt-<id>"));
    };
    if number.is_empty() || !number.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(invalid("benchmark attempt identity has a nonnumeric id"));
    }
    Ok(())
}

fn validate_component(value: &str, field: &str) -> Result<(), BenchError> {
    if value.is_empty()
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
        || !value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        || !value
            .bytes()
            .last()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    {
        return Err(invalid(format!("benchmark {field} is not a normalized component")));
    }
    Ok(())
}

fn is_snake_component(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && value
            .bytes()
            .last()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && !value.contains("__")
}

fn slug(stage_id: &str) -> String {
    stage_id.replace(['.', '_'], "-")
}

fn parse_timestamp(value: &str, field: &str) -> Result<DateTime<FixedOffset>, BenchError> {
    let timestamp = DateTime::parse_from_rfc3339(value)
        .map_err(|_| invalid(format!("benchmark {field} is not RFC 3339")))?;
    if timestamp.offset().local_minus_utc() != 0 {
        return Err(invalid(format!("benchmark {field} is not UTC")));
    }
    Ok(timestamp)
}

fn allowed_transition(from: BenchmarkLifecycleStatus, to: BenchmarkLifecycleStatus) -> bool {
    matches!(
        (from, to),
        (
            BenchmarkLifecycleStatus::Planned,
            BenchmarkLifecycleStatus::Ready
                | BenchmarkLifecycleStatus::BlockedExternal
                | BenchmarkLifecycleStatus::Superseded
        ) | (
            BenchmarkLifecycleStatus::Ready,
            BenchmarkLifecycleStatus::Submitted
                | BenchmarkLifecycleStatus::BlockedExternal
                | BenchmarkLifecycleStatus::Superseded
        ) | (
            BenchmarkLifecycleStatus::Submitted,
            BenchmarkLifecycleStatus::Running
                | BenchmarkLifecycleStatus::Failed
                | BenchmarkLifecycleStatus::BlockedExternal
                | BenchmarkLifecycleStatus::Superseded
        ) | (
            BenchmarkLifecycleStatus::Running,
            BenchmarkLifecycleStatus::Collecting
                | BenchmarkLifecycleStatus::Failed
                | BenchmarkLifecycleStatus::BlockedExternal
                | BenchmarkLifecycleStatus::Superseded
        ) | (
            BenchmarkLifecycleStatus::Collecting,
            BenchmarkLifecycleStatus::Passed
                | BenchmarkLifecycleStatus::Failed
                | BenchmarkLifecycleStatus::BlockedExternal
                | BenchmarkLifecycleStatus::Superseded
        )
    )
}

fn validate_job(job: &BenchmarkJobReference) -> Result<(), BenchError> {
    if job.scheduler != "slurm"
        || job.job_id.is_empty()
        || !job.job_id.bytes().all(|byte| byte.is_ascii_digit())
        || job.job_id.bytes().all(|byte| byte == b'0')
    {
        return Err(invalid("benchmark scheduler job identity is not a valid Slurm job"));
    }
    Ok(())
}

fn validate_evidence_path(path: &str) -> Result<(), BenchError> {
    let Some(relative) = path.strip_prefix("study://") else {
        return Err(invalid("benchmark evidence path must use the study URI scheme"));
    };
    if relative.is_empty()
        || relative.starts_with('/')
        || relative.ends_with('/')
        || relative.contains(['\\', '?', '#', '%'])
        || relative.split('/').any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        return Err(invalid("benchmark evidence study URI is not normalized"));
    }
    Ok(())
}

fn requires_scheduler_job(status: BenchmarkLifecycleStatus) -> bool {
    matches!(
        status,
        BenchmarkLifecycleStatus::Submitted
            | BenchmarkLifecycleStatus::Running
            | BenchmarkLifecycleStatus::Collecting
            | BenchmarkLifecycleStatus::Passed
    )
}

fn is_terminal(status: BenchmarkLifecycleStatus) -> bool {
    matches!(
        status,
        BenchmarkLifecycleStatus::Passed
            | BenchmarkLifecycleStatus::Failed
            | BenchmarkLifecycleStatus::Superseded
            | BenchmarkLifecycleStatus::BlockedExternal
    )
}

/// Validate a benchmark-owned terminal attempt archive.
///
/// # Errors
/// Returns an error when identity, disposition, source, or file evidence is incomplete.
pub fn validate_attempt_archive(archive: &BenchmarkAttemptArchive) -> Result<(), BenchError> {
    if archive.schema_version != BENCHMARK_ATTEMPT_ARCHIVE_SCHEMA_V1 {
        return Err(invalid("benchmark attempt archive schema mismatch"));
    }
    for (value, field) in [
        (&archive.run_id, "run_id"),
        (&archive.attempt_id, "attempt_id"),
        (&archive.tool_id, "tool_id"),
        (&archive.slurm_job_id, "slurm_job_id"),
        (&archive.archived_at_utc, "archived_at_utc"),
    ] {
        require_text(value, field)?;
    }
    if !archive.slurm_job_id.bytes().all(|byte| byte.is_ascii_digit())
        || archive.attempt_id != format!("slurm-{}", archive.slurm_job_id)
    {
        return Err(invalid("benchmark attempt archive scheduler identity is invalid"));
    }
    match (&archive.disposition, archive.reason.as_deref()) {
        (BenchmarkAttemptDisposition::Failed, None) => {}
        (BenchmarkAttemptDisposition::Superseded, Some(reason)) if reason.trim().len() >= 20 => {}
        (BenchmarkAttemptDisposition::Failed, Some(_)) => {
            return Err(invalid("failed benchmark attempt must not claim a supersession reason"));
        }
        (BenchmarkAttemptDisposition::Superseded, _) => {
            return Err(invalid("superseded benchmark attempt requires a substantive reason"));
        }
    }
    for uri in [archive.source_paths.submission.as_str(), archive.source_paths.result.as_str()] {
        if !uri.starts_with("study://") || uri.contains("..") {
            return Err(invalid("benchmark attempt archive source URI is invalid"));
        }
    }
    for digest in [&archive.evidence_sha256, &archive.accounting_sha256] {
        validate_digest(digest)?;
    }
    if archive.files.is_empty() {
        return Err(invalid("benchmark attempt archive contains no file inventory"));
    }
    let mut paths = BTreeSet::new();
    for file in &archive.files {
        if !(file.path.starts_with("submission/") || file.path.starts_with("result/"))
            || file.path.contains("..")
            || !paths.insert(file.path.as_str())
        {
            return Err(invalid("benchmark attempt archive file path is invalid or duplicated"));
        }
        validate_digest(&file.sha256)?;
    }
    Ok(())
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return Err(invalid(format!("benchmark attempt archive {field} is blank")));
    }
    Ok(())
}

fn validate_digest(value: &str) -> Result<(), BenchError> {
    if value.len() != 64
        || !value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(invalid("benchmark attempt archive SHA-256 identity is malformed"));
    }
    Ok(())
}

fn invalid(message: impl Into<String>) -> BenchError {
    BenchError::InvalidPolicy(message.into())
}
