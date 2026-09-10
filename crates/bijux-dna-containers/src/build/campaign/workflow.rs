//! Owner: admitted image campaign and scheduler-attempt orchestration.
use super::admission::{
    campaign_scope, execution_from_tools, select_campaign_tools, validate_attempt_identity,
};
use super::execution::{execute_campaign_tools, CampaignExecutionIdentity};
use super::model::{
    ApptainerImageCampaignRequest, ImageCampaignAttemptRequest, ImageCampaignExecution,
    ImageCampaignJobCompletion, ImageCampaignPlan, IMAGE_CAMPAIGN_JOB_COMPLETION_SCHEMA_V1,
    IMAGE_CAMPAIGN_PLAN_SCHEMA_V1,
};
use super::paths::{campaign_root, portable_study_path};
use super::storage::{ensure_immutable_json, load_progress, publish_campaign_execution};
use super::verification::verify_resumed_outcomes;
use crate::evidence::tool_contract::{
    clean_repository_head_commit, observe_container_catalog, retained_apptainer_tools_from_catalog,
    validate_observed_container_catalog,
};
use anyhow::{bail, Context, Result};
use std::{fs, sync::Mutex};
/// Build or resume every repository-distributable image and publish an immutable terminal record.
///
/// # Errors
/// Returns an error for source drift, invalid catalog closure, conflicting progress, invalid
/// resumed evidence, worker failure, or publication failure. Individual tool failures are retained
/// in the returned execution and do not prevent other tools from running.
pub fn execute_apptainer_image_campaign(
    request: &ApptainerImageCampaignRequest,
) -> Result<ImageCampaignExecution> {
    if request.worker_count == 0 || request.worker_count > 32 {
        bail!("image campaign worker count must be between 1 and 32");
    }
    let commit = clean_repository_head_commit(&request.repository_root)?;
    let observation = observe_container_catalog(&request.repository_root)?;
    let catalog = &observation.catalog;
    let validated = validate_observed_container_catalog(&request.repository_root, catalog)?;
    if !validated.is_valid() {
        bail!("image campaign catalog validation failed: {}", validated.errors.join("; "));
    }
    let redistributable_tools =
        retained_apptainer_tools_from_catalog(catalog, &catalog.apptainer_architecture)?;
    if redistributable_tools.len() != validated.redistributable_image_count {
        bail!("image campaign redistributable-image roster differs from validated catalog");
    }
    let tools = select_campaign_tools(&redistributable_tools, request.selected_tools.as_ref())?;
    let catalog_sha256 = observation.sha256;
    let scope = campaign_scope(&tools, redistributable_tools.len());
    let evidence_root = campaign_root(
        &request.study_root.join("containers/evidence/image-campaigns"),
        &commit,
        scope.as_deref(),
    );
    let image_root =
        campaign_root(&request.study_root.join("containers/staging"), &commit, scope.as_deref());
    bijux_dna_infra::ensure_dir(&evidence_root)?;
    let _lock = bijux_dna_infra::FileLock::try_acquire(&evidence_root.join("campaign.lock"))?;
    let plan = ImageCampaignPlan {
        schema_version: IMAGE_CAMPAIGN_PLAN_SCHEMA_V1.to_string(),
        repository_commit: commit.clone(),
        catalog_sha256: catalog_sha256.clone(),
        architecture: catalog.apptainer_architecture.clone(),
        image_root: portable_study_path(&image_root, &request.study_root)?,
        evidence_root: portable_study_path(&evidence_root, &request.study_root)?,
        worker_count: request.worker_count,
        tools: tools.clone(),
    };
    ensure_immutable_json(&evidence_root.join("plan.json"), &plan)?;
    let progress_path = evidence_root.join("progress.json");
    let outcomes =
        load_progress(&progress_path, &commit, &catalog_sha256, &catalog.apptainer_architecture)?;
    if outcomes.keys().any(|tool| !tools.contains(tool)) {
        bail!("image campaign progress contains a tool outside the redistributable-image roster");
    }
    let outcomes = Mutex::new(outcomes);
    let identity = CampaignExecutionIdentity {
        repository_commit: &commit,
        catalog_sha256: &catalog_sha256,
        architecture: &catalog.apptainer_architecture,
        scope: scope.as_deref(),
    };
    verify_resumed_outcomes(request, &identity, &outcomes)?;
    execute_campaign_tools(request, &tools, &outcomes, &progress_path, &identity)?;
    let tools = outcomes
        .into_inner()
        .map_err(|_| anyhow::anyhow!("image campaign progress lock poisoned"))?;
    let execution =
        execution_from_tools(commit, catalog_sha256, catalog.apptainer_architecture.clone(), tools);
    publish_campaign_execution(&evidence_root, &execution)?;
    Ok(execution)
}

/// Execute one image campaign attempt and atomically publish its scheduler completion receipt.
///
/// # Errors
/// Returns an error when attempt identity is invalid, campaign execution fails, or immutable
/// attempt publications conflict.
pub fn execute_apptainer_image_campaign_attempt(
    request: &ImageCampaignAttemptRequest,
) -> Result<ImageCampaignJobCompletion> {
    validate_attempt_identity(request)?;
    let metadata = fs::symlink_metadata(&request.attempt_root)
        .context("inspect image campaign attempt root")?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("image campaign attempt root is not a regular directory");
    }
    let execution = execute_apptainer_image_campaign(&request.campaign)?;
    let execution_name = "campaign-execution.json";
    let execution_path = request.attempt_root.join(execution_name);
    ensure_immutable_json(&execution_path, &execution)?;
    let completion = ImageCampaignJobCompletion {
        schema_version: IMAGE_CAMPAIGN_JOB_COMPLETION_SCHEMA_V1.to_string(),
        attempt_id: request.attempt_id.clone(),
        repository_commit: execution.repository_commit,
        slurm_job_id: request.slurm_job_id.clone(),
        completed_at: request.completed_at.clone(),
        campaign_execution: execution_name.to_string(),
        campaign_execution_sha256: bijux_dna_infra::hash_file_sha256(&execution_path)?,
    };
    ensure_immutable_json(&request.attempt_root.join("completion.json"), &completion)?;
    Ok(completion)
}
