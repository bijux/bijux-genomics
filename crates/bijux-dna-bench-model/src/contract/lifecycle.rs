//! Validation for benchmark attempt lifecycle records.

use std::collections::BTreeSet;

use bijux_dna_core::contract::canonical::to_canonical_json_bytes;
use chrono::{DateTime, FixedOffset};
use sha2::{Digest, Sha256};

use crate::diagnostics::BenchError;
use crate::model::{
    BenchmarkAttemptArchive, BenchmarkAttemptDisposition, BenchmarkAttemptRecordV2,
    BenchmarkAttemptTransition, BenchmarkCampaignPlan, BenchmarkCycleAdmission,
    BenchmarkCycleAdmissionState, BenchmarkCyclePreparation, BenchmarkCyclePreparationState,
    BenchmarkCycleStatusV2, BenchmarkEvidenceReference, BenchmarkJobReference,
    BenchmarkLifecycleStatus, BenchmarkRunIdentity, BenchmarkSchedulerAccounting,
    BenchmarkSchedulerEvidenceSource, BenchmarkSchedulerObservation,
    BenchmarkSchedulerObservedState, BenchmarkSchedulerTerminalState,
};

use super::{
    BENCHMARK_ATTEMPT_ARCHIVE_SCHEMA_V1, BENCHMARK_ATTEMPT_SCHEMA_V2,
    BENCHMARK_CAMPAIGN_PLAN_SCHEMA_V1, BENCHMARK_CYCLE_ADMISSION_SCHEMA_V1,
    BENCHMARK_CYCLE_PREPARATION_SCHEMA_V1, BENCHMARK_CYCLE_STATUS_SCHEMA_V2,
    BENCHMARK_SCHEDULER_OBSERVATION_SCHEMA_V1,
};

/// Derive the permanent `run-<id>-<domain>-<stage>` benchmark directory name.
///
/// # Errors
/// Returns an error when run, domain, or stage identity is malformed or inconsistent.
pub fn benchmark_run_directory_name(identity: &BenchmarkRunIdentity) -> Result<String, BenchError> {
    validate_run_identity(identity)?;
    Ok(format!("{}-{}", identity.run_id, slug(&identity.stage_id)))
}

/// Validate the complete permanent identity of one benchmark attempt.
///
/// # Errors
/// Returns an error when campaign, run, attempt, or tool identity is malformed.
pub fn validate_attempt_identity(
    identity: &crate::model::BenchmarkAttemptIdentity,
) -> Result<(), BenchError> {
    validate_component(&identity.campaign_id, "campaign_id")?;
    validate_run_identity(&identity.run)?;
    validate_attempt_id(&identity.attempt_id)?;
    validate_component(&identity.tool_id, "tool_id")
}

/// Return the canonical SHA-256 identity of one valid benchmark attempt.
///
/// # Errors
/// Returns an error when the identity is invalid or cannot be canonically serialized.
pub fn benchmark_attempt_identity_sha256(
    identity: &crate::model::BenchmarkAttemptIdentity,
) -> Result<String, BenchError> {
    validate_attempt_identity(identity)?;
    let bytes = to_canonical_json_bytes(identity).map_err(|error| {
        invalid(format!("cannot canonicalize benchmark attempt identity: {error}"))
    })?;
    Ok(sha256_hex(&bytes))
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
    validate_attempt_identity(&record.identity)?;
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
    let mut accounting_jobs = BTreeSet::new();
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
        for accounting in &transition.accounting {
            validate_scheduler_accounting(accounting, transition.to_status)?;
            if !jobs.contains(&(accounting.scheduler.as_str(), accounting.job_id.as_str())) {
                return Err(invalid("benchmark accounting has no declared scheduler job"));
            }
            if !accounting_jobs.insert((accounting.scheduler.as_str(), accounting.job_id.as_str()))
            {
                return Err(invalid("benchmark attempt duplicates scheduler accounting"));
            }
        }
        for reference in &transition.evidence {
            validate_evidence_reference(reference)?;
            if !evidence.insert(reference.uri.as_str()) {
                return Err(invalid("benchmark attempt duplicates an evidence URI"));
            }
        }
        previous_status = Some(transition.to_status);
    }

    let status = previous_status.ok_or_else(|| invalid("benchmark attempt has no status"))?;
    if requires_scheduler_job(status) && jobs.is_empty() {
        return Err(invalid("benchmark lifecycle status requires a Slurm job identity"));
    }
    let has_completed_accounting = record
        .transitions
        .iter()
        .flat_map(|event| &event.accounting)
        .any(|accounting| accounting.state == BenchmarkSchedulerTerminalState::Completed);
    if status == BenchmarkLifecycleStatus::Collecting && !has_completed_accounting {
        return Err(invalid("collecting benchmark attempt requires completed accounting"));
    }
    if matches!(status, BenchmarkLifecycleStatus::Failed | BenchmarkLifecycleStatus::Cancelled)
        && !jobs.is_empty()
        && accounting_jobs.is_empty()
    {
        return Err(invalid(
            "unsuccessful submitted benchmark attempt requires terminal accounting",
        ));
    }
    if status == BenchmarkLifecycleStatus::Passed
        && (evidence.is_empty() || !has_completed_accounting)
    {
        return Err(invalid(
            "passed benchmark attempt requires completed accounting and checksummed evidence",
        ));
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

    let mut attempt_identities = BTreeSet::new();
    let mut active = BTreeSet::new();
    for attempt in &cycle.attempts {
        validate_attempt_record_v2(attempt)?;
        if attempt.identity.campaign_id != cycle.campaign_id {
            return Err(invalid("benchmark attempt campaign differs from cycle campaign"));
        }
        if !attempt_identities.insert((
            attempt.identity.run.run_id.as_str(),
            attempt.identity.run.domain.as_str(),
            attempt.identity.run.stage_id.as_str(),
            attempt.identity.tool_id.as_str(),
            attempt.identity.attempt_id.as_str(),
        )) {
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
            .find(|candidate| {
                candidate.identity.attempt_id == superseded_id
                    && candidate.identity.run == attempt.identity.run
                    && candidate.identity.tool_id == attempt.identity.tool_id
            })
            .ok_or_else(|| invalid("benchmark attempt supersedes an unknown attempt"))?;
        if !superseded.current_status().is_some_and(is_terminal) {
            return Err(invalid("benchmark attempt supersedes a nonterminal attempt"));
        }
    }
    Ok(())
}

/// Validate one complete non-barrier benchmark preparation pass.
///
/// # Errors
/// Returns an error for schema, campaign, timestamp, identity, execution-spec, or state drift.
pub fn validate_cycle_preparation(
    preparation: &BenchmarkCyclePreparation,
) -> Result<(), BenchError> {
    if preparation.schema_version != BENCHMARK_CYCLE_PREPARATION_SCHEMA_V1 {
        return Err(invalid("benchmark cycle preparation schema mismatch"));
    }
    validate_component(&preparation.campaign_id, "preparation campaign_id")?;
    let pass_prepared_at = parse_timestamp(&preparation.prepared_at_utc, "prepared_at_utc")?;
    let mut identities = BTreeSet::new();
    for unit in &preparation.units {
        validate_attempt_identity(&unit.identity)?;
        if unit.identity.campaign_id != preparation.campaign_id {
            return Err(invalid("preparation unit campaign differs from preparation campaign"));
        }
        if !identities.insert((
            unit.identity.run.run_id.as_str(),
            unit.identity.tool_id.as_str(),
            unit.identity.attempt_id.as_str(),
        )) {
            return Err(invalid("benchmark preparation duplicates an attempt identity"));
        }
        if let Some(spec) = &unit.execution_spec {
            super::execution::validate_execution_spec(spec)?;
            let spec_prepared_at = parse_timestamp(&spec.prepared_at_utc, "spec prepared_at_utc")?;
            if spec.identity != unit.identity {
                return Err(invalid("preparation execution spec differs from its unit identity"));
            }
            if spec_prepared_at > pass_prepared_at {
                return Err(invalid("execution spec was prepared after its preparation pass"));
            }
        }
        let valid = match unit.state {
            BenchmarkCyclePreparationState::Prepared => {
                unit.prior_status == Some(BenchmarkLifecycleStatus::Planned)
                    && unit.current_status == Some(BenchmarkLifecycleStatus::Ready)
                    && unit.execution_spec.is_some()
                    && unit.error.is_none()
            }
            BenchmarkCyclePreparationState::AlreadyPrepared => {
                unit.prior_status == Some(BenchmarkLifecycleStatus::Ready)
                    && unit.current_status == Some(BenchmarkLifecycleStatus::Ready)
                    && unit.execution_spec.is_some()
                    && unit.error.is_none()
            }
            BenchmarkCyclePreparationState::NotReady => {
                unit.current_status == unit.prior_status
                    && unit.execution_spec.is_none()
                    && unit.error.is_none()
            }
            BenchmarkCyclePreparationState::Failed => {
                unit.execution_spec.is_none()
                    && unit.error.as_deref().is_some_and(|value| !value.trim().is_empty())
            }
        };
        if !valid {
            return Err(invalid("benchmark preparation unit state is inconsistent"));
        }
    }
    Ok(())
}

/// Validate one complete non-barrier benchmark evidence-admission pass.
///
/// # Errors
/// Returns an error for schema, campaign, timestamp, identity, evidence, or state drift.
pub fn validate_cycle_admission(admission: &BenchmarkCycleAdmission) -> Result<(), BenchError> {
    if admission.schema_version != BENCHMARK_CYCLE_ADMISSION_SCHEMA_V1 {
        return Err(invalid("benchmark cycle admission schema mismatch"));
    }
    validate_component(&admission.campaign_id, "admission campaign_id")?;
    parse_timestamp(&admission.admitted_at_utc, "admitted_at_utc")?;
    let mut identities = BTreeSet::new();
    for unit in &admission.units {
        validate_attempt_identity(&unit.identity)?;
        if unit.identity.campaign_id != admission.campaign_id {
            return Err(invalid("admission unit campaign differs from admission campaign"));
        }
        if !identities.insert((
            unit.identity.run.run_id.as_str(),
            unit.identity.tool_id.as_str(),
            unit.identity.attempt_id.as_str(),
        )) {
            return Err(invalid("benchmark admission duplicates an attempt identity"));
        }
        if let Some(evidence) = &unit.evidence {
            validate_evidence_reference(evidence)?;
        }
        let valid = match unit.state {
            BenchmarkCycleAdmissionState::Admitted => {
                unit.prior_status == Some(BenchmarkLifecycleStatus::Collecting)
                    && unit.current_status == Some(BenchmarkLifecycleStatus::Passed)
                    && unit.evidence.is_some()
                    && unit.error.is_none()
            }
            BenchmarkCycleAdmissionState::AlreadyAdmitted => {
                unit.prior_status == Some(BenchmarkLifecycleStatus::Passed)
                    && unit.current_status == Some(BenchmarkLifecycleStatus::Passed)
                    && unit.evidence.is_some()
                    && unit.error.is_none()
            }
            BenchmarkCycleAdmissionState::NotReady => {
                unit.current_status == unit.prior_status
                    && unit.evidence.is_none()
                    && unit.error.is_none()
            }
            BenchmarkCycleAdmissionState::Failed => {
                unit.evidence.is_none()
                    && unit.error.as_deref().is_some_and(|value| !value.trim().is_empty())
            }
        };
        if !valid {
            return Err(invalid("benchmark admission unit state is inconsistent"));
        }
    }
    Ok(())
}

/// Validate a source-locked campaign plan and its complete planned cycle.
///
/// # Errors
/// Returns an error when schema, source commit, registry digest, or cycle evidence is invalid.
pub fn validate_campaign_plan(plan: &BenchmarkCampaignPlan) -> Result<(), BenchError> {
    if plan.schema_version != BENCHMARK_CAMPAIGN_PLAN_SCHEMA_V1 {
        return Err(invalid("benchmark campaign plan schema mismatch"));
    }
    if plan.repository_commit.len() != 40
        || !plan.repository_commit.bytes().all(|byte| byte.is_ascii_hexdigit())
        || plan.repository_commit.bytes().any(|byte| byte.is_ascii_uppercase())
        || plan.repository_commit.bytes().all(|byte| byte == b'0')
    {
        return Err(invalid("benchmark campaign plan commit identity is malformed"));
    }
    if plan.campaign_registry_sha256.len() != 64
        || !plan.campaign_registry_sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
        || plan.campaign_registry_sha256.bytes().any(|byte| byte.is_ascii_uppercase())
        || plan.campaign_registry_sha256.bytes().all(|byte| byte == b'0')
    {
        return Err(invalid("benchmark campaign registry digest is malformed"));
    }
    validate_cycle_status_v2(&plan.cycle)
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
            | BenchmarkLifecycleStatus::Cancelled
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
    for accounting in &transition.accounting {
        validate_scheduler_accounting(accounting, transition.to_status)?;
    }
    for reference in &transition.evidence {
        validate_evidence_reference(reference)?;
    }
    Ok(())
}

/// Validate one immutable scheduler observation independent of its attempt path.
///
/// # Errors
/// Returns an error when identity, sequence, timestamp, source, state, or accounting disagree.
pub fn validate_scheduler_observation(
    record: &BenchmarkSchedulerObservation,
) -> Result<(), BenchError> {
    if record.schema_version != BENCHMARK_SCHEDULER_OBSERVATION_SCHEMA_V1 {
        return Err(invalid("benchmark scheduler observation schema mismatch"));
    }
    if record.sequence == 0 {
        return Err(invalid("benchmark scheduler observation sequence starts at one"));
    }
    let observation = &record.observation;
    validate_job(&BenchmarkJobReference {
        scheduler: observation.scheduler.clone(),
        job_id: observation.job_id.clone(),
    })?;
    parse_timestamp(&observation.observed_at_utc, "scheduler observed_at_utc")?;
    match (observation.source, observation.state, &observation.accounting) {
        (
            BenchmarkSchedulerEvidenceSource::Submission,
            BenchmarkSchedulerObservedState::Submitted,
            None,
        )
        | (
            BenchmarkSchedulerEvidenceSource::Queue,
            BenchmarkSchedulerObservedState::Submitted
            | BenchmarkSchedulerObservedState::Pending
            | BenchmarkSchedulerObservedState::Running,
            None,
        )
        | (
            BenchmarkSchedulerEvidenceSource::Accounting,
            BenchmarkSchedulerObservedState::AccountingPending,
            None,
        ) => Ok(()),
        (BenchmarkSchedulerEvidenceSource::Accounting, state, Some(accounting))
            if state.is_terminal() =>
        {
            validate_scheduler_accounting_shape(accounting)?;
            if accounting.scheduler != observation.scheduler
                || accounting.job_id != observation.job_id
                || observed_terminal_state(accounting.state) != state
            {
                return Err(invalid(
                    "benchmark scheduler observation accounting identity or state drifted",
                ));
            }
            Ok(())
        }
        _ => Err(invalid("benchmark scheduler observation source, state, and accounting disagree")),
    }
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

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut hex, "{byte:02x}");
    }
    hex
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
                | BenchmarkLifecycleStatus::Collecting
                | BenchmarkLifecycleStatus::Failed
                | BenchmarkLifecycleStatus::Cancelled
                | BenchmarkLifecycleStatus::BlockedExternal
                | BenchmarkLifecycleStatus::Superseded
        ) | (
            BenchmarkLifecycleStatus::Running,
            BenchmarkLifecycleStatus::Collecting
                | BenchmarkLifecycleStatus::Failed
                | BenchmarkLifecycleStatus::Cancelled
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

fn validate_scheduler_accounting(
    accounting: &BenchmarkSchedulerAccounting,
    transition_status: BenchmarkLifecycleStatus,
) -> Result<(), BenchError> {
    validate_scheduler_accounting_shape(accounting)?;
    match accounting.state {
        BenchmarkSchedulerTerminalState::Completed
            if matches!(
                transition_status,
                BenchmarkLifecycleStatus::Collecting | BenchmarkLifecycleStatus::Passed
            ) && accounting.exit_code == 0
                && accounting.signal == 0 => {}
        BenchmarkSchedulerTerminalState::Cancelled
            if transition_status == BenchmarkLifecycleStatus::Cancelled => {}
        BenchmarkSchedulerTerminalState::Failed
        | BenchmarkSchedulerTerminalState::Timeout
        | BenchmarkSchedulerTerminalState::Preempted
        | BenchmarkSchedulerTerminalState::NodeFailure
        | BenchmarkSchedulerTerminalState::OutOfMemory
            if transition_status == BenchmarkLifecycleStatus::Failed => {}
        _ => return Err(invalid("benchmark scheduler accounting contradicts lifecycle status")),
    }
    Ok(())
}

fn validate_scheduler_accounting_shape(
    accounting: &BenchmarkSchedulerAccounting,
) -> Result<(), BenchError> {
    validate_job(&BenchmarkJobReference {
        scheduler: accounting.scheduler.clone(),
        job_id: accounting.job_id.clone(),
    })?;
    if accounting.raw_state.trim().is_empty()
        || accounting.raw_state != accounting.raw_state.trim()
        || accounting.node_list.trim().is_empty()
        || accounting.node_list != accounting.node_list.trim()
        || accounting.exit_code < 0
        || accounting.signal < 0
        || accounting
            .reason
            .as_deref()
            .is_some_and(|reason| reason.trim().is_empty() || reason != reason.trim())
    {
        return Err(invalid("benchmark scheduler accounting is malformed"));
    }
    if accounting.state == BenchmarkSchedulerTerminalState::Completed
        && (accounting.exit_code != 0 || accounting.signal != 0)
    {
        return Err(invalid("completed benchmark scheduler accounting has a nonzero exit"));
    }
    Ok(())
}

fn observed_terminal_state(
    state: BenchmarkSchedulerTerminalState,
) -> BenchmarkSchedulerObservedState {
    match state {
        BenchmarkSchedulerTerminalState::Completed => BenchmarkSchedulerObservedState::Completed,
        BenchmarkSchedulerTerminalState::Failed => BenchmarkSchedulerObservedState::Failed,
        BenchmarkSchedulerTerminalState::Cancelled => BenchmarkSchedulerObservedState::Cancelled,
        BenchmarkSchedulerTerminalState::Timeout => BenchmarkSchedulerObservedState::Timeout,
        BenchmarkSchedulerTerminalState::Preempted => BenchmarkSchedulerObservedState::Preempted,
        BenchmarkSchedulerTerminalState::NodeFailure => {
            BenchmarkSchedulerObservedState::NodeFailure
        }
        BenchmarkSchedulerTerminalState::OutOfMemory => {
            BenchmarkSchedulerObservedState::OutOfMemory
        }
    }
}

fn validate_evidence_reference(reference: &BenchmarkEvidenceReference) -> Result<(), BenchError> {
    let Some(relative) = reference.uri.strip_prefix("study://") else {
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
    validate_digest(&reference.sha256)?;
    Ok(())
}

fn requires_scheduler_job(status: BenchmarkLifecycleStatus) -> bool {
    matches!(
        status,
        BenchmarkLifecycleStatus::Submitted
            | BenchmarkLifecycleStatus::Running
            | BenchmarkLifecycleStatus::Collecting
            | BenchmarkLifecycleStatus::Cancelled
            | BenchmarkLifecycleStatus::Passed
    )
}

fn is_terminal(status: BenchmarkLifecycleStatus) -> bool {
    matches!(
        status,
        BenchmarkLifecycleStatus::Passed
            | BenchmarkLifecycleStatus::Failed
            | BenchmarkLifecycleStatus::Cancelled
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
        || value.bytes().all(|byte| byte == b'0')
    {
        return Err(invalid("benchmark SHA-256 identity is malformed"));
    }
    Ok(())
}

fn invalid(message: impl Into<String>) -> BenchError {
    BenchError::InvalidPolicy(message.into())
}
