//! Owner: bijux-dna-runner
//! Aggregate, non-waiting scheduler passes for container qualification.

use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use super::{
    collect_container_qualification_slurm_job, load_container_qualification_scheduler_evidence,
    load_container_qualification_slurm_job_handle, load_container_qualification_slurm_spec,
    load_prepared_container_qualification_slurm_job, observe_container_qualification_slurm_job,
    submit_prepared_container_qualification_job, ContainerQualificationSchedulerStatus,
    ContainerQualificationSlurmSpec,
};
use crate::backend::slurm::{SlurmClientPaths, SlurmJobState};

pub const CONTAINER_QUALIFICATION_CYCLE_STATUS_SCHEMA_V1: &str =
    "bijux.runner.container_qualification_cycle_status.v1";
const DEFAULT_POLL_INTERVAL_SECONDS: i64 = 300;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerQualificationCycleRequest {
    pub repository_root: PathBuf,
    pub registry_path: PathBuf,
    pub publication_root: PathBuf,
    pub submission_root: PathBuf,
    pub observed_at_utc: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContainerQualificationCycleState {
    Published,
    Blocked,
    Ready,
    Submitted,
    Pending,
    Running,
    AccountingPending,
    Collectable,
    Passed,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContainerQualificationCycleAction {
    None,
    Prepare,
    Submit,
    Poll,
    Collect,
    ArchiveFailure,
    Book,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContainerQualificationCycleUnit {
    pub run_id: String,
    pub tool_id: String,
    pub state: ContainerQualificationCycleState,
    pub next_action: ContainerQualificationCycleAction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub specification_path: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub job_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContainerQualificationCycleSummary {
    pub unit_count: usize,
    pub published_count: usize,
    pub blocked_count: usize,
    pub ready_count: usize,
    pub active_count: usize,
    pub passed_count: usize,
    pub failed_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContainerQualificationCycleStatus {
    pub schema_version: String,
    pub observed_at_utc: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub earliest_poll_at_utc: Option<String>,
    pub summary: ContainerQualificationCycleSummary,
    pub units: Vec<ContainerQualificationCycleUnit>,
}

/// Derive a scheduler-independent cycle plan from governed run and attempt evidence.
///
/// # Errors
/// Returns an error when the registry, a discovered specification, or retained identity conflicts.
pub fn plan_container_qualification_cycle(
    request: &ContainerQualificationCycleRequest,
) -> Result<ContainerQualificationCycleStatus> {
    derive_cycle(request, None)
}

/// Observe every submitted qualification once and return immediately.
///
/// # Errors
/// Returns an error when scheduler observation or retained evidence validation fails.
pub fn status_container_qualification_cycle(
    paths: &SlurmClientPaths,
    request: &ContainerQualificationCycleRequest,
) -> Result<ContainerQualificationCycleStatus> {
    derive_cycle(request, Some(paths))
}

/// Submit every independently ready qualification once and return immediately.
///
/// Existing handles are loaded idempotently by the single-attempt scheduler authority.
///
/// # Errors
/// Returns an error when planning fails or any ready submission cannot be reconciled.
pub fn advance_container_qualification_cycle(
    paths: &SlurmClientPaths,
    request: &ContainerQualificationCycleRequest,
) -> Result<ContainerQualificationCycleStatus> {
    let planned = plan_container_qualification_cycle(request)?;
    let mut failures = Vec::new();
    for unit in
        planned.units.iter().filter(|unit| unit.state == ContainerQualificationCycleState::Ready)
    {
        let path = unit.specification_path.as_ref().context("ready cycle unit lacks a spec")?;
        let spec = load_container_qualification_slurm_spec(path)?;
        let prepared = load_prepared_container_qualification_slurm_job(&spec)?;
        if let Err(error) = submit_prepared_container_qualification_job(paths, &prepared) {
            failures.push(format!("{}:{}: {error:#}", unit.run_id, unit.tool_id));
        }
    }
    if !failures.is_empty() {
        bail!("one or more qualification submissions failed: {}", failures.join("; "));
    }
    plan_container_qualification_cycle(request)
}

/// Run one collection pass for every submitted qualification and return immediately.
///
/// Active and accounting-pending jobs remain active; terminal jobs are published idempotently.
///
/// # Errors
/// Returns an error when planning or any collection cannot be reconciled.
pub fn collect_container_qualification_cycle(
    paths: &SlurmClientPaths,
    request: &ContainerQualificationCycleRequest,
) -> Result<ContainerQualificationCycleStatus> {
    let planned = plan_container_qualification_cycle(request)?;
    let mut failures = Vec::new();
    for unit in planned.units.iter().filter(|unit| {
        matches!(
            unit.state,
            ContainerQualificationCycleState::Submitted
                | ContainerQualificationCycleState::Pending
                | ContainerQualificationCycleState::Running
                | ContainerQualificationCycleState::AccountingPending
        )
    }) {
        let path = unit.specification_path.as_ref().context("active cycle unit lacks a spec")?;
        let spec = load_container_qualification_slurm_spec(path)?;
        if let Err(error) =
            collect_container_qualification_slurm_job(paths, &spec, &request.observed_at_utc)
        {
            failures.push(format!("{}:{}: {error:#}", unit.run_id, unit.tool_id));
        }
    }
    if !failures.is_empty() {
        bail!("one or more qualification collections failed: {}", failures.join("; "));
    }
    plan_container_qualification_cycle(request)
}

fn derive_cycle(
    request: &ContainerQualificationCycleRequest,
    scheduler: Option<&SlurmClientPaths>,
) -> Result<ContainerQualificationCycleStatus> {
    let observed_at = DateTime::parse_from_rfc3339(&request.observed_at_utc)
        .context("parse container cycle observation timestamp")?
        .with_timezone(&Utc);
    let registry = bijux_dna_containers::api::load_qualification_run_registry(
        &request.repository_root,
        &request.registry_path,
    )?;
    let specifications = discover_specifications(&request.submission_root)?;
    let mut units = Vec::new();
    for run in registry.runs.iter().filter(|run| {
        matches!(
            run.state,
            bijux_dna_containers::api::QualificationRunState::Active
                | bijux_dna_containers::api::QualificationRunState::Allocated
        )
    }) {
        let [tool_id] = run.tool_ids.as_slice() else {
            bail!("current qualification run {} does not select exactly one tool", run.run_id);
        };
        let publication = request.publication_root.join(format!("{}-{tool_id}", run.run_id));
        if regular_file_exists(&publication.join("evidence.lock.json"))? {
            units.push(unit(
                &run.run_id,
                tool_id,
                ContainerQualificationCycleState::Published,
                ContainerQualificationCycleAction::None,
                None,
                None,
                None,
            ));
            continue;
        }
        let key = (run.run_id.clone(), tool_id.clone());
        let Some((path, spec)) = specifications.get(&key) else {
            units.push(unit(
                &run.run_id,
                tool_id,
                ContainerQualificationCycleState::Blocked,
                ContainerQualificationCycleAction::Prepare,
                None,
                None,
                Some("prepare an immutable qualification attempt from the governed protocol"),
            ));
            continue;
        };
        units.push(classify_spec(path, spec, scheduler, &request.observed_at_utc)?);
    }
    let active = units.iter().any(|unit| {
        matches!(
            unit.state,
            ContainerQualificationCycleState::Submitted
                | ContainerQualificationCycleState::Pending
                | ContainerQualificationCycleState::Running
                | ContainerQualificationCycleState::AccountingPending
        )
    });
    let earliest_poll_at_utc = active
        .then(|| (observed_at + Duration::seconds(DEFAULT_POLL_INTERVAL_SECONDS)).to_rfc3339());
    Ok(ContainerQualificationCycleStatus {
        schema_version: CONTAINER_QUALIFICATION_CYCLE_STATUS_SCHEMA_V1.to_string(),
        observed_at_utc: observed_at.to_rfc3339(),
        earliest_poll_at_utc,
        summary: summarize(&units),
        units,
    })
}

fn discover_specifications(
    root: &Path,
) -> Result<BTreeMap<(String, String), (PathBuf, ContainerQualificationSlurmSpec)>> {
    if !root.exists() {
        return Ok(BTreeMap::new());
    }
    let mut specifications = BTreeMap::new();
    for entry in WalkDir::new(root).follow_links(false) {
        let entry = entry.with_context(|| format!("walk qualification root {}", root.display()))?;
        if !entry.file_type().is_file() || entry.file_name() != "qualification-spec.json" {
            continue;
        }
        let path = entry.path().to_path_buf();
        let spec = load_container_qualification_slurm_spec(&path)?;
        let key = (spec.run_id.clone(), spec.tool_id.clone());
        if specifications.insert(key.clone(), (path, spec)).is_some() {
            bail!("multiple live qualification specifications exist for {}:{}", key.0, key.1);
        }
    }
    Ok(specifications)
}

fn classify_spec(
    path: &Path,
    spec: &ContainerQualificationSlurmSpec,
    scheduler: Option<&SlurmClientPaths>,
    observed_at_utc: &str,
) -> Result<ContainerQualificationCycleUnit> {
    if let Some(evidence) = load_container_qualification_scheduler_evidence(spec)? {
        let (state, action) = match evidence.status {
            ContainerQualificationSchedulerStatus::Passed => {
                (ContainerQualificationCycleState::Passed, ContainerQualificationCycleAction::Book)
            }
            ContainerQualificationSchedulerStatus::Failed => (
                ContainerQualificationCycleState::Failed,
                ContainerQualificationCycleAction::ArchiveFailure,
            ),
        };
        return Ok(unit(
            &spec.run_id,
            &spec.tool_id,
            state,
            action,
            Some(path),
            Some(evidence.job_id),
            evidence.failure.as_deref(),
        ));
    }
    let handle_path = spec.evidence_root.join("scheduler/job-handle.json");
    if regular_file_exists(&handle_path)? {
        let handle = load_container_qualification_slurm_job_handle(spec)?;
        let (state, action, detail) = if let Some(paths) = scheduler {
            let observation =
                observe_container_qualification_slurm_job(paths, spec, observed_at_utc)?;
            scheduler_state(observation.state)
        } else {
            (
                ContainerQualificationCycleState::Submitted,
                ContainerQualificationCycleAction::Poll,
                None,
            )
        };
        return Ok(unit(
            &spec.run_id,
            &spec.tool_id,
            state,
            action,
            Some(path),
            Some(handle.job_id),
            detail,
        ));
    }
    load_prepared_container_qualification_slurm_job(spec)?;
    Ok(unit(
        &spec.run_id,
        &spec.tool_id,
        ContainerQualificationCycleState::Ready,
        ContainerQualificationCycleAction::Submit,
        Some(path),
        None,
        None,
    ))
}

fn scheduler_state(
    state: SlurmJobState,
) -> (ContainerQualificationCycleState, ContainerQualificationCycleAction, Option<&'static str>) {
    match state {
        SlurmJobState::Submitted => (
            ContainerQualificationCycleState::Submitted,
            ContainerQualificationCycleAction::Poll,
            None,
        ),
        SlurmJobState::Pending => (
            ContainerQualificationCycleState::Pending,
            ContainerQualificationCycleAction::Poll,
            None,
        ),
        SlurmJobState::Running => (
            ContainerQualificationCycleState::Running,
            ContainerQualificationCycleAction::Poll,
            None,
        ),
        SlurmJobState::AccountingPending => (
            ContainerQualificationCycleState::AccountingPending,
            ContainerQualificationCycleAction::Poll,
            Some("terminal accounting is not available yet"),
        ),
        SlurmJobState::Completed
        | SlurmJobState::Failed
        | SlurmJobState::Cancelled
        | SlurmJobState::Timeout
        | SlurmJobState::Preempted
        | SlurmJobState::NodeFailure
        | SlurmJobState::OutOfMemory => (
            ContainerQualificationCycleState::Collectable,
            ContainerQualificationCycleAction::Collect,
            Some("scheduler accounting is terminal and ready for collection"),
        ),
    }
}

fn unit(
    run_id: &str,
    tool_id: &str,
    state: ContainerQualificationCycleState,
    next_action: ContainerQualificationCycleAction,
    specification_path: Option<&Path>,
    job_id: Option<String>,
    detail: Option<&str>,
) -> ContainerQualificationCycleUnit {
    ContainerQualificationCycleUnit {
        run_id: run_id.to_string(),
        tool_id: tool_id.to_string(),
        state,
        next_action,
        specification_path: specification_path.map(Path::to_path_buf),
        job_id,
        detail: detail.map(str::to_string),
    }
}

fn summarize(units: &[ContainerQualificationCycleUnit]) -> ContainerQualificationCycleSummary {
    ContainerQualificationCycleSummary {
        unit_count: units.len(),
        published_count: count(units, &[ContainerQualificationCycleState::Published]),
        blocked_count: count(units, &[ContainerQualificationCycleState::Blocked]),
        ready_count: count(units, &[ContainerQualificationCycleState::Ready]),
        active_count: count(
            units,
            &[
                ContainerQualificationCycleState::Submitted,
                ContainerQualificationCycleState::Pending,
                ContainerQualificationCycleState::Running,
                ContainerQualificationCycleState::AccountingPending,
            ],
        ),
        passed_count: count(units, &[ContainerQualificationCycleState::Passed]),
        failed_count: count(units, &[ContainerQualificationCycleState::Failed]),
    }
}

fn count(
    units: &[ContainerQualificationCycleUnit],
    states: &[ContainerQualificationCycleState],
) -> usize {
    units.iter().filter(|unit| states.contains(&unit.state)).count()
}

fn regular_file_exists(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => Ok(true),
        Ok(_) => bail!("cycle evidence path is not a regular file: {}", path.display()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => {
            Err(error).with_context(|| format!("inspect cycle evidence {}", path.display()))
        }
    }
}
