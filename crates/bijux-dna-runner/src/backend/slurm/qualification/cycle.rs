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
    prepare_container_qualification_slurm_job, submit_prepared_container_qualification_job,
    ContainerQualificationPreparationRequest, ContainerQualificationSchedulerStatus,
    ContainerQualificationSlurmSpec,
};
use crate::backend::slurm::{SlurmClientPaths, SlurmJobState};

pub const CONTAINER_QUALIFICATION_CYCLE_STATUS_SCHEMA_V1: &str =
    "bijux.runner.container_qualification_cycle_status.v1";
pub const CONTAINER_QUALIFICATION_CYCLE_STATUS_SCHEMA_V2: &str =
    "bijux.runner.container_qualification_cycle_status.v2";
pub const CONTAINER_QUALIFICATION_CYCLE_PREPARATION_SCHEMA_V1: &str =
    "bijux.runner.container_qualification_cycle_preparation.v1";
pub const CONTAINER_QUALIFICATION_CYCLE_PREPARATION_SCHEMA_V2: &str =
    "bijux.runner.container_qualification_cycle_preparation.v2";
const DEFAULT_OBSERVATION_INTERVAL_SECONDS: i64 = 300;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContainerQualificationCyclePreparation {
    pub schema_version: String,
    pub sources_root: PathBuf,
    pub governance_path: PathBuf,
    pub credential_reference_path: PathBuf,
    pub executor_program_path: PathBuf,
    pub hostname: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_materialization: Option<ContainerQualificationSourceMaterialization>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContainerQualificationSourceMaterialization {
    pub selections_root: PathBuf,
    pub data_index_path: PathBuf,
    pub operator: String,
    pub walltime: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerQualificationCycleRequest {
    pub repository_root: PathBuf,
    pub registry_path: PathBuf,
    pub inventory_path: PathBuf,
    pub publication_root: PathBuf,
    pub submission_root: PathBuf,
    pub external_artifact_admission_root: Option<PathBuf>,
    pub preparation: Option<ContainerQualificationCyclePreparation>,
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
    Admitted,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContainerQualificationCycleAction {
    None,
    Materialize,
    Prepare,
    Submit,
    #[serde(alias = "poll")]
    Observe,
    Collect,
    Admit,
    ArchiveFailure,
    ArchiveSuperseded,
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
    pub admitted_count: usize,
    pub failed_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContainerQualificationCycleStatus {
    pub schema_version: String,
    pub observed_at_utc: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(alias = "earliest_poll_at_utc")]
    pub earliest_observation_at_utc: Option<String>,
    pub summary: ContainerQualificationCycleSummary,
    pub units: Vec<ContainerQualificationCycleUnit>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AdvancePass {
    Collect,
    ArchiveFailure,
    ArchiveSuperseded,
    Admit,
    Materialize,
    Prepare,
    Submit,
}

const ADVANCE_PASS_ORDER: [AdvancePass; 7] = [
    AdvancePass::Collect,
    AdvancePass::ArchiveFailure,
    AdvancePass::ArchiveSuperseded,
    AdvancePass::Admit,
    AdvancePass::Materialize,
    AdvancePass::Prepare,
    AdvancePass::Submit,
];

/// Load and validate one site-owned qualification cycle preparation authority.
///
/// # Errors
/// Returns an error for a malformed document, unsupported schema, relative path, unsafe source
/// root, invalid hostname, or unavailable declared authority.
pub fn load_container_qualification_cycle_preparation(
    path: &Path,
) -> Result<ContainerQualificationCyclePreparation> {
    let preparation: ContainerQualificationCyclePreparation = serde_json::from_slice(
        &bijux_dna_infra::read_to_end_bounded(path, 16 * 1024 * 1024)
            .with_context(|| format!("read cycle preparation {}", path.display()))?,
    )
    .with_context(|| format!("parse cycle preparation {}", path.display()))?;
    validate_cycle_preparation(&preparation)?;
    Ok(preparation)
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

/// Collect, archive failures, admit, prepare, and submit the independent frontier once, then return.
///
/// Existing handles are loaded idempotently by the single-attempt scheduler authority.
///
/// # Errors
/// Returns an error after the complete frontier pass when planning fails or any collection,
/// archival, admission, preparation, or submission cannot be reconciled.
pub fn advance_container_qualification_cycle(
    paths: &SlurmClientPaths,
    request: &ContainerQualificationCycleRequest,
) -> Result<ContainerQualificationCycleStatus> {
    let mut failures = Vec::new();
    for pass in ADVANCE_PASS_ORDER {
        let stage_failures = match pass {
            AdvancePass::Collect => collect_frontier(paths, request)?,
            AdvancePass::ArchiveFailure => archive_failure_frontier(request)?,
            AdvancePass::ArchiveSuperseded => archive_superseded_frontier(request)?,
            AdvancePass::Admit => admit_frontier(request)?,
            AdvancePass::Materialize => materialize_source_frontier(request)?,
            AdvancePass::Prepare => prepare_frontier(request)?,
            AdvancePass::Submit => submit_frontier(paths, request)?,
        };
        failures.extend(stage_failures.into_iter().map(|failure| {
            format!(
                "{}: {failure}",
                match pass {
                    AdvancePass::Collect => "collect",
                    AdvancePass::ArchiveFailure => "archive_failure",
                    AdvancePass::ArchiveSuperseded => "archive_superseded",
                    AdvancePass::Admit => "admit",
                    AdvancePass::Materialize => "materialize",
                    AdvancePass::Prepare => "prepare",
                    AdvancePass::Submit => "submit",
                }
            )
        }));
    }
    if failures.is_empty() {
        plan_container_qualification_cycle(request)
    } else {
        bail!("one or more qualification advance actions failed: {}", failures.join("; "));
    }
}

fn materialize_source_frontier(
    request: &ContainerQualificationCycleRequest,
) -> Result<Vec<String>> {
    let planned = plan_container_qualification_cycle(request)?;
    let Some(preparation) = request.preparation.as_ref() else {
        return Ok(Vec::new());
    };
    let Some(materialization) = preparation.source_materialization.as_ref() else {
        return Ok(Vec::new());
    };
    let governance =
        bijux_dna_containers::api::load_container_governance(&preparation.governance_path)?;
    let repository_commit =
        bijux_dna_containers::api::repository_head_commit(&request.repository_root)?;
    let frozen_repository = governance.paths.frozen_repositories.join(&repository_commit);
    let partition = governance
        .slurm
        .partitions
        .first()
        .context("qualification governance has no Slurm partition")?;
    let mut failures = Vec::new();
    for unit in planned.units.iter().filter(|unit| {
        unit.state == ContainerQualificationCycleState::Ready
            && unit.next_action == ContainerQualificationCycleAction::Materialize
    }) {
        let result = (|| -> Result<()> {
            let selection_path =
                qualification_source_path(&materialization.selections_root, &unit.tool_id)?
                    .context("materialization-ready qualification selection disappeared")?;
            let selection =
                bijux_dna_containers_qa::api::load_qualification_source_selection(&selection_path)?;
            let source =
                bijux_dna_containers_qa::api::materialize_qualification_specification_source(
                    &bijux_dna_containers_qa::api::QualificationSourceMaterializationRequest {
                        repository_root: request.repository_root.clone(),
                        registry_path: request.registry_path.clone(),
                        inventory_path: request.inventory_path.clone(),
                        data_index_path: materialization.data_index_path.clone(),
                        governed_study_root: governance.paths.study_root.clone(),
                        repository_commit: repository_commit.clone(),
                        operator: materialization.operator.clone(),
                        submitted_at_utc: request.observed_at_utc.clone(),
                        account: governance.slurm.account.clone(),
                        partition: partition.clone(),
                        walltime: materialization.walltime.clone(),
                    },
                    &selection,
                )?;
            if source.run_id != unit.run_id || source.tool_id != unit.tool_id {
                bail!("materialized qualification source conflicts with cycle unit");
            }
            let bound =
                bijux_dna_containers_qa::api::bind_qualification_source_to_frozen_repository(
                    &source,
                    &governance,
                    &frozen_repository,
                )?;
            let mut bytes = serde_json::to_vec_pretty(&bound)?;
            bytes.push(b'\n');
            ensure_bytes_identity(
                &preparation.sources_root.join(format!("{}.json", unit.tool_id)),
                &bytes,
            )
        })();
        if let Err(error) = result {
            failures.push(format!("{}:{}: {error:#}", unit.run_id, unit.tool_id));
        }
    }
    Ok(failures)
}

fn prepare_frontier(request: &ContainerQualificationCycleRequest) -> Result<Vec<String>> {
    let planned = plan_container_qualification_cycle(request)?;
    let Some(preparation) = request.preparation.as_ref() else {
        return Ok(Vec::new());
    };
    validate_cycle_preparation(preparation)?;
    let governance =
        bijux_dna_containers::api::load_container_governance(&preparation.governance_path)?;
    let governed_attempt_root = governance
        .paths
        .containers_root
        .join("evidence/attempts")
        .canonicalize()
        .context("canonicalize governed qualification attempt root")?;
    let submission_root = request
        .submission_root
        .canonicalize()
        .context("canonicalize qualification submission root")?;
    if submission_root != governed_attempt_root {
        bail!("qualification submission root differs from governed attempt root");
    }
    let repository_commit =
        bijux_dna_containers::api::repository_head_commit(&request.repository_root)?;
    let compilation = bijux_dna_containers_qa::api::QualificationSpecificationCompilationRequest {
        repository_root: request.repository_root.clone(),
        registry_path: request.registry_path.clone(),
        inventory_path: request.inventory_path.clone(),
    };
    let mut failures = Vec::new();
    for unit in planned.units.iter().filter(|unit| {
        unit.state == ContainerQualificationCycleState::Ready
            && unit.next_action == ContainerQualificationCycleAction::Prepare
    }) {
        let result = (|| -> Result<()> {
            let source_path = qualification_source_path(&preparation.sources_root, &unit.tool_id)?
                .context("preparation-ready qualification source disappeared")?;
            let source = bijux_dna_containers_qa::api::load_qualification_specification_source(
                &source_path,
            )?;
            if source.run_id != unit.run_id || source.tool_id != unit.tool_id {
                bail!("qualification preparation source identity conflicts with cycle unit");
            }
            let pipeline =
                bijux_dna_containers_qa::api::compile_container_qualification_specification(
                    &compilation,
                    &source,
                )?;
            let mut pipeline_bytes = serde_json::to_vec_pretty(&pipeline)?;
            pipeline_bytes.push(b'\n');
            let pipeline_path = request
                .submission_root
                .join(".cycle-pipelines")
                .join(&repository_commit)
                .join(format!("{}-{}", unit.run_id, unit.tool_id))
                .join("config-pipeline.json");
            ensure_bytes_identity(&pipeline_path, &pipeline_bytes)?;
            let pipeline_sha256 = bijux_dna_infra::hash_file_sha256(&pipeline_path)?;
            let reservation = bijux_dna_runtime::reserve_attempt_sequence(
                &request.submission_root,
                bijux_dna_infra::AttemptEvidenceOwner::Containers,
                &bijux_dna_runtime::AttemptSequenceIdentity {
                    run_directory: unit.run_id.clone(),
                    tool_id: unit.tool_id.clone(),
                    campaign_id: source.campaign,
                },
                &pipeline_sha256,
                &request.observed_at_utc,
            )?;
            let attempt_identity = reservation.identity.clone();
            if let Err(error) = prepare_container_qualification_slurm_job(
                &ContainerQualificationPreparationRequest {
                    attempt_id: attempt_identity.attempt_id.clone(),
                    hostname: preparation.hostname.clone(),
                    governance_path: preparation.governance_path.clone(),
                    pipeline_path,
                    credential_reference_path: preparation.credential_reference_path.clone(),
                    executor_program_path: preparation.executor_program_path.clone(),
                },
                &request.observed_at_utc,
            ) {
                let archival = bijux_dna_infra::archive_attempt_layout(
                    &request.submission_root,
                    bijux_dna_infra::AttemptEvidenceOwner::Containers,
                    &attempt_identity,
                    bijux_dna_infra::AttemptArchiveDisposition::Failed,
                    &request.observed_at_utc,
                );
                return match archival {
                    Ok(_) => Err(error).context("prepare reserved qualification attempt"),
                    Err(archive_error) => bail!(
                        "prepare reserved qualification attempt: {error:#}; archive failed reservation: {archive_error:#}"
                    ),
                };
            }
            Ok(())
        })();
        if let Err(error) = result {
            failures.push(format!("{}:{}: {error:#}", unit.run_id, unit.tool_id));
        }
    }
    Ok(failures)
}

fn archive_failure_frontier(request: &ContainerQualificationCycleRequest) -> Result<Vec<String>> {
    let planned = plan_container_qualification_cycle(request)?;
    let mut failures = Vec::new();
    for unit in
        planned.units.iter().filter(|unit| unit.state == ContainerQualificationCycleState::Failed)
    {
        let result = (|| -> Result<()> {
            let path = unit
                .specification_path
                .as_ref()
                .context("failed cycle unit lacks a specification")?;
            let spec = load_container_qualification_slurm_spec(path)?;
            super::archive_failed_container_qualification_attempt(&spec, &request.observed_at_utc)?;
            Ok(())
        })();
        if let Err(error) = result {
            failures.push(format!("{}:{}: {error:#}", unit.run_id, unit.tool_id));
        }
    }
    Ok(failures)
}

fn archive_superseded_frontier(
    request: &ContainerQualificationCycleRequest,
) -> Result<Vec<String>> {
    let planned = plan_container_qualification_cycle(request)?;
    let mut failures = Vec::new();
    for unit in planned.units.iter().filter(|unit| {
        unit.state == ContainerQualificationCycleState::Blocked
            && unit.next_action == ContainerQualificationCycleAction::ArchiveSuperseded
    }) {
        let result = (|| -> Result<()> {
            let path = unit
                .specification_path
                .as_ref()
                .context("superseded cycle unit lacks a specification")?;
            let spec = load_container_qualification_slurm_spec(path)?;
            let identity = bijux_dna_infra::AttemptLayoutIdentity {
                run_directory: spec.run_id.clone(),
                tool_id: spec.tool_id.clone(),
                campaign_id: spec
                    .campaign_id
                    .clone()
                    .context("superseded qualification attempt omits campaign identity")?,
                attempt_id: spec
                    .attempt_id
                    .clone()
                    .context("superseded qualification attempt omits attempt identity")?,
            };
            let expected =
                bijux_dna_infra::attempt_layout_paths(&request.submission_root, &identity)?;
            if expected.attempt_dir != spec.evidence_root || spec.result_root != spec.evidence_root
            {
                bail!("superseded qualification attempt is outside its owner-scoped layout");
            }
            bijux_dna_runtime::supersede_attempt_sequence(
                &request.submission_root,
                bijux_dna_infra::AttemptEvidenceOwner::Containers,
                &identity,
                &request.observed_at_utc,
            )?;
            Ok(())
        })();
        if let Err(error) = result {
            failures.push(format!("{}:{}: {error:#}", unit.run_id, unit.tool_id));
        }
    }
    Ok(failures)
}

fn submit_frontier(
    paths: &SlurmClientPaths,
    request: &ContainerQualificationCycleRequest,
) -> Result<Vec<String>> {
    let planned = plan_container_qualification_cycle(request)?;
    let mut failures = Vec::new();
    for unit in planned.units.iter().filter(|unit| {
        unit.state == ContainerQualificationCycleState::Ready
            && unit.next_action == ContainerQualificationCycleAction::Submit
    }) {
        let result = (|| -> Result<()> {
            let path = unit.specification_path.as_ref().context("ready cycle unit lacks a spec")?;
            let spec = load_container_qualification_slurm_spec(path)?;
            let prepared = load_prepared_container_qualification_slurm_job(&spec)?;
            submit_prepared_container_qualification_job(paths, &prepared)?;
            Ok(())
        })();
        if let Err(error) = result {
            failures.push(format!("{}:{}: {error:#}", unit.run_id, unit.tool_id));
        }
    }
    Ok(failures)
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
    let failures = collect_frontier(paths, request)?;
    if !failures.is_empty() {
        bail!("one or more qualification collections failed: {}", failures.join("; "));
    }
    plan_container_qualification_cycle(request)
}

fn collect_frontier(
    paths: &SlurmClientPaths,
    request: &ContainerQualificationCycleRequest,
) -> Result<Vec<String>> {
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
        let result = (|| -> Result<()> {
            let path =
                unit.specification_path.as_ref().context("active cycle unit lacks a spec")?;
            let spec = load_container_qualification_slurm_spec(path)?;
            collect_container_qualification_slurm_job(paths, &spec, &request.observed_at_utc)?;
            Ok(())
        })();
        if let Err(error) = result {
            failures.push(format!("{}:{}: {error:#}", unit.run_id, unit.tool_id));
        }
    }
    Ok(failures)
}

/// Admit every independently collected passing qualification once and return immediately.
///
/// # Errors
/// Returns an error when planning fails or a terminal evidence chain cannot be admitted.
pub fn admit_container_qualification_cycle(
    request: &ContainerQualificationCycleRequest,
) -> Result<ContainerQualificationCycleStatus> {
    let failures = admit_frontier(request)?;
    if !failures.is_empty() {
        bail!("one or more qualification admissions failed: {}", failures.join("; "));
    }
    plan_container_qualification_cycle(request)
}

fn admit_frontier(request: &ContainerQualificationCycleRequest) -> Result<Vec<String>> {
    let planned = plan_container_qualification_cycle(request)?;
    let mut failures = Vec::new();
    for unit in
        planned.units.iter().filter(|unit| unit.state == ContainerQualificationCycleState::Passed)
    {
        let result = (|| -> Result<()> {
            let path =
                unit.specification_path.as_ref().context("passed cycle unit lacks a spec")?;
            let spec = load_container_qualification_slurm_spec(path)?;
            super::admit_container_qualification_collection(&spec, &request.observed_at_utc)?;
            Ok(())
        })();
        if let Err(error) = result {
            failures.push(format!("{}:{}: {error:#}", unit.run_id, unit.tool_id));
        }
    }
    Ok(failures)
}

fn derive_cycle(
    request: &ContainerQualificationCycleRequest,
    scheduler: Option<&SlurmClientPaths>,
) -> Result<ContainerQualificationCycleStatus> {
    if let Some(preparation) = request.preparation.as_ref() {
        validate_cycle_preparation(preparation)?;
    }
    let observed_at = DateTime::parse_from_rfc3339(&request.observed_at_utc)
        .context("parse container cycle observation timestamp")?
        .with_timezone(&Utc);
    let registry = bijux_dna_containers::api::load_qualification_run_registry(
        &request.repository_root,
        &request.registry_path,
    )?;
    let catalog = bijux_dna_containers::api::load_container_catalog(&request.repository_root)?;
    let repository_commit =
        bijux_dna_containers::api::repository_head_commit(&request.repository_root)?;
    let inventory =
        bijux_dna_containers::api::load_image_inventory_publication(&request.inventory_path)?;
    if inventory.repository_commit != repository_commit {
        bail!(
            "qualification image inventory source {} differs from current repository commit {}",
            inventory.repository_commit,
            repository_commit
        );
    }
    let catalog_sha256 = bijux_dna_infra::hash_file_sha256(
        &request.repository_root.join("containers/contracts/catalog.toml"),
    )?;
    bijux_dna_containers::api::validate_container_inventory_catalog(
        &catalog_sha256,
        &catalog,
        &inventory,
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
        units.push(derive_run_unit(
            request,
            &catalog,
            &inventory,
            run,
            &specifications,
            &repository_commit,
            scheduler,
        )?);
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
    let earliest_observation_at_utc = active.then(|| {
        (observed_at + Duration::seconds(DEFAULT_OBSERVATION_INTERVAL_SECONDS)).to_rfc3339()
    });
    Ok(ContainerQualificationCycleStatus {
        schema_version: CONTAINER_QUALIFICATION_CYCLE_STATUS_SCHEMA_V2.to_string(),
        observed_at_utc: observed_at.to_rfc3339(),
        earliest_observation_at_utc,
        summary: summarize(&units),
        units,
    })
}

fn derive_run_unit(
    request: &ContainerQualificationCycleRequest,
    catalog: &bijux_dna_containers::api::ContainerCatalog,
    inventory: &bijux_dna_containers::api::ContainerImageInventoryPublication,
    run: &bijux_dna_containers::api::QualificationRunDefinition,
    specifications: &BTreeMap<(String, String), Vec<PathBuf>>,
    repository_commit: &str,
    scheduler: Option<&SlurmClientPaths>,
) -> Result<ContainerQualificationCycleUnit> {
    let [tool_id] = run.tool_ids.as_slice() else {
        bail!("current qualification run {} does not select exactly one tool", run.run_id);
    };
    let tool = catalog
        .tools
        .iter()
        .find(|tool| tool.id == *tool_id)
        .with_context(|| format!("qualification tool {tool_id} is absent from catalog"))?;
    let artifact = if tool.external_artifact_supported {
        external_artifact_resolution(request, &run.run_id, tool_id)?
    } else {
        inventory_artifact_resolution(inventory, &run.run_id, tool_id)?
    };
    let current_artifact_sha256 = match artifact {
        CurrentArtifactResolution::Ready(sha256) => sha256,
        CurrentArtifactResolution::Blocked(blocker) => return Ok(blocker),
    };
    let publication = request.publication_root.join(format!("{}-{tool_id}", run.run_id));
    let mut publication_staleness = None;
    if run.state == bijux_dna_containers::api::QualificationRunState::Active
        && regular_file_exists(&publication.join("evidence.lock.json"))?
    {
        let evidence = bijux_dna_containers::api::load_run_evidence_lock(&publication)?;
        publication_staleness =
            publication_staleness_detail(&evidence, repository_commit, &current_artifact_sha256)?;
        if publication_staleness.is_none() {
            return Ok(unit(
                &run.run_id,
                tool_id,
                ContainerQualificationCycleState::Published,
                ContainerQualificationCycleAction::None,
                None,
                None,
                None,
            ));
        }
    }
    let key = (run.run_id.clone(), tool_id.clone());
    let Some(paths) = specifications.get(&key) else {
        return derive_unprepared_unit(
            request,
            run,
            tool,
            tool_id,
            publication_staleness.as_deref(),
        );
    };
    let [path] = paths.as_slice() else {
        bail!("multiple live qualification specifications exist for {}:{}", key.0, key.1);
    };
    let spec = load_container_qualification_slurm_spec(path)?;
    classify_spec(
        path,
        &spec,
        repository_commit,
        &current_artifact_sha256,
        request.preparation.as_ref(),
        scheduler,
        &request.observed_at_utc,
    )
}

fn derive_unprepared_unit(
    request: &ContainerQualificationCycleRequest,
    run: &bijux_dna_containers::api::QualificationRunDefinition,
    tool: &bijux_dna_containers::api::ContainerCatalogTool,
    tool_id: &str,
    publication_staleness: Option<&str>,
) -> Result<ContainerQualificationCycleUnit> {
    let default_detail = if tool.external_artifact_supported {
        "prepare an immutable qualification attempt bound to the admitted external artifact"
    } else {
        "prepare an immutable qualification attempt from the governed protocol"
    };
    let Some(preparation) = request.preparation.as_ref() else {
        return Ok(unit(
            &run.run_id,
            tool_id,
            ContainerQualificationCycleState::Blocked,
            ContainerQualificationCycleAction::Prepare,
            None,
            None,
            publication_staleness.or(Some(default_detail)),
        ));
    };
    if let Some(source_path) = qualification_source_path(&preparation.sources_root, tool_id)? {
        let validation = validate_preparation_source(request, &source_path, run, tool_id);
        return Ok(match validation {
            Ok(()) => unit(
                &run.run_id,
                tool_id,
                ContainerQualificationCycleState::Ready,
                ContainerQualificationCycleAction::Prepare,
                None,
                None,
                publication_staleness.or(Some(default_detail)),
            ),
            Err(error) => unit(
                &run.run_id,
                tool_id,
                ContainerQualificationCycleState::Blocked,
                ContainerQualificationCycleAction::None,
                None,
                None,
                Some(&format!("qualification preparation source is invalid: {error:#}")),
            ),
        });
    }
    let Some(materialization) = preparation.source_materialization.as_ref() else {
        return Ok(unit(
            &run.run_id,
            tool_id,
            ContainerQualificationCycleState::Blocked,
            ContainerQualificationCycleAction::Prepare,
            None,
            None,
            publication_staleness.or(Some(default_detail)),
        ));
    };
    let Some(selection_path) =
        qualification_source_path(&materialization.selections_root, tool_id)?
    else {
        return Ok(unit(
            &run.run_id,
            tool_id,
            ContainerQualificationCycleState::Blocked,
            ContainerQualificationCycleAction::Materialize,
            None,
            None,
            publication_staleness.or(Some("declare an admitted qualification source selection")),
        ));
    };
    let selection =
        bijux_dna_containers_qa::api::load_qualification_source_selection(&selection_path)?;
    if selection.tool_id != tool_id {
        bail!("qualification selection identity conflicts with its file name");
    }
    Ok(unit(
        &run.run_id,
        tool_id,
        ContainerQualificationCycleState::Ready,
        ContainerQualificationCycleAction::Materialize,
        None,
        None,
        publication_staleness.or(Some("materialize an admitted qualification source")),
    ))
}

fn validate_preparation_source(
    request: &ContainerQualificationCycleRequest,
    source_path: &Path,
    run: &bijux_dna_containers::api::QualificationRunDefinition,
    tool_id: &str,
) -> Result<()> {
    let source =
        bijux_dna_containers_qa::api::load_qualification_specification_source(source_path)?;
    if source.run_id != run.run_id || source.tool_id != tool_id {
        bail!("source identity conflicts with the qualification registry");
    }
    bijux_dna_containers_qa::api::compile_container_qualification_specification(
        &bijux_dna_containers_qa::api::QualificationSpecificationCompilationRequest {
            repository_root: request.repository_root.clone(),
            registry_path: request.registry_path.clone(),
            inventory_path: request.inventory_path.clone(),
        },
        &source,
    )?;
    Ok(())
}

enum CurrentArtifactResolution {
    Ready(String),
    Blocked(ContainerQualificationCycleUnit),
}

fn inventory_artifact_resolution(
    inventory: &bijux_dna_containers::api::ContainerImageInventoryPublication,
    run_id: &str,
    tool_id: &str,
) -> Result<CurrentArtifactResolution> {
    let tool = inventory
        .tools
        .iter()
        .find(|tool| tool.tool == tool_id)
        .with_context(|| format!("qualification inventory omits tool {tool_id}"))?;
    if tool.state == bijux_dna_containers::api::ContainerImageState::Current {
        let sha256 = tool
            .selected_image_sha256
            .clone()
            .context("current qualification image omits its selected identity")?;
        return Ok(CurrentArtifactResolution::Ready(sha256));
    }
    let state = match tool.state {
        bijux_dna_containers::api::ContainerImageState::Current => unreachable!(),
        bijux_dna_containers::api::ContainerImageState::Historical => "historical",
        bijux_dna_containers::api::ContainerImageState::Ambiguous => "ambiguous",
        bijux_dna_containers::api::ContainerImageState::Missing => "missing",
        bijux_dna_containers::api::ContainerImageState::NotApplicable => "not_applicable",
    };
    let detail = format!("current qualification image is unavailable: inventory state {state}");
    Ok(CurrentArtifactResolution::Blocked(unit(
        run_id,
        tool_id,
        ContainerQualificationCycleState::Blocked,
        ContainerQualificationCycleAction::None,
        None,
        None,
        Some(&detail),
    )))
}

fn external_artifact_resolution(
    request: &ContainerQualificationCycleRequest,
    run_id: &str,
    tool_id: &str,
) -> Result<CurrentArtifactResolution> {
    let Some(root) = &request.external_artifact_admission_root else {
        return Ok(CurrentArtifactResolution::Blocked(unit(
            run_id,
            tool_id,
            ContainerQualificationCycleState::Blocked,
            ContainerQualificationCycleAction::None,
            None,
            None,
            Some("external artifact admission root is not configured"),
        )));
    };
    let path = root.join(format!("{tool_id}.json"));
    if !regular_file_exists(&path)? {
        return Ok(CurrentArtifactResolution::Blocked(unit(
            run_id,
            tool_id,
            ContainerQualificationCycleState::Blocked,
            ContainerQualificationCycleAction::None,
            None,
            None,
            Some("publish a governed external artifact admission or access-blocker record"),
        )));
    }
    let loaded = bijux_dna_containers::api::load_external_artifact_admission(&path)?;
    if loaded.tool_id != tool_id {
        bail!(
            "external artifact record {} selects {} instead of {tool_id}",
            path.display(),
            loaded.tool_id
        );
    }
    let admission = bijux_dna_containers::api::verify_external_artifact_admission(
        &request.repository_root,
        Path::new(&loaded.distributable_image_root),
        &path,
    )?;
    if admission.availability
        == bijux_dna_containers::api::ExternalArtifactAdmissionStatus::Admitted
    {
        let artifact = admission.artifact.context("admitted external artifact has no identity")?;
        return Ok(CurrentArtifactResolution::Ready(artifact.sha256));
    }
    let detail = match admission.availability {
        bijux_dna_containers::api::ExternalArtifactAdmissionStatus::BlockedExternalAccess => {
            format!(
                "external artifact access is blocked: {}",
                admission.blocker_reason.as_deref().unwrap_or("unspecified governed blocker")
            )
        }
        bijux_dna_containers::api::ExternalArtifactAdmissionStatus::Unavailable => format!(
            "external artifact is unavailable: {}",
            admission.blocker_reason.as_deref().unwrap_or("unspecified governed blocker")
        ),
        bijux_dna_containers::api::ExternalArtifactAdmissionStatus::Admitted => unreachable!(),
    };
    Ok(CurrentArtifactResolution::Blocked(unit(
        run_id,
        tool_id,
        ContainerQualificationCycleState::Blocked,
        ContainerQualificationCycleAction::None,
        None,
        None,
        Some(&detail),
    )))
}

fn publication_staleness_detail(
    evidence: &bijux_dna_containers::api::RunEvidenceLock,
    repository_commit: &str,
    artifact_sha256: &str,
) -> Result<Option<String>> {
    if evidence.execution_status != bijux_dna_containers::api::AttemptExecutionStatus::Passed {
        return Ok(Some("qualification publication execution is not passing".to_string()));
    }
    if evidence.repository_commit != repository_commit {
        return Ok(Some(format!(
            "qualification publication source {} differs from current repository commit {}",
            evidence.repository_commit, repository_commit
        )));
    }
    let published_sha256 = evidence
        .image
        .get("sha256")
        .and_then(serde_json::Value::as_str)
        .context("qualification publication image omits its SHA-256 identity")?;
    if published_sha256 != artifact_sha256 {
        return Ok(Some(format!(
            "qualification publication artifact {published_sha256} differs from current artifact {artifact_sha256}"
        )));
    }
    Ok(None)
}

fn discover_specifications(root: &Path) -> Result<BTreeMap<(String, String), Vec<PathBuf>>> {
    if !root.exists() {
        return Ok(BTreeMap::new());
    }
    let mut specifications = BTreeMap::new();
    for entry in WalkDir::new(root).follow_links(false).into_iter().filter_entry(|entry| {
        entry.depth() == 0
            || !entry.file_type().is_dir()
            || !matches!(
                entry.file_name().to_str(),
                Some(".attempt-archives" | ".attempt-archive-rollbacks")
            )
    }) {
        let entry = entry.with_context(|| format!("walk qualification root {}", root.display()))?;
        if !entry.file_type().is_file() || entry.file_name() != "qualification-spec.json" {
            continue;
        }
        let path = entry.path().to_path_buf();
        let identity: DiscoveredSpecificationIdentity =
            serde_json::from_slice(&fs::read(&path).with_context(|| {
                format!("read qualification specification identity {}", path.display())
            })?)
            .with_context(|| {
                format!("parse qualification specification identity {}", path.display())
            })?;
        let key = (identity.run_id, identity.tool_id);
        specifications.entry(key).or_insert_with(Vec::new).push(path);
    }
    Ok(specifications)
}

#[derive(Deserialize)]
struct DiscoveredSpecificationIdentity {
    run_id: String,
    tool_id: String,
}

fn classify_spec(
    path: &Path,
    spec: &ContainerQualificationSlurmSpec,
    repository_commit: &str,
    current_artifact_sha256: &str,
    preparation: Option<&ContainerQualificationCyclePreparation>,
    scheduler: Option<&SlurmClientPaths>,
    observed_at_utc: &str,
) -> Result<ContainerQualificationCycleUnit> {
    let staleness = specification_staleness_detail(
        spec,
        repository_commit,
        current_artifact_sha256,
        preparation,
    )?;
    if let Some(evidence) = load_container_qualification_scheduler_evidence(spec)? {
        if evidence.status == ContainerQualificationSchedulerStatus::Passed {
            if let Some(detail) = staleness.as_deref() {
                return Ok(unit(
                    &spec.run_id,
                    &spec.tool_id,
                    ContainerQualificationCycleState::Blocked,
                    ContainerQualificationCycleAction::ArchiveSuperseded,
                    Some(path),
                    Some(evidence.job_id),
                    Some(detail),
                ));
            }
        }
        let (state, action) = match evidence.status {
            ContainerQualificationSchedulerStatus::Passed => {
                if super::load_container_qualification_admission(spec)?.is_some() {
                    (
                        ContainerQualificationCycleState::Admitted,
                        ContainerQualificationCycleAction::Book,
                    )
                } else {
                    (
                        ContainerQualificationCycleState::Passed,
                        ContainerQualificationCycleAction::Admit,
                    )
                }
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
                ContainerQualificationCycleAction::Observe,
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
    if let Some(detail) = staleness.as_deref() {
        return Ok(unit(
            &spec.run_id,
            &spec.tool_id,
            ContainerQualificationCycleState::Blocked,
            ContainerQualificationCycleAction::ArchiveSuperseded,
            Some(path),
            None,
            Some(detail),
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

fn specification_staleness_detail(
    spec: &ContainerQualificationSlurmSpec,
    repository_commit: &str,
    current_artifact_sha256: &str,
    preparation: Option<&ContainerQualificationCyclePreparation>,
) -> Result<Option<String>> {
    if let Some(detail) = stale_specification_detail(spec, repository_commit) {
        return Ok(Some(detail));
    }
    let pipeline = bijux_dna_containers::api::load_container_pipeline(&spec.pipeline_path)?;
    let specification_artifact_sha256 = match pipeline {
        bijux_dna_containers::api::ContainerPipelineContract::Legacy(pipeline) => {
            pipeline.image.sha256
        }
        bijux_dna_containers::api::ContainerPipelineContract::Current(pipeline) => {
            pipeline.image.sha256
        }
    };
    if specification_artifact_sha256 != current_artifact_sha256 {
        return Ok(Some(format!(
            "qualification specification artifact {specification_artifact_sha256} does not match current artifact {current_artifact_sha256}"
        )));
    }
    if let Some(preparation) = preparation {
        for (expected_path, retained_sha256, label) in [
            (&preparation.governance_path, &spec.governance_sha256, "governance"),
            (
                &preparation.credential_reference_path,
                &spec.credential_reference_sha256,
                "credential reference",
            ),
        ] {
            let current_sha256 = bijux_dna_infra::hash_file_sha256(expected_path)?;
            if current_sha256 != *retained_sha256 {
                return Ok(Some(format!(
                    "qualification specification {label} differs from current preparation authority"
                )));
            }
        }
        if spec.hostname != preparation.hostname {
            return Ok(Some(
                "qualification specification hostname differs from current preparation authority"
                    .to_string(),
            ));
        }
    }
    Ok(None)
}

fn stale_specification_detail(
    spec: &ContainerQualificationSlurmSpec,
    repository_commit: &str,
) -> Option<String> {
    (spec.repository_commit != repository_commit).then(|| {
        format!(
            "qualification specification source {} does not match current repository commit {}",
            spec.repository_commit, repository_commit
        )
    })
}

fn scheduler_state(
    state: SlurmJobState,
) -> (ContainerQualificationCycleState, ContainerQualificationCycleAction, Option<&'static str>) {
    match state {
        SlurmJobState::Submitted => (
            ContainerQualificationCycleState::Submitted,
            ContainerQualificationCycleAction::Observe,
            None,
        ),
        SlurmJobState::Pending => (
            ContainerQualificationCycleState::Pending,
            ContainerQualificationCycleAction::Observe,
            None,
        ),
        SlurmJobState::Running => (
            ContainerQualificationCycleState::Running,
            ContainerQualificationCycleAction::Observe,
            None,
        ),
        SlurmJobState::AccountingPending => (
            ContainerQualificationCycleState::AccountingPending,
            ContainerQualificationCycleAction::Observe,
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
        admitted_count: count(units, &[ContainerQualificationCycleState::Admitted]),
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

fn validate_cycle_preparation(preparation: &ContainerQualificationCyclePreparation) -> Result<()> {
    if !matches!(
        preparation.schema_version.as_str(),
        CONTAINER_QUALIFICATION_CYCLE_PREPARATION_SCHEMA_V1
            | CONTAINER_QUALIFICATION_CYCLE_PREPARATION_SCHEMA_V2
    ) {
        bail!("unsupported container qualification cycle preparation schema");
    }
    if preparation.schema_version == CONTAINER_QUALIFICATION_CYCLE_PREPARATION_SCHEMA_V1
        && preparation.source_materialization.is_some()
    {
        bail!("v1 qualification preparation cannot declare source materialization");
    }
    if preparation.schema_version == CONTAINER_QUALIFICATION_CYCLE_PREPARATION_SCHEMA_V2
        && preparation.source_materialization.is_none()
    {
        bail!("v2 qualification preparation requires source materialization authority");
    }
    require_regular_directory(&preparation.sources_root, "qualification sources root")?;
    for (path, label) in [
        (&preparation.governance_path, "qualification governance"),
        (&preparation.credential_reference_path, "qualification credential reference"),
        (&preparation.executor_program_path, "qualification executor program"),
    ] {
        if !path.is_absolute() || !regular_file_exists(path)? {
            bail!("{label} is not an absolute regular file: {}", path.display());
        }
    }
    if preparation.hostname.is_empty()
        || !preparation
            .hostname
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_'))
    {
        bail!("qualification preparation hostname is invalid");
    }
    if let Some(materialization) = preparation.source_materialization.as_ref() {
        require_regular_directory(
            &materialization.selections_root,
            "qualification source selections root",
        )?;
        if !materialization.data_index_path.is_absolute()
            || !regular_file_exists(&materialization.data_index_path)?
        {
            bail!(
                "qualification data index is not an absolute regular file: {}",
                materialization.data_index_path.display()
            );
        }
        for (value, label) in
            [(&materialization.operator, "operator"), (&materialization.walltime, "walltime")]
        {
            if value.trim().is_empty() || value != value.trim() {
                bail!("qualification source materialization {label} is invalid");
            }
        }
    }
    Ok(())
}

fn require_regular_directory(path: &Path, label: &str) -> Result<()> {
    if !path.is_absolute() {
        bail!("{label} must be absolute");
    }
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => Ok(()),
        Ok(_) => bail!("{label} is not a regular directory: {}", path.display()),
        Err(error) => Err(error).with_context(|| format!("inspect {label} {}", path.display())),
    }
}

fn qualification_source_path(root: &Path, tool_id: &str) -> Result<Option<PathBuf>> {
    let mut sources = Vec::new();
    for extension in ["json", "yaml", "yml"] {
        let candidate = root.join(format!("{tool_id}.{extension}"));
        if regular_file_exists(&candidate)? {
            sources.push(candidate);
        }
    }
    match sources.as_slice() {
        [] => Ok(None),
        [source] => Ok(Some(source.clone())),
        _ => bail!("qualification source is ambiguous for tool {tool_id}"),
    }
}

fn ensure_bytes_identity(path: &Path, bytes: &[u8]) -> Result<()> {
    if regular_file_exists(path)? {
        let retained = bijux_dna_infra::read_to_end_bounded(path, 16 * 1024 * 1024)?;
        if retained != bytes {
            bail!("retained qualification pipeline identity conflicts at {}", path.display());
        }
        return Ok(());
    }
    bijux_dna_infra::atomic_create_bytes(path, bytes)
        .with_context(|| format!("publish qualification pipeline {}", path.display()))
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::super::{
        ContainerQualificationSlurmResources, ContainerQualificationSubmissionKind,
    };
    use super::{
        classify_spec, discover_specifications, stale_specification_detail, AdvancePass,
        ContainerQualificationCycleAction, ContainerQualificationCycleState,
        ContainerQualificationSlurmSpec, ADVANCE_PASS_ORDER,
    };

    #[test]
    fn advance_runs_the_complete_one_shot_frontier_in_dependency_order() {
        assert_eq!(
            ADVANCE_PASS_ORDER,
            [
                AdvancePass::Collect,
                AdvancePass::ArchiveFailure,
                AdvancePass::ArchiveSuperseded,
                AdvancePass::Admit,
                AdvancePass::Materialize,
                AdvancePass::Prepare,
                AdvancePass::Submit,
            ]
        );
    }

    #[test]
    fn specification_discovery_excludes_archived_attempts() -> anyhow::Result<()> {
        let root = bijux_dna_infra::temp_dir("qualification-cycle-discovery")?;
        let active = root.path().join("run-1150/attempt-002");
        let archived = root.path().join(".attempt-archives/run-1150/attempt-001");
        std::fs::create_dir_all(&active)?;
        std::fs::create_dir_all(&archived)?;
        let identity = br#"{"run_id":"run-1150","tool_id":"rcorrector"}"#;
        std::fs::write(active.join("qualification-spec.json"), identity)?;
        std::fs::write(archived.join("qualification-spec.json"), identity)?;

        let discovered = discover_specifications(root.path())?;

        assert_eq!(discovered.len(), 1);
        assert_eq!(
            discovered.get(&("run-1150".to_string(), "rcorrector".to_string())).map(Vec::len),
            Some(1)
        );
        Ok(())
    }

    #[test]
    fn stale_unsubmitted_specification_is_superseded_before_replacement() -> anyhow::Result<()> {
        let spec = ContainerQualificationSlurmSpec {
            schema_version: super::super::CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V5.to_string(),
            submission_kind: ContainerQualificationSubmissionKind::default(),
            historical_replay_source: None,
            run_id: "run-1150".to_string(),
            tool_id: "rcorrector".to_string(),
            campaign_id: Some("retained-container-qualification".to_string()),
            attempt_id: Some("attempt-003".to_string()),
            job_name: "qualification-run-1150".to_string(),
            hostname: "lunarc".to_string(),
            repository_path: PathBuf::from("/repository"),
            repository_commit: "1".repeat(40),
            governance_path: PathBuf::from("/governance.json"),
            governance_sha256: "2".repeat(64),
            pipeline_path: PathBuf::from("/pipeline.json"),
            pipeline_sha256: "3".repeat(64),
            credential_reference_path: PathBuf::from("/credential.json"),
            credential_reference_sha256: "4".repeat(64),
            executor_program_path: PathBuf::from("/executor"),
            executor_program_sha256: "5".repeat(64),
            result_root: PathBuf::from("/result"),
            evidence_root: PathBuf::from("/evidence"),
            resources: ContainerQualificationSlurmResources {
                account: "account".to_string(),
                partition: "partition".to_string(),
                cpus: 1,
                memory_mb: 1024,
                walltime: "00:10:00".to_string(),
                scratch_mb: 1024,
            },
            dependencies: Vec::new(),
        };

        let unit = classify_spec(
            Path::new("/evidence/qualification-spec.json"),
            &spec,
            &"6".repeat(40),
            &"7".repeat(64),
            None,
            None,
            "2026-08-09T23:14:00Z",
        )?;

        assert_eq!(unit.state, ContainerQualificationCycleState::Blocked);
        assert_eq!(unit.next_action, ContainerQualificationCycleAction::ArchiveSuperseded);
        assert!(unit.detail.is_some_and(|detail| {
            detail.contains(&"1".repeat(40)) && detail.contains(&"6".repeat(40))
        }));
        assert!(stale_specification_detail(&spec, &spec.repository_commit).is_none());
        Ok(())
    }

    #[test]
    fn observation_action_serializes_without_polling_semantics() -> anyhow::Result<()> {
        assert_eq!(
            serde_json::to_string(&ContainerQualificationCycleAction::Observe)?,
            r#""observe""#
        );
        assert_eq!(
            serde_json::from_str::<ContainerQualificationCycleAction>(r#""poll""#)?,
            ContainerQualificationCycleAction::Observe
        );
        Ok(())
    }

    #[test]
    fn historical_poll_timestamp_deserializes_as_observation_cadence() -> anyhow::Result<()> {
        let status: super::ContainerQualificationCycleStatus =
            serde_json::from_value(serde_json::json!({
                "schema_version": super::CONTAINER_QUALIFICATION_CYCLE_STATUS_SCHEMA_V1,
                "observed_at_utc": "2026-08-10T12:00:00Z",
                "earliest_poll_at_utc": "2026-08-10T12:05:00Z",
                "summary": {
                    "unit_count": 0,
                    "published_count": 0,
                    "blocked_count": 0,
                    "ready_count": 0,
                    "active_count": 0,
                    "passed_count": 0,
                    "admitted_count": 0,
                    "failed_count": 0
                },
                "units": []
            }))?;

        assert_eq!(status.earliest_observation_at_utc.as_deref(), Some("2026-08-10T12:05:00Z"));
        Ok(())
    }

    #[test]
    fn stale_artifact_specification_is_superseded_before_replacement() -> anyhow::Result<()> {
        let repository_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let repository_commit =
            bijux_dna_containers::api::repository_head_commit(&repository_root)?;
        let pipeline_path =
            repository_root.join("containers/runs/run-1098-atropos/config-pipeline.yml");
        let spec = ContainerQualificationSlurmSpec {
            schema_version: super::super::CONTAINER_QUALIFICATION_SLURM_SPEC_SCHEMA_V5.to_string(),
            submission_kind: ContainerQualificationSubmissionKind::default(),
            historical_replay_source: None,
            run_id: "run-1098".to_string(),
            tool_id: "atropos".to_string(),
            campaign_id: Some("public-fastq-adapter-trim-qualification".to_string()),
            attempt_id: Some("attempt-001".to_string()),
            job_name: "qualification-run-1098".to_string(),
            hostname: "lunarc".to_string(),
            repository_path: repository_root,
            repository_commit: repository_commit.clone(),
            governance_path: PathBuf::from("/governance.json"),
            governance_sha256: "2".repeat(64),
            pipeline_path,
            pipeline_sha256: "3".repeat(64),
            credential_reference_path: PathBuf::from("/credential.json"),
            credential_reference_sha256: "4".repeat(64),
            executor_program_path: PathBuf::from("/executor"),
            executor_program_sha256: "5".repeat(64),
            result_root: PathBuf::from("/result"),
            evidence_root: PathBuf::from("/evidence"),
            resources: ContainerQualificationSlurmResources {
                account: "account".to_string(),
                partition: "partition".to_string(),
                cpus: 1,
                memory_mb: 1024,
                walltime: "00:10:00".to_string(),
                scratch_mb: 1024,
            },
            dependencies: Vec::new(),
        };

        let unit = classify_spec(
            Path::new("/evidence/qualification-spec.json"),
            &spec,
            &repository_commit,
            &"7".repeat(64),
            None,
            None,
            "2026-08-09T23:14:00Z",
        )?;

        assert_eq!(unit.state, ContainerQualificationCycleState::Blocked);
        assert_eq!(unit.next_action, ContainerQualificationCycleAction::ArchiveSuperseded);
        assert!(unit.detail.is_some_and(|detail| {
            detail.contains("qualification specification artifact")
                && detail.contains("does not match current artifact")
        }));
        Ok(())
    }
}
