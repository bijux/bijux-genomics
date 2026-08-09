use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use anyhow::{bail, Context, Result};
use chrono::DateTime;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::evidence::tool_contract::{
    clean_repository_head_commit, derive_apptainer_tool_contract, load_container_catalog,
    retained_apptainer_tool_ids, validate_container_catalog, validate_tool_image_identity_evidence,
    verify_apptainer_tool_contract, ToolImageIdentityEvidence,
};

use super::{
    publish_canonical_apptainer_image, CanonicalApptainerImagePublication,
    CanonicalApptainerImagePublicationRequest,
};

pub const IMAGE_CAMPAIGN_PLAN_SCHEMA_V1: &str = "bijux.container.image_campaign_plan.v1";
pub const IMAGE_CAMPAIGN_EXECUTION_SCHEMA_V1: &str = "bijux.container.image_campaign_execution.v1";
pub const IMAGE_CAMPAIGN_JOB_COMPLETION_SCHEMA_V1: &str = "bijux.image_campaign_job_completion.v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApptainerImageCampaignRequest {
    pub repository_root: PathBuf,
    pub study_root: PathBuf,
    pub apptainer_executable: PathBuf,
    pub worker_count: usize,
    /// Optional retained-tool closure for an identity-addressed targeted campaign.
    pub selected_tools: Option<BTreeSet<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageCampaignPlan {
    pub schema_version: String,
    pub repository_commit: String,
    pub catalog_sha256: String,
    pub architecture: String,
    pub image_root: String,
    pub evidence_root: String,
    pub worker_count: usize,
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ImageCampaignToolStatus {
    Passed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageCampaignToolOutcome {
    pub status: ImageCampaignToolStatus,
    pub publication: Option<CanonicalApptainerImagePublication>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageCampaignExecution {
    pub schema_version: String,
    pub repository_commit: String,
    pub catalog_sha256: String,
    pub architecture: String,
    pub tools: BTreeMap<String, ImageCampaignToolOutcome>,
    pub passed: usize,
    pub failed: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImageCampaignJobCompletion {
    pub schema_version: String,
    pub attempt_id: String,
    pub repository_commit: String,
    pub slurm_job_id: String,
    pub completed_at: String,
    pub campaign_execution: String,
    pub campaign_execution_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageCampaignAttemptRequest {
    pub campaign: ApptainerImageCampaignRequest,
    pub attempt_root: PathBuf,
    pub attempt_id: String,
    pub slurm_job_id: String,
    pub completed_at: String,
}

#[must_use]
pub fn image_campaign_plan_schema() -> Value {
    serde_json::to_value(schema_for!(ImageCampaignPlan)).unwrap_or_else(|_| serde_json::json!({}))
}

#[must_use]
pub fn image_campaign_execution_schema() -> Value {
    serde_json::to_value(schema_for!(ImageCampaignExecution))
        .unwrap_or_else(|_| serde_json::json!({}))
}

/// Write the retained-image campaign plan schema atomically.
///
/// # Errors
/// Returns an error when the destination cannot be written.
pub fn write_image_campaign_plan_schema(path: &Path) -> Result<()> {
    bijux_dna_infra::atomic_write_json(path, &image_campaign_plan_schema())?;
    Ok(())
}

/// Write the retained-image campaign execution schema atomically.
///
/// # Errors
/// Returns an error when the destination cannot be written.
pub fn write_image_campaign_execution_schema(path: &Path) -> Result<()> {
    bijux_dna_infra::atomic_write_json(path, &image_campaign_execution_schema())?;
    Ok(())
}

/// Build or resume every retained image and publish an immutable terminal execution record.
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
    let catalog = load_container_catalog(&request.repository_root)?;
    let validated = validate_container_catalog(&request.repository_root)?;
    if !validated.is_valid() {
        bail!("image campaign catalog validation failed: {}", validated.errors.join("; "));
    }
    let retained_tools =
        retained_apptainer_tool_ids(&request.repository_root, &catalog.apptainer_architecture)?;
    if retained_tools.len() != validated.retained_count {
        bail!("image campaign retained roster differs from validated catalog");
    }
    let tools = select_campaign_tools(&retained_tools, request.selected_tools.as_ref())?;
    let catalog_sha256 = bijux_dna_infra::hash_file_sha256(
        &request.repository_root.join("containers/contracts/catalog.toml"),
    )?;
    let scope = campaign_scope(&tools, retained_tools.len());
    let evidence_root = campaign_root(
        &request.study_root.join("containers/evidence/image-campaigns"),
        &commit,
        scope.as_deref(),
    );
    let image_root =
        campaign_root(&request.study_root.join("containers/staging"), &commit, scope.as_deref());
    bijux_dna_infra::ensure_dir(&evidence_root)?;
    let _lock = bijux_dna_infra::FileLock::acquire(
        &evidence_root.join("campaign.lock"),
        std::time::Duration::from_secs(30),
    )?;
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
        bail!("image campaign progress contains a tool outside the retained roster");
    }
    let outcomes = Mutex::new(outcomes);
    verify_resumed_outcomes(
        request,
        &catalog.apptainer_architecture,
        &commit,
        &catalog_sha256,
        &outcomes,
    )?;
    let identity = CampaignExecutionIdentity {
        repository_commit: &commit,
        catalog_sha256: &catalog_sha256,
        architecture: &catalog.apptainer_architecture,
        scope: scope.as_deref(),
    };
    execute_campaign_tools(request, &tools, &outcomes, &progress_path, &identity)?;
    let tools = outcomes
        .into_inner()
        .map_err(|_| anyhow::anyhow!("image campaign progress lock poisoned"))?;
    let execution =
        execution_from_tools(commit, catalog_sha256, catalog.apptainer_architecture, tools);
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

fn validate_attempt_identity(request: &ImageCampaignAttemptRequest) -> Result<()> {
    if request.attempt_id.is_empty()
        || request.attempt_id.contains('/')
        || request.attempt_id.contains('\\')
        || request.attempt_id == "."
        || request.attempt_id == ".."
    {
        bail!("image campaign attempt identity is invalid");
    }
    if request.slurm_job_id.is_empty()
        || !request.slurm_job_id.bytes().all(|byte| byte.is_ascii_digit())
    {
        bail!("image campaign Slurm job identity is invalid");
    }
    DateTime::parse_from_rfc3339(&request.completed_at)
        .context("image campaign completion timestamp is invalid")?;
    Ok(())
}

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
    let catalog = load_container_catalog(repository_root)?;
    let validated = validate_container_catalog(repository_root)?;
    if !validated.is_valid() {
        bail!("image campaign catalog validation failed: {}", validated.errors.join("; "));
    }
    let catalog_sha256 = bijux_dna_infra::hash_file_sha256(
        &repository_root.join("containers/contracts/catalog.toml"),
    )?;
    let tools = load_progress(
        &execution_path,
        &source_commit,
        &catalog_sha256,
        &catalog.apptainer_architecture,
    )?;
    let retained = retained_apptainer_tool_ids(repository_root, &catalog.apptainer_architecture)?
        .into_iter()
        .collect::<BTreeSet<_>>();
    if tools.keys().cloned().collect::<BTreeSet<_>>() != retained
        || tools.len() != validated.retained_count
        || tools.values().any(|outcome| {
            outcome.status != ImageCampaignToolStatus::Passed
                || outcome.publication.is_none()
                || outcome.error.is_some()
        })
    {
        bail!("image campaign receipt does not close the retained tool roster");
    }
    let execution = execution_from_tools(
        source_commit.clone(),
        catalog_sha256.clone(),
        catalog.apptainer_architecture.clone(),
        tools.clone(),
    );
    let retained_tools =
        retained_apptainer_tool_ids(repository_root, &catalog.apptainer_architecture)?;
    let execution_tools = execution.tools.keys().cloned().collect::<Vec<_>>();
    let execution_tool_set = execution_tools.iter().cloned().collect::<BTreeSet<_>>();
    let selected_tools = select_campaign_tools(&retained_tools, Some(&execution_tool_set))?;
    let scope = campaign_scope(&selected_tools, retained_tools.len());
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
        &catalog.apptainer_architecture,
        &source_commit,
        &catalog_sha256,
        &Mutex::new(tools),
    )?;
    Ok(execution)
}

fn select_campaign_tools(
    retained_tools: &[String],
    selected_tools: Option<&BTreeSet<String>>,
) -> Result<Vec<String>> {
    let Some(selected_tools) = selected_tools else {
        return Ok(retained_tools.to_vec());
    };
    if selected_tools.is_empty() {
        bail!("selected image campaign contains no tools");
    }
    let retained = retained_tools.iter().cloned().collect::<BTreeSet<_>>();
    let unknown = selected_tools.difference(&retained).cloned().collect::<Vec<_>>();
    if !unknown.is_empty() {
        bail!("selected image campaign contains unknown retained tools: {}", unknown.join(","));
    }
    Ok(retained_tools.iter().filter(|tool| selected_tools.contains(*tool)).cloned().collect())
}

fn campaign_scope(tools: &[String], retained_count: usize) -> Option<String> {
    (tools.len() != retained_count).then(|| {
        format!("selected-{}", bijux_dna_infra::hash_bytes_sha256(tools.join("\n").as_bytes()))
    })
}

fn campaign_root(base: &Path, commit: &str, scope: Option<&str>) -> PathBuf {
    let root = base.join(commit);
    scope.map_or(root.clone(), |scope| root.join(scope))
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

struct CampaignExecutionIdentity<'a> {
    repository_commit: &'a str,
    catalog_sha256: &'a str,
    architecture: &'a str,
    scope: Option<&'a str>,
}

fn execute_campaign_tools(
    request: &ApptainerImageCampaignRequest,
    tools: &[String],
    outcomes: &Mutex<BTreeMap<String, ImageCampaignToolOutcome>>,
    progress_path: &Path,
    identity: &CampaignExecutionIdentity<'_>,
) -> Result<()> {
    let passed = outcomes
        .lock()
        .map_err(|_| anyhow::anyhow!("image campaign progress lock poisoned"))?
        .iter()
        .filter(|(_, item)| item.status == ImageCampaignToolStatus::Passed)
        .map(|(tool, _)| tool.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let pending = tools.iter().filter(|tool| !passed.contains(*tool)).cloned().collect::<Vec<_>>();
    let next = AtomicUsize::new(0);
    std::thread::scope(|scope| -> Result<()> {
        let mut handles = Vec::new();
        for _ in 0..request.worker_count.min(pending.len()) {
            handles.push(scope.spawn(|| -> Result<()> {
                while let Some(tool) = pending.get(next.fetch_add(1, Ordering::Relaxed)) {
                    let mut build_request =
                        CanonicalApptainerImagePublicationRequest::retained_campaign(
                            request.repository_root.clone(),
                            request.study_root.clone(),
                            tool.clone(),
                        );
                    build_request.apptainer_executable.clone_from(&request.apptainer_executable);
                    build_request.campaign_scope = identity.scope.map(str::to_string);
                    let outcome = match publish_canonical_apptainer_image(&build_request) {
                        Ok(publication) => ImageCampaignToolOutcome {
                            status: ImageCampaignToolStatus::Passed,
                            publication: Some(publication),
                            error: None,
                        },
                        Err(error) => ImageCampaignToolOutcome {
                            status: ImageCampaignToolStatus::Failed,
                            publication: None,
                            error: Some(format!("{error:#}")),
                        },
                    };
                    let mut items = outcomes
                        .lock()
                        .map_err(|_| anyhow::anyhow!("image campaign progress lock poisoned"))?;
                    items.insert(tool.clone(), outcome);
                    write_progress(
                        progress_path,
                        identity.repository_commit,
                        identity.catalog_sha256,
                        identity.architecture,
                        &items,
                    )?;
                }
                Ok(())
            }));
        }
        for handle in handles {
            handle.join().map_err(|_| anyhow::anyhow!("image campaign worker panicked"))??;
        }
        Ok(())
    })
}

fn verify_resumed_outcomes(
    request: &ApptainerImageCampaignRequest,
    architecture: &str,
    repository_commit: &str,
    catalog_sha256: &str,
    outcomes: &Mutex<BTreeMap<String, ImageCampaignToolOutcome>>,
) -> Result<()> {
    let catalog = load_container_catalog(&request.repository_root)?;
    let retained_tools =
        retained_apptainer_tool_ids(&request.repository_root, &catalog.apptainer_architecture)?;
    let selected_tools = select_campaign_tools(&retained_tools, request.selected_tools.as_ref())?;
    let scope = campaign_scope(&selected_tools, retained_tools.len());
    let expected_image_root = campaign_root(
        &request.study_root.join("containers/staging"),
        repository_commit,
        scope.as_deref(),
    );
    let expected_evidence_root = campaign_root(
        &request.study_root.join("containers/evidence/image-campaigns"),
        repository_commit,
        scope.as_deref(),
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

fn require_campaign_path(path: &Path, root: &Path, label: &str, tool: &str) -> Result<()> {
    if path == root || !path.starts_with(root) {
        bail!("resumed {label} path is outside the campaign scope for {tool}");
    }
    Ok(())
}

fn load_progress(
    path: &Path,
    commit: &str,
    catalog_sha256: &str,
    architecture: &str,
) -> Result<BTreeMap<String, ImageCampaignToolOutcome>> {
    if !path.is_file() {
        return Ok(BTreeMap::new());
    }
    let execution: ImageCampaignExecution = serde_json::from_slice(&fs::read(path)?)?;
    if execution.schema_version != IMAGE_CAMPAIGN_EXECUTION_SCHEMA_V1
        || execution.repository_commit != commit
        || execution.catalog_sha256 != catalog_sha256
        || execution.architecture != architecture
    {
        bail!("image campaign progress identity differs from current source");
    }
    let expected = execution_from_tools(
        commit.to_string(),
        catalog_sha256.to_string(),
        architecture.to_string(),
        execution.tools.clone(),
    );
    if execution.passed != expected.passed || execution.failed != expected.failed {
        bail!("image campaign progress counts differ from tool outcomes");
    }
    for (tool, outcome) in &execution.tools {
        match outcome.status {
            ImageCampaignToolStatus::Passed
                if outcome.publication.is_some() && outcome.error.is_none() => {}
            ImageCampaignToolStatus::Failed
                if outcome.publication.is_none()
                    && outcome.error.as_ref().is_some_and(|error| !error.trim().is_empty()) => {}
            _ => bail!("image campaign progress outcome is incoherent for {tool}"),
        }
    }
    Ok(execution.tools)
}

fn write_progress(
    path: &Path,
    commit: &str,
    catalog_sha256: &str,
    architecture: &str,
    tools: &BTreeMap<String, ImageCampaignToolOutcome>,
) -> Result<()> {
    let execution = execution_from_tools(
        commit.to_string(),
        catalog_sha256.to_string(),
        architecture.to_string(),
        tools.clone(),
    );
    Ok(bijux_dna_infra::atomic_write_json(path, &execution)?)
}

fn execution_from_tools(
    repository_commit: String,
    catalog_sha256: String,
    architecture: String,
    tools: BTreeMap<String, ImageCampaignToolOutcome>,
) -> ImageCampaignExecution {
    let passed =
        tools.values().filter(|item| item.status == ImageCampaignToolStatus::Passed).count();
    let failed =
        tools.values().filter(|item| item.status == ImageCampaignToolStatus::Failed).count();
    ImageCampaignExecution {
        schema_version: IMAGE_CAMPAIGN_EXECUTION_SCHEMA_V1.to_string(),
        repository_commit,
        catalog_sha256,
        architecture,
        tools,
        passed,
        failed,
    }
}

fn publish_campaign_execution(
    evidence_root: &Path,
    execution: &ImageCampaignExecution,
) -> Result<()> {
    let attempts_root = evidence_root.join("executions");
    bijux_dna_infra::ensure_dir(&attempts_root)?;
    for attempt in 1..=999_u16 {
        let path = attempts_root.join(format!("execution-{attempt:03}.json"));
        match bijux_dna_infra::atomic_create_json(&path, execution) {
            Ok(()) => {
                if execution.failed == 0 {
                    ensure_immutable_json(&evidence_root.join("execution.json"), execution)?;
                }
                return Ok(());
            }
            Err(_) if path.is_file() => {}
            Err(error) => return Err(error.into()),
        }
    }
    bail!("image campaign exhausted 999 immutable execution identities")
}

fn ensure_immutable_json<T: Serialize + serde::de::DeserializeOwned + PartialEq>(
    path: &Path,
    value: &T,
) -> Result<()> {
    match bijux_dna_infra::atomic_create_json(path, value) {
        Ok(()) => Ok(()),
        Err(error) if path.is_file() => {
            let existing: T = serde_json::from_slice(&fs::read(path)?)?;
            if &existing == value {
                Ok(())
            } else {
                bail!(
                    "immutable image campaign publication conflicts at {}: {error}",
                    path.display()
                )
            }
        }
        Err(error) => Err(error.into()),
    }
}

fn portable_study_path(path: &Path, study_root: &Path) -> Result<String> {
    let relative = path.strip_prefix(study_root).context("campaign path is outside study root")?;
    Ok(format!("study://{}", relative.to_string_lossy().replace('\\', "/")))
}

fn resolve_study_path(study_root: &Path, value: &str) -> Result<PathBuf> {
    let relative = value.strip_prefix("study://").context("campaign path is not study-relative")?;
    let relative = Path::new(relative);
    if relative.components().any(|part| !matches!(part, Component::Normal(_))) {
        bail!("campaign path contains a non-normal component");
    }
    Ok(study_root.join(relative))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_campaign_preserves_catalog_order_and_uses_scoped_root() -> Result<()> {
        let retained = vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()];
        let selected = BTreeSet::from(["beta".to_string(), "gamma".to_string()]);

        let tools = select_campaign_tools(&retained, Some(&selected))?;
        assert_eq!(tools, ["beta", "gamma"]);
        let scope = campaign_scope(&tools, retained.len()).context("selected scope")?;
        assert!(scope.starts_with("selected-"));
        assert_eq!(
            campaign_root(Path::new("/study/campaigns"), "commit", Some(&scope)),
            Path::new("/study/campaigns").join("commit").join(scope)
        );
        assert!(require_campaign_path(
            Path::new("/study/campaigns/commit/selected-a/image.sif"),
            Path::new("/study/campaigns/commit/selected-a"),
            "image",
            "beta",
        )
        .is_ok());
        assert!(require_campaign_path(
            Path::new("/study/campaigns/commit/image.sif"),
            Path::new("/study/campaigns/commit/selected-a"),
            "image",
            "beta",
        )
        .is_err());
        Ok(())
    }

    #[test]
    fn selected_campaign_rejects_empty_and_unknown_tool_sets() {
        let retained = vec!["alpha".to_string(), "beta".to_string()];
        assert!(select_campaign_tools(&retained, Some(&BTreeSet::new())).is_err());
        assert!(select_campaign_tools(&retained, Some(&BTreeSet::from(["unknown".to_string()])),)
            .is_err());
    }

    #[test]
    fn complete_campaign_preserves_historical_commit_root() -> Result<()> {
        let retained = vec!["alpha".to_string(), "beta".to_string()];
        let tools = select_campaign_tools(&retained, None)?;
        assert_eq!(campaign_scope(&tools, retained.len()), None);
        assert_eq!(
            campaign_root(Path::new("/study/campaigns"), "commit", None),
            Path::new("/study/campaigns/commit")
        );
        Ok(())
    }

    #[test]
    fn campaign_attempt_requires_stable_scheduler_identity() {
        let campaign = ApptainerImageCampaignRequest {
            repository_root: PathBuf::from("repository"),
            study_root: PathBuf::from("study"),
            apptainer_executable: PathBuf::from("apptainer"),
            worker_count: 1,
            selected_tools: None,
        };
        let valid = ImageCampaignAttemptRequest {
            campaign: campaign.clone(),
            attempt_root: PathBuf::from("attempt"),
            attempt_id: "attempt-001".to_string(),
            slurm_job_id: "3473000".to_string(),
            completed_at: "2026-08-09T07:10:00Z".to_string(),
        };
        assert!(validate_attempt_identity(&valid).is_ok());

        let mut invalid = valid.clone();
        invalid.attempt_id = "../attempt".to_string();
        assert!(validate_attempt_identity(&invalid).is_err());
        invalid = valid.clone();
        invalid.slurm_job_id = "job-3473000".to_string();
        assert!(validate_attempt_identity(&invalid).is_err());
        invalid = valid;
        invalid.completed_at = "not-a-timestamp".to_string();
        assert!(validate_attempt_identity(&invalid).is_err());
    }
}
