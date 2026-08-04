//! Owner: bijux-dna-runtime
//! Persistent append-only benchmark attempt lifecycle storage.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    append_attempt_transition, benchmark_run_directory_name, validate_attempt_identity,
    validate_attempt_record_v2, BenchmarkAttemptIdentity, BenchmarkAttemptRecordV2,
    BenchmarkAttemptTransition, BenchmarkEvidenceReference, BenchmarkJobReference,
    BenchmarkLifecycleStatus, BenchmarkSchedulerAccounting, BenchmarkSchedulerTerminalState,
};

const ATTEMPT_FILE: &str = "attempt.json";
const TRANSITIONS_DIR: &str = "transitions";
const MAX_LIFECYCLE_FILE_BYTES: usize = 1024 * 1024;
const LIFECYCLE_LOCK_TIMEOUT: Duration = Duration::from_secs(30);

/// Derive the owned directory for one permanent benchmark attempt.
///
/// # Errors
/// Returns an error when any identity component is invalid.
pub fn benchmark_attempt_directory(
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
) -> Result<PathBuf> {
    validate_attempt_identity(identity).context("validate benchmark attempt identity")?;
    let run_directory =
        benchmark_run_directory_name(&identity.run).context("derive benchmark run directory")?;
    Ok(benchmark_root
        .join(run_directory)
        .join(&identity.tool_id)
        .join("attempts")
        .join(&identity.campaign_id)
        .join(&identity.attempt_id))
}

/// Initialize a permanent attempt with its immutable planned event.
///
/// # Errors
/// Returns an error when the record is invalid, contains later transitions, or its attempt
/// directory already exists.
pub fn initialize_attempt_lifecycle(
    benchmark_root: &Path,
    record: &BenchmarkAttemptRecordV2,
) -> Result<PathBuf> {
    validate_attempt_record_v2(record).context("validate initial benchmark attempt")?;
    if record.transitions.len() != 1
        || record.current_status() != Some(BenchmarkLifecycleStatus::Planned)
    {
        bail!("benchmark attempt initialization requires exactly one planned transition");
    }
    validate_supersession_target(benchmark_root, record)?;
    let _lock = acquire_attempt_lock(benchmark_root, &record.identity)?;
    let attempt_directory = benchmark_attempt_directory(benchmark_root, &record.identity)?;
    if attempt_directory.exists() {
        bail!("benchmark attempt directory already exists: {}", attempt_directory.display());
    }
    bijux_dna_infra::atomic_create_json(&attempt_directory.join(ATTEMPT_FILE), record)
        .with_context(|| format!("create benchmark attempt {}", attempt_directory.display()))?;
    Ok(attempt_directory)
}

/// Reconstruct and validate an attempt from its immutable initial record and transition events.
///
/// # Errors
/// Returns an error for missing, malformed, unsafe, discontinuous, or identity-drifting evidence.
pub fn load_attempt_lifecycle(
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
) -> Result<BenchmarkAttemptRecordV2> {
    let _lock = acquire_attempt_lock(benchmark_root, identity)?;
    load_attempt_lifecycle_unlocked(benchmark_root, identity)
}

fn load_attempt_lifecycle_unlocked(
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
) -> Result<BenchmarkAttemptRecordV2> {
    let attempt_directory = benchmark_attempt_directory(benchmark_root, identity)?;
    validate_attempt_directory_shape(&attempt_directory)?;
    let initial_path = attempt_directory.join(ATTEMPT_FILE);
    let initial_bytes =
        bijux_dna_infra::read_to_end_bounded(&initial_path, MAX_LIFECYCLE_FILE_BYTES)
            .with_context(|| format!("read benchmark attempt {}", initial_path.display()))?;
    let mut record: BenchmarkAttemptRecordV2 = serde_json::from_slice(&initial_bytes)
        .with_context(|| format!("parse benchmark attempt {}", initial_path.display()))?;
    if &record.identity != identity {
        bail!("benchmark attempt path identity differs from its initial record");
    }
    if record.transitions.len() != 1
        || record.current_status() != Some(BenchmarkLifecycleStatus::Planned)
    {
        bail!("benchmark attempt initial record is not an immutable planned event");
    }
    validate_attempt_record_v2(&record).context("validate initial benchmark attempt record")?;

    for (sequence, path) in transition_files(&attempt_directory.join(TRANSITIONS_DIR))? {
        let bytes = bijux_dna_infra::read_to_end_bounded(&path, MAX_LIFECYCLE_FILE_BYTES)
            .with_context(|| format!("read benchmark transition {}", path.display()))?;
        let transition: BenchmarkAttemptTransition = serde_json::from_slice(&bytes)
            .with_context(|| format!("parse benchmark transition {}", path.display()))?;
        if transition.sequence != sequence {
            bail!("benchmark transition filename differs from its sequence");
        }
        append_attempt_transition(&mut record, transition)
            .context("reconstruct benchmark attempt transition chain")?;
    }
    validate_attempt_record_v2(&record).context("validate reconstructed benchmark attempt")?;
    Ok(record)
}

/// Atomically append one sequence-addressed immutable transition and return the reconstructed record.
///
/// # Errors
/// Returns an error when stored evidence is invalid, the transition is illegal, or its sequence
/// file already exists.
pub fn append_attempt_lifecycle_transition(
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
    transition: BenchmarkAttemptTransition,
) -> Result<BenchmarkAttemptRecordV2> {
    let _lock = acquire_attempt_lock(benchmark_root, identity)?;
    append_attempt_lifecycle_transition_unlocked(benchmark_root, identity, transition)
}

fn append_attempt_lifecycle_transition_unlocked(
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
    transition: BenchmarkAttemptTransition,
) -> Result<BenchmarkAttemptRecordV2> {
    let mut candidate = load_attempt_lifecycle_unlocked(benchmark_root, identity)?;
    append_attempt_transition(&mut candidate, transition.clone())
        .context("validate appended benchmark transition")?;
    let attempt_directory = benchmark_attempt_directory(benchmark_root, identity)?;
    let event_path =
        attempt_directory.join(TRANSITIONS_DIR).join(format!("{:08}.json", transition.sequence));
    bijux_dna_infra::atomic_create_json(&event_path, &transition)
        .with_context(|| format!("append benchmark transition {}", event_path.display()))?;

    let reconstructed = load_attempt_lifecycle_unlocked(benchmark_root, identity)?;
    if reconstructed != candidate {
        bail!("persisted benchmark attempt differs from the validated transition candidate");
    }
    Ok(reconstructed)
}

/// Record that planning and admission have made an attempt ready for scheduler submission.
///
/// Repeating the operation after the same ready event was recorded is idempotent.
///
/// # Errors
/// Returns an error when the stored lifecycle or derived transition is invalid.
pub fn record_attempt_ready(
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
    observed_at_utc: &str,
) -> Result<BenchmarkAttemptRecordV2> {
    apply_derived_transition(benchmark_root, identity, |record| {
        if has_status(record, BenchmarkLifecycleStatus::Ready) {
            return Ok(None);
        }
        Ok(Some(next_transition(record, BenchmarkLifecycleStatus::Ready, observed_at_utc)?))
    })
}

/// Retain the exact job identities returned by scheduler submission.
///
/// Repeating the operation with the same identities is idempotent. A different identity set is
/// refused instead of replacing prior scheduler evidence.
///
/// # Errors
/// Returns an error when jobs are empty, conflict with stored evidence, or produce an invalid
/// lifecycle transition.
pub fn record_attempt_submission(
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
    observed_at_utc: &str,
    jobs: Vec<BenchmarkJobReference>,
) -> Result<BenchmarkAttemptRecordV2> {
    apply_derived_transition(benchmark_root, identity, |record| {
        if jobs.is_empty() {
            bail!("benchmark submission requires at least one scheduler job");
        }
        let retained = retained_jobs(record);
        if !retained.is_empty() {
            if retained == jobs {
                return Ok(None);
            }
            bail!("benchmark submission conflicts with retained scheduler jobs");
        }
        let mut transition =
            next_transition(record, BenchmarkLifecycleStatus::Submitted, observed_at_utc)?;
        transition.jobs = jobs;
        Ok(Some(transition))
    })
}

/// Record one observed running state for an already retained scheduler job.
///
/// # Errors
/// Returns an error when the job does not belong to the attempt or the transition is invalid.
pub fn record_attempt_running(
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
    observed_at_utc: &str,
    job: &BenchmarkJobReference,
) -> Result<BenchmarkAttemptRecordV2> {
    apply_derived_transition(benchmark_root, identity, |record| {
        if has_status(record, BenchmarkLifecycleStatus::Running) {
            return Ok(None);
        }
        if !retained_jobs(record).contains(job) {
            bail!("benchmark running observation has no retained scheduler job");
        }
        Ok(Some(next_transition(record, BenchmarkLifecycleStatus::Running, observed_at_utc)?))
    })
}

/// Derive collecting or failed state from terminal scheduler accounting.
///
/// Completed accounting enters collection. Every other terminal state fails the attempt while
/// preserving raw state, exit data, resource accounting, node identity, and scheduler reason.
/// Repeating collection with byte-equivalent accounting is idempotent.
///
/// # Errors
/// Returns an error when accounting conflicts with retained evidence or the lifecycle transition is
/// invalid.
pub fn record_attempt_accounting(
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
    observed_at_utc: &str,
    accounting: BenchmarkSchedulerAccounting,
) -> Result<BenchmarkAttemptRecordV2> {
    apply_derived_transition(benchmark_root, identity, |record| {
        for retained in retained_accounting(record) {
            if retained.scheduler == accounting.scheduler && retained.job_id == accounting.job_id {
                if retained == &accounting {
                    return Ok(None);
                }
                bail!("benchmark scheduler accounting conflicts with retained evidence");
            }
        }
        if !retained_jobs(record)
            .iter()
            .any(|job| job.scheduler == accounting.scheduler && job.job_id == accounting.job_id)
        {
            bail!("benchmark scheduler accounting has no retained job identity");
        }
        let status = if accounting.state == BenchmarkSchedulerTerminalState::Completed {
            BenchmarkLifecycleStatus::Collecting
        } else {
            BenchmarkLifecycleStatus::Failed
        };
        let mut transition = next_transition(record, status, observed_at_utc)?;
        if status == BenchmarkLifecycleStatus::Failed {
            transition.reason = Some(accounting.reason.clone().unwrap_or_else(|| {
                format!("scheduler reported terminal state {}", accounting.raw_state)
            }));
        }
        transition.accounting.push(accounting);
        Ok(Some(transition))
    })
}

/// Record checksummed scientific and downstream evidence that admits a collected attempt.
///
/// Repeating acceptance with the same evidence is idempotent.
///
/// # Errors
/// Returns an error when evidence is empty, conflicts with a passed attempt, or is invalid.
pub fn record_attempt_acceptance(
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
    observed_at_utc: &str,
    evidence: Vec<BenchmarkEvidenceReference>,
) -> Result<BenchmarkAttemptRecordV2> {
    apply_derived_transition(benchmark_root, identity, |record| {
        if evidence.is_empty() {
            bail!("benchmark acceptance requires checksummed evidence");
        }
        if record.current_status() == Some(BenchmarkLifecycleStatus::Passed) {
            if retained_evidence(record) == evidence {
                return Ok(None);
            }
            bail!("benchmark acceptance conflicts with retained evidence");
        }
        let mut transition =
            next_transition(record, BenchmarkLifecycleStatus::Passed, observed_at_utc)?;
        transition.evidence = evidence;
        Ok(Some(transition))
    })
}

/// Record an external dependency block without rewriting prior attempt evidence.
///
/// # Errors
/// Returns an error when the reason or transition is invalid.
pub fn record_attempt_blocked_external(
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
    observed_at_utc: &str,
    reason: &str,
) -> Result<BenchmarkAttemptRecordV2> {
    record_reasoned_terminal(
        benchmark_root,
        identity,
        observed_at_utc,
        reason,
        BenchmarkLifecycleStatus::BlockedExternal,
    )
}

/// Record that an immutable attempt has been superseded by operator decision.
///
/// # Errors
/// Returns an error when the reason or transition is invalid.
pub fn record_attempt_superseded(
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
    observed_at_utc: &str,
    reason: &str,
) -> Result<BenchmarkAttemptRecordV2> {
    record_reasoned_terminal(
        benchmark_root,
        identity,
        observed_at_utc,
        reason,
        BenchmarkLifecycleStatus::Superseded,
    )
}

fn record_reasoned_terminal(
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
    observed_at_utc: &str,
    reason: &str,
    status: BenchmarkLifecycleStatus,
) -> Result<BenchmarkAttemptRecordV2> {
    apply_derived_transition(benchmark_root, identity, |record| {
        if record.current_status() == Some(status) {
            let retained_reason =
                record.transitions.last().and_then(|event| event.reason.as_deref());
            if retained_reason == Some(reason) {
                return Ok(None);
            }
            bail!("benchmark terminal decision conflicts with retained reason");
        }
        let mut transition = next_transition(record, status, observed_at_utc)?;
        transition.reason = Some(reason.to_string());
        Ok(Some(transition))
    })
}

fn apply_derived_transition(
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
    derive: impl FnOnce(&BenchmarkAttemptRecordV2) -> Result<Option<BenchmarkAttemptTransition>>,
) -> Result<BenchmarkAttemptRecordV2> {
    let _lock = acquire_attempt_lock(benchmark_root, identity)?;
    let record = load_attempt_lifecycle_unlocked(benchmark_root, identity)?;
    let Some(transition) = derive(&record)? else {
        return Ok(record);
    };
    append_attempt_lifecycle_transition_unlocked(benchmark_root, identity, transition)
}

fn next_transition(
    record: &BenchmarkAttemptRecordV2,
    to_status: BenchmarkLifecycleStatus,
    observed_at_utc: &str,
) -> Result<BenchmarkAttemptTransition> {
    let sequence = u32::try_from(record.transitions.len())
        .ok()
        .and_then(|value| value.checked_add(1))
        .context("benchmark transition sequence overflow")?;
    Ok(BenchmarkAttemptTransition {
        sequence,
        from_status: record.current_status(),
        to_status,
        observed_at_utc: observed_at_utc.to_string(),
        reason: None,
        jobs: Vec::new(),
        accounting: Vec::new(),
        evidence: Vec::new(),
    })
}

fn has_status(record: &BenchmarkAttemptRecordV2, status: BenchmarkLifecycleStatus) -> bool {
    record.transitions.iter().any(|transition| transition.to_status == status)
}

fn retained_jobs(record: &BenchmarkAttemptRecordV2) -> Vec<BenchmarkJobReference> {
    record.transitions.iter().flat_map(|transition| transition.jobs.clone()).collect()
}

fn retained_accounting(
    record: &BenchmarkAttemptRecordV2,
) -> impl Iterator<Item = &BenchmarkSchedulerAccounting> {
    record.transitions.iter().flat_map(|transition| &transition.accounting)
}

fn retained_evidence(record: &BenchmarkAttemptRecordV2) -> Vec<BenchmarkEvidenceReference> {
    record.transitions.iter().flat_map(|transition| transition.evidence.clone()).collect()
}

fn validate_supersession_target(
    benchmark_root: &Path,
    record: &BenchmarkAttemptRecordV2,
) -> Result<()> {
    let Some(prior_attempt_id) = record.supersedes_attempt_id.as_deref() else {
        return Ok(());
    };
    let mut prior_identity = record.identity.clone();
    prior_identity.attempt_id = prior_attempt_id.to_string();
    let prior = load_attempt_lifecycle(benchmark_root, &prior_identity)
        .context("load superseded benchmark attempt")?;
    if !prior.current_status().is_some_and(is_terminal_status) {
        bail!("benchmark retry can only supersede a terminal attempt");
    }
    Ok(())
}

fn is_terminal_status(status: BenchmarkLifecycleStatus) -> bool {
    matches!(
        status,
        BenchmarkLifecycleStatus::Passed
            | BenchmarkLifecycleStatus::Failed
            | BenchmarkLifecycleStatus::Superseded
            | BenchmarkLifecycleStatus::BlockedExternal
    )
}

fn acquire_attempt_lock(
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
) -> Result<bijux_dna_infra::FileLock> {
    validate_attempt_identity(identity).context("validate benchmark attempt lock identity")?;
    let run_directory =
        benchmark_run_directory_name(&identity.run).context("derive benchmark lock directory")?;
    let lock_path = benchmark_root
        .join(".lifecycle-locks")
        .join(run_directory)
        .join(&identity.tool_id)
        .join(&identity.campaign_id)
        .join(format!("{}.lock", identity.attempt_id));
    bijux_dna_infra::FileLock::acquire(&lock_path, LIFECYCLE_LOCK_TIMEOUT)
        .with_context(|| format!("acquire benchmark attempt lock {}", lock_path.display()))
}

fn validate_attempt_directory_shape(attempt_directory: &Path) -> Result<()> {
    for entry in fs::read_dir(attempt_directory).with_context(|| {
        format!("read benchmark attempt directory {}", attempt_directory.display())
    })? {
        let entry = entry.with_context(|| {
            format!("read benchmark attempt entry in {}", attempt_directory.display())
        })?;
        let name = entry.file_name();
        let file_type = entry.file_type().with_context(|| {
            format!("inspect benchmark attempt entry {}", entry.path().display())
        })?;
        if name == ATTEMPT_FILE {
            if !file_type.is_file() || file_type.is_symlink() {
                bail!("benchmark initial attempt record is not a regular file");
            }
        } else if name == TRANSITIONS_DIR {
            if !file_type.is_dir() || file_type.is_symlink() {
                bail!("benchmark transition evidence is not a regular directory");
            }
        } else {
            bail!("benchmark attempt directory contains an unmanaged entry");
        }
    }
    Ok(())
}

fn transition_files(directory: &Path) -> Result<Vec<(u32, PathBuf)>> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error).context(format!("read {}", directory.display())),
    };
    let mut files = Vec::new();
    for entry in entries {
        let entry = entry.with_context(|| format!("read entry in {}", directory.display()))?;
        let file_type = entry
            .file_type()
            .with_context(|| format!("inspect benchmark transition {}", entry.path().display()))?;
        if !file_type.is_file() || file_type.is_symlink() {
            bail!("benchmark transition evidence contains a non-regular file");
        }
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| anyhow::anyhow!("benchmark transition filename is not UTF-8"))?;
        let sequence = parse_transition_filename(&name)?;
        files.push((sequence, entry.path()));
    }
    files.sort_by_key(|(sequence, _)| *sequence);
    Ok(files)
}

fn parse_transition_filename(name: &str) -> Result<u32> {
    let Some(sequence) = name.strip_suffix(".json") else {
        bail!("benchmark transition filename must end in .json");
    };
    if sequence.len() != 8 || !sequence.bytes().all(|byte| byte.is_ascii_digit()) {
        bail!("benchmark transition filename must use eight sequence digits");
    }
    sequence.parse().context("parse benchmark transition sequence")
}
