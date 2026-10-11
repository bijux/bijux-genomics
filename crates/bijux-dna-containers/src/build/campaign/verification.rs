//! Owner: terminal receipt and resumed image verification against current source and runtime observations.
use super::admission::{
    campaign_scope, execution_from_tools, selected_receipt_tools, valid_sha256,
};
use super::execution::CampaignExecutionIdentity;
use super::model::{
    ApptainerImageCampaignRequest, ImageCampaignExecution, ImageCampaignJobCompletion,
    ImageCampaignToolOutcome, ImageCampaignToolStatus, IMAGE_CAMPAIGN_JOB_COMPLETION_SCHEMA_V1,
};
use super::paths::{campaign_root, require_campaign_path, resolve_study_path};
use super::storage::load_progress;
use crate::build::CanonicalApptainerImagePublication;
use crate::evidence::tool_contract::{
    clean_repository_head_commit, derive_apptainer_tool_contract, observe_container_catalog,
    retained_apptainer_tools_from_catalog, validate_observed_container_catalog,
    validate_tool_image_identity_evidence, verify_apptainer_tool_contract,
    ToolImageIdentityEvidence,
};
use anyhow::{bail, Context, Result};
use chrono::DateTime;
use std::collections::BTreeMap;
use std::{fs, path::Path, sync::Mutex};
/// Admit a terminal Slurm campaign receipt and every publication it references.
///
/// # Errors
/// Returns an error for receipt, source, catalog, roster, publication, image, or runtime drift.
pub fn verify_apptainer_image_campaign_receipt(
    repository_root: &Path,
    study_root: &Path,
    apptainer_executable: &Path,
    attempt_root: &Path,
) -> Result<ImageCampaignExecution> {
    let completion_path = attempt_root.join("completion.json");
    let completion: ImageCampaignJobCompletion = serde_json::from_slice(
        &fs::read(&completion_path)
            .with_context(|| format!("read {}", completion_path.display()))?,
    )
    .with_context(|| format!("parse {}", completion_path.display()))?;
    if completion.schema_version != IMAGE_CAMPAIGN_JOB_COMPLETION_SCHEMA_V1
        || completion.attempt_id.trim().is_empty()
        || completion.slurm_job_id.is_empty()
        || !completion.slurm_job_id.bytes().all(|byte| byte.is_ascii_digit())
        || DateTime::parse_from_rfc3339(&completion.completed_at).is_err()
        || completion.campaign_execution != "campaign-execution.json"
        || !valid_sha256(&completion.campaign_execution_sha256)
    {
        bail!("image campaign completion identity is invalid");
    }
    let execution_path = attempt_root.join(&completion.campaign_execution);
    if bijux_dna_infra::hash_file_sha256(&execution_path)? != completion.campaign_execution_sha256 {
        bail!("image campaign execution digest differs from completion");
    }

    let source_commit = clean_repository_head_commit(repository_root)?;
    if completion.repository_commit != source_commit {
        bail!("image campaign completion differs from frozen source commit");
    }
    let observation = observe_container_catalog(repository_root)?;
    let catalog = &observation.catalog;
    let validated = validate_observed_container_catalog(repository_root, catalog)?;
    if !validated.is_valid() {
        bail!("image campaign catalog validation failed: {}", validated.errors.join("; "));
    }
    let catalog_sha256 = observation.sha256;
    let tools = load_progress(
        &execution_path,
        &source_commit,
        &catalog_sha256,
        &catalog.apptainer_architecture,
    )?;
    let redistributable_tools =
        retained_apptainer_tools_from_catalog(catalog, &catalog.apptainer_architecture)?;
    let selected_tools = selected_receipt_tools(&redistributable_tools, tools.keys())?;
    if tools.values().any(|outcome| {
        outcome.status != ImageCampaignToolStatus::Passed
            || outcome.publication.is_none()
            || outcome.error.is_some()
    }) {
        bail!("image campaign receipt does not close the selected tool roster");
    }
    let execution = execution_from_tools(
        source_commit.clone(),
        catalog_sha256.clone(),
        catalog.apptainer_architecture.clone(),
        tools.clone(),
    );
    let scope = campaign_scope(&selected_tools, redistributable_tools.len());
    let canonical_path = campaign_root(
        &study_root.join("containers/evidence/image-campaigns"),
        &source_commit,
        scope.as_deref(),
    )
    .join("execution.json");
    let canonical: ImageCampaignExecution = serde_json::from_slice(
        &fs::read(&canonical_path).with_context(|| format!("read {}", canonical_path.display()))?,
    )?;
    if canonical != execution {
        bail!("image campaign receipt differs from canonical execution publication");
    }
    verify_resumed_outcomes(
        &ApptainerImageCampaignRequest {
            repository_root: repository_root.to_path_buf(),
            study_root: study_root.to_path_buf(),
            apptainer_executable: apptainer_executable.to_path_buf(),
            worker_count: 1,
            selected_tools: Some(selected_tools.into_iter().collect()),
        },
        &CampaignExecutionIdentity {
            architecture: &catalog.apptainer_architecture,
            repository_commit: &source_commit,
            catalog_sha256: &catalog_sha256,
            scope: scope.as_deref(),
        },
        &Mutex::new(tools),
    )?;
    Ok(execution)
}

pub(super) fn verify_resumed_outcomes(
    request: &ApptainerImageCampaignRequest,
    identity: &CampaignExecutionIdentity<'_>,
    outcomes: &Mutex<BTreeMap<String, ImageCampaignToolOutcome>>,
) -> Result<()> {
    let CampaignExecutionIdentity { architecture, repository_commit, catalog_sha256, scope } =
        *identity;
    let expected_image_root =
        campaign_root(&request.study_root.join("containers/staging"), repository_commit, scope);
    let expected_evidence_root = campaign_root(
        &request.study_root.join("containers/evidence/image-campaigns"),
        repository_commit,
        scope,
    );
    let items =
        outcomes.lock().map_err(|_| anyhow::anyhow!("image campaign progress lock poisoned"))?;
    for (tool, outcome) in
        items.iter().filter(|(_, item)| item.status == ImageCampaignToolStatus::Passed)
    {
        let publication =
            outcome.publication.as_ref().context("passed campaign outcome omits publication")?;
        if publication.tool != *tool
            || publication.status != "passed"
            || publication.repository_commit != repository_commit
            || publication.catalog_sha256 != catalog_sha256
            || publication.architecture != architecture
        {
            bail!("resumed build publication identity differs for {tool}");
        }
        let contract = derive_apptainer_tool_contract(
            &request.repository_root,
            tool,
            repository_commit,
            architecture,
        )?;
        if publication.version != contract.version
            || publication.build_evidence_id != contract.build_evidence_id
            || publication.source_sha256 != contract.source_sha256
            || publication.pinned_commit != contract.pinned_commit
            || publication.interface_sha256 != contract.interface_sha256
        {
            bail!("resumed build publication contract differs for {tool}");
        }
        let image_path = resolve_study_path(&request.study_root, &publication.image.path)?;
        require_campaign_path(&image_path, &expected_image_root, "image", tool)?;
        if fs::metadata(&image_path)?.len() != publication.image.size_bytes {
            bail!("resumed image size differs for {tool}");
        }
        let observed = verify_apptainer_tool_contract(
            &request.apptainer_executable,
            &image_path,
            &publication.image.sha256,
            &contract,
        )?;
        let identity_path = resolve_study_path(&request.study_root, &publication.identity.path)?;
        require_campaign_path(&identity_path, &expected_evidence_root, "identity", tool)?;
        if bijux_dna_infra::hash_file_sha256(&identity_path)? != publication.identity.sha256 {
            bail!("resumed identity digest differs for {tool}");
        }
        let persisted: ToolImageIdentityEvidence =
            serde_json::from_slice(&fs::read(identity_path)?)?;
        validate_tool_image_identity_evidence(&persisted, &publication.image.sha256, &contract)?;
        if persisted != observed {
            bail!("resumed image inspection differs for {tool}");
        }
        let evidence_path = resolve_study_path(&request.study_root, &publication.evidence_path)?;
        require_campaign_path(&evidence_path, &expected_evidence_root, "evidence", tool)?;
        let persisted_publication: CanonicalApptainerImagePublication =
            serde_json::from_slice(&fs::read(evidence_path)?)?;
        if persisted_publication != *publication {
            bail!("resumed build evidence differs for {tool}");
        }
        let build_log_path = resolve_study_path(&request.study_root, &publication.build_log.path)?;
        require_campaign_path(&build_log_path, &expected_evidence_root, "build log", tool)?;
        if bijux_dna_infra::hash_file_sha256(&build_log_path)? != publication.build_log.sha256 {
            bail!("resumed build log digest differs for {tool}");
        }
    }
    Ok(())
}
