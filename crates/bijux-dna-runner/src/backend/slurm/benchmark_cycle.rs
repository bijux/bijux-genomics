//! One-shot scheduler collection for benchmark campaign attempts.

use std::cmp::Ordering;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_campaign_plan, validate_execution_spec, validate_utc_timestamp,
    BenchmarkAttemptIdentity, BenchmarkCampaignPlan, BenchmarkExecutionSpec,
    BenchmarkLifecycleStatus, BenchmarkSchedulerAccounting, BenchmarkSchedulerEvidenceSource,
    BenchmarkSchedulerObservationDraft, BenchmarkSchedulerObservedState,
    BenchmarkSchedulerTerminalState,
};
use serde::{Deserialize, Serialize};

use super::{
    collect_slurm_job, load_benchmark_slurm_job_handle, prepare_benchmark_slurm_job,
    submit_prepared_benchmark_job, SlurmAccounting, SlurmClientPaths, SlurmEvidenceSource,
    SlurmJobObservation, SlurmJobState,
};

const MAX_EXECUTION_SPEC_BYTES: usize = 16 * 1024 * 1024;

/// Result for one attempt inspected during a campaign collection pass.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCampaignCollectionUnit {
    pub identity: BenchmarkAttemptIdentity,
    pub prior_status: BenchmarkLifecycleStatus,
    pub current_status: BenchmarkLifecycleStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scheduler_observation: Option<SlurmJobObservation>,
}

/// Complete result of one bounded, non-waiting benchmark scheduler collection pass.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCampaignCollection {
    pub campaign_id: String,
    pub observed_at_utc: String,
    pub units: Vec<BenchmarkCampaignCollectionUnit>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkCampaignSubmissionState {
    Submitted,
    AlreadyActive,
    LimitReached,
    NotReady,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCampaignSubmissionUnit {
    pub identity: BenchmarkAttemptIdentity,
    pub prior_status: BenchmarkLifecycleStatus,
    pub current_status: BenchmarkLifecycleStatus,
    pub state: BenchmarkCampaignSubmissionState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub job_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCampaignSubmission {
    pub campaign_id: String,
    pub submitted_at_utc: String,
    pub maximum_submissions: usize,
    pub units: Vec<BenchmarkCampaignSubmissionUnit>,
}

/// Submit at most `maximum_submissions` independently ready attempts and return immediately.
///
/// Scheduler effects are crash-bounded by persisted submission intents. Attempt-local failures do
/// not prevent independent siblings from being considered in the same pass.
///
/// # Errors
/// Returns an error when shared plan, timestamp, bound, or evidence-root authority is invalid.
pub fn submit_benchmark_campaign_frontier(
    paths: &SlurmClientPaths,
    benchmark_root: &Path,
    plan: &BenchmarkCampaignPlan,
    submitted_at_utc: &str,
    maximum_submissions: usize,
) -> Result<BenchmarkCampaignSubmission> {
    validate_campaign_plan(plan).context("validate benchmark campaign submission plan")?;
    validate_utc_timestamp(submitted_at_utc, "benchmark submission submitted_at_utc")?;
    if maximum_submissions == 0 {
        bail!("benchmark submission maximum must be greater than zero");
    }
    let benchmark_root = benchmark_root.canonicalize()?;
    let mut attempts = plan.cycle.attempts.iter().collect::<Vec<_>>();
    attempts.sort_by(|left, right| compare_attempt_identity(&left.identity, &right.identity));
    let mut effects_attempted = 0usize;
    let mut units = Vec::with_capacity(attempts.len());
    for attempt in attempts {
        let identity = &attempt.identity;
        let lifecycle = bijux_dna_runtime::load_attempt_lifecycle(&benchmark_root, identity)?;
        let prior_status = lifecycle
            .current_status()
            .context("benchmark submission attempt has no lifecycle status")?;
        if matches!(
            prior_status,
            BenchmarkLifecycleStatus::Submitted | BenchmarkLifecycleStatus::Running
        ) {
            units.push(submission_unit(
                identity,
                prior_status,
                prior_status,
                BenchmarkCampaignSubmissionState::AlreadyActive,
                None,
                None,
            ));
            continue;
        }
        if prior_status != BenchmarkLifecycleStatus::Ready {
            units.push(submission_unit(
                identity,
                prior_status,
                prior_status,
                BenchmarkCampaignSubmissionState::NotReady,
                None,
                None,
            ));
            continue;
        }
        if effects_attempted == maximum_submissions {
            units.push(submission_unit(
                identity,
                prior_status,
                prior_status,
                BenchmarkCampaignSubmissionState::LimitReached,
                None,
                None,
            ));
            continue;
        }
        effects_attempted += 1;
        let result = (|| -> Result<String> {
            let spec = load_execution_spec(&benchmark_root, identity)?;
            let prepared = prepare_benchmark_slurm_job(&spec, submitted_at_utc)?;
            let handle = submit_prepared_benchmark_job(paths, &prepared)?;
            bijux_dna_runtime::record_attempt_submission(
                &benchmark_root,
                identity,
                submitted_at_utc,
                vec![bijux_dna_bench_model::BenchmarkJobReference {
                    scheduler: "slurm".to_string(),
                    job_id: handle.job_id.clone(),
                }],
            )?;
            Ok(handle.job_id)
        })();
        match result {
            Ok(job_id) => units.push(submission_unit(
                identity,
                prior_status,
                BenchmarkLifecycleStatus::Submitted,
                BenchmarkCampaignSubmissionState::Submitted,
                Some(job_id),
                None,
            )),
            Err(error) => {
                let current_status =
                    bijux_dna_runtime::load_attempt_lifecycle(&benchmark_root, identity)
                        .ok()
                        .and_then(|value| value.current_status())
                        .unwrap_or(prior_status);
                units.push(submission_unit(
                    identity,
                    prior_status,
                    current_status,
                    BenchmarkCampaignSubmissionState::Failed,
                    None,
                    Some(format!("{error:#}")),
                ));
            }
        }
    }
    Ok(BenchmarkCampaignSubmission {
        campaign_id: plan.cycle.campaign_id.clone(),
        submitted_at_utc: submitted_at_utc.to_string(),
        maximum_submissions,
        units,
    })
}

fn compare_attempt_identity(
    left: &BenchmarkAttemptIdentity,
    right: &BenchmarkAttemptIdentity,
) -> Ordering {
    (left.case_id.as_deref(), left.tool_id.as_str())
        .cmp(&(right.case_id.as_deref(), right.tool_id.as_str()))
}

fn submission_unit(
    identity: &BenchmarkAttemptIdentity,
    prior_status: BenchmarkLifecycleStatus,
    current_status: BenchmarkLifecycleStatus,
    state: BenchmarkCampaignSubmissionState,
    job_id: Option<String>,
    error: Option<String>,
) -> BenchmarkCampaignSubmissionUnit {
    BenchmarkCampaignSubmissionUnit {
        identity: identity.clone(),
        prior_status,
        current_status,
        state,
        job_id,
        error,
    }
}

/// Observe every scheduler-active attempt at most once and persist accounting-ready transitions.
///
/// Attempts that are not submitted or running are read once and returned without a scheduler
/// query. The function never waits for a job, child process, output file, or report.
///
/// # Errors
/// Returns an error when the plan or evidence root is invalid, scheduler evidence is malformed, or
/// an immutable lifecycle transition conflicts with retained evidence.
pub fn collect_benchmark_campaign_cycle(
    paths: &SlurmClientPaths,
    benchmark_root: &Path,
    plan: &BenchmarkCampaignPlan,
    observed_at_utc: &str,
) -> Result<BenchmarkCampaignCollection> {
    validate_campaign_plan(plan).context("validate benchmark campaign collection plan")?;
    validate_utc_timestamp(observed_at_utc, "benchmark collection observed_at_utc")
        .context("validate benchmark collection timestamp")?;
    let benchmark_root =
        benchmark_root.canonicalize().context("canonicalize benchmark campaign evidence root")?;
    let mut units = Vec::with_capacity(plan.cycle.attempts.len());
    for planned in &plan.cycle.attempts {
        let identity = &planned.identity;
        let lifecycle = bijux_dna_runtime::load_attempt_lifecycle(&benchmark_root, identity)?;
        let prior_status = lifecycle
            .current_status()
            .context("benchmark campaign attempt has no lifecycle status")?;
        let scheduler_observation = if matches!(
            prior_status,
            BenchmarkLifecycleStatus::Submitted | BenchmarkLifecycleStatus::Running
        ) {
            Some(collect_active_attempt(paths, &benchmark_root, identity, observed_at_utc)?)
        } else {
            None
        };
        let current_status = bijux_dna_runtime::load_attempt_lifecycle(&benchmark_root, identity)?
            .current_status()
            .context("collected benchmark attempt has no lifecycle status")?;
        units.push(BenchmarkCampaignCollectionUnit {
            identity: identity.clone(),
            prior_status,
            current_status,
            scheduler_observation,
        });
    }
    Ok(BenchmarkCampaignCollection {
        campaign_id: plan.cycle.campaign_id.clone(),
        observed_at_utc: observed_at_utc.to_string(),
        units,
    })
}

fn collect_active_attempt(
    paths: &SlurmClientPaths,
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
    observed_at_utc: &str,
) -> Result<SlurmJobObservation> {
    let spec = load_execution_spec(benchmark_root, identity)?;
    let handle = load_benchmark_slurm_job_handle(&spec)?;
    let observation = collect_slurm_job(paths, &handle, observed_at_utc)?;
    let draft = benchmark_observation(&observation)?;
    let prior_observations =
        bijux_dna_runtime::load_attempt_scheduler_observations(benchmark_root, identity)?;
    if prior_observations.last().map(|record| &record.observation) != Some(&draft) {
        bijux_dna_runtime::record_attempt_scheduler_observation(benchmark_root, identity, draft)?;
    }
    let job = bijux_dna_bench_model::BenchmarkJobReference {
        scheduler: "slurm".to_string(),
        job_id: handle.job_id,
    };
    if observation.state == SlurmJobState::Running {
        bijux_dna_runtime::record_attempt_running(benchmark_root, identity, observed_at_utc, &job)?;
    } else if observation.state.is_terminal() {
        let accounting = observation
            .accounting
            .as_ref()
            .context("terminal benchmark observation omits scheduler accounting")?;
        bijux_dna_runtime::record_attempt_accounting(
            benchmark_root,
            identity,
            observed_at_utc,
            benchmark_accounting(accounting)?,
        )?;
    }
    Ok(observation)
}

fn load_execution_spec(
    benchmark_root: &Path,
    identity: &BenchmarkAttemptIdentity,
) -> Result<BenchmarkExecutionSpec> {
    let path: PathBuf = bijux_dna_runtime::benchmark_attempt_directory(benchmark_root, identity)?
        .join("execution-spec.json");
    let bytes = bijux_dna_infra::read_to_end_bounded(&path, MAX_EXECUTION_SPEC_BYTES)
        .with_context(|| format!("read benchmark execution specification {}", path.display()))?;
    let spec: BenchmarkExecutionSpec = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse benchmark execution specification {}", path.display()))?;
    validate_execution_spec(&spec).context("validate benchmark execution specification")?;
    if &spec.identity != identity {
        bail!("benchmark execution specification identity differs from its attempt path");
    }
    Ok(spec)
}

fn benchmark_observation(
    observation: &SlurmJobObservation,
) -> Result<BenchmarkSchedulerObservationDraft> {
    Ok(BenchmarkSchedulerObservationDraft {
        scheduler: "slurm".to_string(),
        job_id: observation.job_id.clone(),
        state: observed_state(observation.state),
        source: match observation.source {
            SlurmEvidenceSource::Submission => BenchmarkSchedulerEvidenceSource::Submission,
            SlurmEvidenceSource::Queue => BenchmarkSchedulerEvidenceSource::Queue,
            SlurmEvidenceSource::Accounting => BenchmarkSchedulerEvidenceSource::Accounting,
        },
        observed_at_utc: observation.observed_at_utc.clone(),
        accounting: observation.accounting.as_ref().map(benchmark_accounting).transpose()?,
    })
}

fn benchmark_accounting(accounting: &SlurmAccounting) -> Result<BenchmarkSchedulerAccounting> {
    if !accounting.array_tasks.is_empty() {
        bail!("benchmark campaign collection does not admit array accounting");
    }
    let state = match accounting.state {
        SlurmJobState::Completed => BenchmarkSchedulerTerminalState::Completed,
        SlurmJobState::Failed => BenchmarkSchedulerTerminalState::Failed,
        SlurmJobState::Cancelled => BenchmarkSchedulerTerminalState::Cancelled,
        SlurmJobState::Timeout => BenchmarkSchedulerTerminalState::Timeout,
        SlurmJobState::Preempted => BenchmarkSchedulerTerminalState::Preempted,
        SlurmJobState::NodeFailure => BenchmarkSchedulerTerminalState::NodeFailure,
        SlurmJobState::OutOfMemory => BenchmarkSchedulerTerminalState::OutOfMemory,
        _ => bail!("benchmark scheduler accounting is not terminal"),
    };
    Ok(BenchmarkSchedulerAccounting {
        scheduler: "slurm".to_string(),
        job_id: accounting.job_id.clone(),
        state,
        raw_state: accounting.raw_state.clone(),
        exit_code: accounting.exit_code,
        signal: accounting.signal,
        elapsed_seconds: accounting.elapsed_seconds,
        total_cpu_seconds: accounting.total_cpu_seconds,
        max_rss_bytes: accounting.max_rss_bytes,
        node_list: accounting.node_list.clone(),
        reason: accounting.reason.clone(),
    })
}

fn observed_state(state: SlurmJobState) -> BenchmarkSchedulerObservedState {
    match state {
        SlurmJobState::Submitted => BenchmarkSchedulerObservedState::Submitted,
        SlurmJobState::Pending => BenchmarkSchedulerObservedState::Pending,
        SlurmJobState::Running => BenchmarkSchedulerObservedState::Running,
        SlurmJobState::Completed => BenchmarkSchedulerObservedState::Completed,
        SlurmJobState::Failed => BenchmarkSchedulerObservedState::Failed,
        SlurmJobState::Cancelled => BenchmarkSchedulerObservedState::Cancelled,
        SlurmJobState::Timeout => BenchmarkSchedulerObservedState::Timeout,
        SlurmJobState::Preempted => BenchmarkSchedulerObservedState::Preempted,
        SlurmJobState::NodeFailure => BenchmarkSchedulerObservedState::NodeFailure,
        SlurmJobState::OutOfMemory => BenchmarkSchedulerObservedState::OutOfMemory,
        SlurmJobState::AccountingPending => BenchmarkSchedulerObservedState::AccountingPending,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_accounting_translation_preserves_failure_evidence() -> Result<()> {
        let accounting = SlurmAccounting {
            job_id: "12345".to_string(),
            state: SlurmJobState::OutOfMemory,
            raw_state: "OUT_OF_MEMORY".to_string(),
            exit_code: 1,
            signal: 9,
            elapsed_seconds: 12,
            total_cpu_seconds: 7,
            max_rss_bytes: Some(4096),
            node_list: "compute-1".to_string(),
            reason: Some("memory limit".to_string()),
            array_tasks: Vec::new(),
        };
        let translated = benchmark_accounting(&accounting)?;
        assert_eq!(translated.state, BenchmarkSchedulerTerminalState::OutOfMemory);
        assert_eq!(translated.raw_state, "OUT_OF_MEMORY");
        assert_eq!(translated.max_rss_bytes, Some(4096));
        assert_eq!(translated.reason.as_deref(), Some("memory limit"));
        Ok(())
    }

    #[test]
    fn array_accounting_is_rejected_by_scalar_campaign_collection() {
        let accounting = SlurmAccounting {
            job_id: "12345".to_string(),
            state: SlurmJobState::Completed,
            raw_state: "COMPLETED".to_string(),
            exit_code: 0,
            signal: 0,
            elapsed_seconds: 12,
            total_cpu_seconds: 7,
            max_rss_bytes: None,
            node_list: "compute-1".to_string(),
            reason: None,
            array_tasks: vec![super::super::SlurmArrayTaskAccounting {
                task_id: 0,
                job_id: "12345_0".to_string(),
                state: SlurmJobState::Completed,
                raw_state: "COMPLETED".to_string(),
                exit_code: 0,
                signal: 0,
                elapsed_seconds: 12,
                total_cpu_seconds: 7,
                max_rss_bytes: None,
                node_list: "compute-1".to_string(),
                reason: None,
            }],
        };
        assert!(benchmark_accounting(&accounting).is_err());
    }
}
