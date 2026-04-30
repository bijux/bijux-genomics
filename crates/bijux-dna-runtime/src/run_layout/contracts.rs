use std::fmt::Write as _;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use sha2::Digest;

use bijux_dna_core::contract::canonical::to_canonical_json_bytes;
use bijux_dna_core::contract::{
    ContractVersion, ManifestMigrationAuditV1, ManifestMigrationStatusV1, RetryPolicy,
};
use bijux_dna_core::prelude::input_assessment::FastqLayout;
use bijux_dna_core::prelude::{CacheKey, Result as CoreResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunEnvironment {
    pub hostname: String,
    pub os: String,
    pub arch: String,
    pub runner: String,
    pub platform: String,
    pub tool_images: Vec<ToolImageDigest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolImageDigest {
    pub tool: String,
    pub image: String,
    pub digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStageEntry {
    pub stage_id: String,
    pub tool_id: String,
    pub execution_metrics_path: PathBuf,
    pub domain_metrics_path: PathBuf,
    pub logs_dir: PathBuf,
    pub outputs_dir: PathBuf,
    pub tool_invocation_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunManifest {
    pub schema_version: String,
    pub contract_version: ContractVersion,
    pub run_id: String,
    pub started_at: String,
    pub finished_at: String,
    pub pipeline: String,
    pub graph_hash: String,
    #[serde(default)]
    pub cache_key: Option<CacheKey>,
    pub layout: FastqLayout,
    pub stages: Vec<RunStageEntry>,
    #[serde(default)]
    pub tool_invocations: Vec<bijux_dna_core::metrics::ToolInvocationV1>,
    #[serde(default)]
    pub artifacts: Vec<RunArtifactEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunArtifactEntry {
    pub name: String,
    pub path: PathBuf,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunIndexEntry {
    pub run_id: String,
    pub domain: String,
    pub pipeline: String,
    pub stages: Vec<String>,
    pub layout: FastqLayout,
    pub tools: Vec<String>,
    pub objective: Option<String>,
    pub platform: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunIndexLine {
    pub schema_version: u32,
    pub run: RunIndexEntry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RunExecutionModeV1 {
    DryRun,
    Simulation,
    Advisory,
    #[default]
    Enforced,
}

impl std::fmt::Display for RunExecutionModeV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::DryRun => "dry_run",
            Self::Simulation => "simulation",
            Self::Advisory => "advisory",
            Self::Enforced => "enforced",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunLifecycleStateV1 {
    Planned,
    Prepared,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

impl std::fmt::Display for RunLifecycleStateV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Planned => "planned",
            Self::Prepared => "prepared",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStateTransitionV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_state: Option<RunLifecycleStateV1>,
    pub to_state: RunLifecycleStateV1,
    pub occurred_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunStateV1 {
    pub schema_version: String,
    pub run_id: String,
    pub mode: RunExecutionModeV1,
    pub state: RunLifecycleStateV1,
    #[serde(default)]
    pub transitions: Vec<RunStateTransitionV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manifest_path: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkpoint_path: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ExecutorDescriptorV1 {
    Local {
        runtime: String,
        execution_model: String,
        working_directory_policy: String,
        image_policy: String,
    },
    Container {
        runtime: String,
        image_resolution_policy: String,
        bind_mount_policy: String,
    },
    Hpc {
        scheduler: String,
        submission_mode: String,
        scratch_layout_policy: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        container_runtime: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunExecutorDescriptorV1 {
    pub schema_version: String,
    pub run_id: String,
    pub mode: RunExecutionModeV1,
    pub descriptor: ExecutorDescriptorV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancellationPolicyV1 {
    pub supports_external_cancellation: bool,
    pub checkpoint_before_cancel: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointPolicyV1 {
    pub strategy: String,
    pub granularity: String,
    pub resume_from_latest_completed_stage: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimePolicyV1 {
    pub schema_version: String,
    pub run_id: String,
    pub mode: RunExecutionModeV1,
    pub deterministic_scheduler: bool,
    pub retry_policy: RetryPolicy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_timeout_s: Option<u64>,
    pub cancellation: CancellationPolicyV1,
    pub checkpoint: CheckpointPolicyV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RunBackendDescriptorV1 {
    Local {
        temp_root_policy: String,
        temp_cleanup_policy: String,
        artifact_write_policy: String,
        log_capture_policy: String,
        interruption_recovery_policy: String,
    },
    Container {
        runtime: String,
        image_identity: String,
        bind_mount_policy: String,
        user_identity_policy: String,
        working_directory_policy: String,
        network_policy: String,
        resource_limit_policy: String,
        stdout_stderr_policy: String,
    },
    Slurm {
        scheduler: String,
        submission_script_path: PathBuf,
        poll_command: Vec<String>,
        cancel_command: Vec<String>,
        retry_policy: String,
        log_collection_policy: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        container_runtime: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunBackendRecordV1 {
    pub schema_version: String,
    pub run_id: String,
    pub mode: RunExecutionModeV1,
    pub descriptor: RunBackendDescriptorV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunMountBindingV1 {
    pub source: PathBuf,
    pub target: PathBuf,
    pub access: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSmokeWorkflowPlanV1 {
    pub schema_version: String,
    pub run_id: String,
    pub runner: String,
    pub image_identity: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub command: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mounts: Vec<RunMountBindingV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub expected_artifacts: Vec<String>,
    pub log_capture_policy: String,
}

#[must_use]
pub fn docker_smoke_workflow_plan(
    run_id: &str,
    image_identity: &str,
    mounts_root: PathBuf,
) -> RunSmokeWorkflowPlanV1 {
    RunSmokeWorkflowPlanV1 {
        schema_version: "bijux.run_smoke_workflow.v1".to_string(),
        run_id: run_id.to_string(),
        runner: "docker".to_string(),
        image_identity: image_identity.to_string(),
        command: vec!["sh".to_string(), "-c".to_string(), "echo bijux-smoke".to_string()],
        mounts: vec![
            RunMountBindingV1 {
                source: mounts_root.join("inputs"),
                target: PathBuf::from("/bijux/inputs"),
                access: "read_only".to_string(),
            },
            RunMountBindingV1 {
                source: mounts_root.join("artifacts"),
                target: PathBuf::from("/bijux/artifacts"),
                access: "read_write".to_string(),
            },
        ],
        expected_artifacts: vec!["smoke.stdout".to_string(), "smoke.exit_code".to_string()],
        log_capture_policy: "capture_stdout_stderr_and_container_id".to_string(),
    }
}

#[must_use]
pub fn apptainer_smoke_workflow_plan(
    run_id: &str,
    sif_identity: &str,
    bind_root: PathBuf,
) -> RunSmokeWorkflowPlanV1 {
    RunSmokeWorkflowPlanV1 {
        schema_version: "bijux.run_smoke_workflow.v1".to_string(),
        run_id: run_id.to_string(),
        runner: "apptainer".to_string(),
        image_identity: sif_identity.to_string(),
        command: vec!["sh".to_string(), "-c".to_string(), "echo bijux-smoke".to_string()],
        mounts: vec![
            RunMountBindingV1 {
                source: bind_root.join("inputs"),
                target: PathBuf::from("/bijux/inputs"),
                access: "read_only".to_string(),
            },
            RunMountBindingV1 {
                source: bind_root.join("artifacts"),
                target: PathBuf::from("/bijux/artifacts"),
                access: "read_write".to_string(),
            },
        ],
        expected_artifacts: vec!["smoke.stdout".to_string(), "smoke.exit_code".to_string()],
        log_capture_policy: "capture_stdout_stderr_and_runtime_logs".to_string(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResourceRequestV1 {
    pub cpu_threads: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_mb: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scratch_mb: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub walltime_s: Option<u64>,
    pub io_intensity: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container_runtime: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageExecutionRequirementV1 {
    pub stage_id: String,
    pub requires_local_runtime: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_container_runtime: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_scheduler: Option<String>,
    #[serde(default)]
    pub required_evidence_topics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorCapabilitiesV1 {
    pub runner: String,
    pub supports_local_runtime: bool,
    #[serde(default)]
    pub container_runtimes: Vec<String>,
    #[serde(default)]
    pub schedulers: Vec<String>,
    #[serde(default)]
    pub evidence_topics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorCapabilityDecisionV1 {
    pub admitted: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refusal_codes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

#[must_use]
pub fn negotiate_executor_capabilities(
    requirement: &StageExecutionRequirementV1,
    capabilities: &ExecutorCapabilitiesV1,
) -> ExecutorCapabilityDecisionV1 {
    let mut refusal_codes = Vec::new();
    let mut warnings = Vec::new();

    if requirement.requires_local_runtime && !capabilities.supports_local_runtime {
        refusal_codes.push("missing_local_runtime".to_string());
    }
    if let Some(required_runtime) = &requirement.required_container_runtime {
        if !capabilities.container_runtimes.iter().any(|runtime| runtime == required_runtime) {
            refusal_codes.push("missing_container_runtime".to_string());
        }
    }
    if let Some(required_scheduler) = &requirement.required_scheduler {
        if !capabilities.schedulers.iter().any(|scheduler| scheduler == required_scheduler) {
            refusal_codes.push("missing_scheduler".to_string());
        }
    }

    for topic in &requirement.required_evidence_topics {
        if !capabilities.evidence_topics.iter().any(|item| item == topic) {
            warnings.push(format!("missing_evidence_topic:{topic}"));
        }
    }

    ExecutorCapabilityDecisionV1 { admitted: refusal_codes.is_empty(), refusal_codes, warnings }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackSafetyRequestV1 {
    pub primary_runner: String,
    pub fallback_runner: String,
    pub output_contract_hash: String,
    pub fallback_output_contract_hash: String,
    #[serde(default)]
    pub evidence_obligations: Vec<String>,
    #[serde(default)]
    pub fallback_evidence_topics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackSafetyDecisionV1 {
    pub safe: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refusal_codes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

#[must_use]
pub fn evaluate_fallback_safety(request: &FallbackSafetyRequestV1) -> FallbackSafetyDecisionV1 {
    let mut refusal_codes = Vec::new();
    let mut notes = Vec::new();

    if request.output_contract_hash != request.fallback_output_contract_hash {
        refusal_codes.push("fallback_output_contract_mismatch".to_string());
    }

    let missing_topics = request
        .evidence_obligations
        .iter()
        .filter(|required| !request.fallback_evidence_topics.iter().any(|topic| topic == *required))
        .map(|missing| missing.to_string())
        .collect::<Vec<_>>();
    if !missing_topics.is_empty() {
        refusal_codes.push("fallback_evidence_obligation_gap".to_string());
        notes.push(format!(
            "fallback runner {} misses evidence topics: {}",
            request.fallback_runner,
            missing_topics.join(",")
        ));
    }
    if refusal_codes.is_empty() {
        notes.push(format!(
            "fallback from {} to {} preserves output and evidence obligations",
            request.primary_runner, request.fallback_runner
        ));
    }

    FallbackSafetyDecisionV1 { safe: refusal_codes.is_empty(), refusal_codes, notes }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSchedulingDecisionV1 {
    pub schema_version: String,
    pub run_id: String,
    pub runner: String,
    pub queue_class: String,
    pub placement_reason: String,
    pub requested_resources: RunResourceRequestV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeResourceLimitsV1 {
    pub max_cpu_threads: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_memory_mb: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_scratch_mb: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_walltime_s: Option<u64>,
    #[serde(default)]
    pub allowed_io_intensity: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeResourceAdmissionV1 {
    pub admitted: bool,
    pub queue_class: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub refusal_codes: Vec<String>,
}

#[must_use]
pub fn admit_runtime_resources(
    request: &RunResourceRequestV1,
    limits: &RuntimeResourceLimitsV1,
) -> RuntimeResourceAdmissionV1 {
    let mut refusal_codes = Vec::new();
    let mut warnings = Vec::new();

    if request.cpu_threads > limits.max_cpu_threads {
        refusal_codes.push("cpu_threads_exceed_limit".to_string());
    }
    if let (Some(requested), Some(max)) = (request.memory_mb, limits.max_memory_mb) {
        if requested > max {
            refusal_codes.push("memory_mb_exceed_limit".to_string());
        }
    }
    if let (Some(requested), Some(max)) = (request.scratch_mb, limits.max_scratch_mb) {
        if requested > max {
            refusal_codes.push("scratch_mb_exceed_limit".to_string());
        }
    }
    if let (Some(requested), Some(max)) = (request.walltime_s, limits.max_walltime_s) {
        if requested > max {
            refusal_codes.push("walltime_s_exceed_limit".to_string());
        }
    }

    if !limits.allowed_io_intensity.is_empty()
        && !limits.allowed_io_intensity.iter().any(|io_class| io_class == &request.io_intensity)
    {
        refusal_codes.push("io_intensity_not_allowed".to_string());
    }

    if request.container_runtime.is_none() {
        warnings.push("container_runtime_unspecified".to_string());
    }
    if request.walltime_s.is_none() {
        warnings.push("walltime_unspecified".to_string());
    }

    let queue_class = if request.cpu_threads >= limits.max_cpu_threads.saturating_div(2).max(1) {
        "high_resource".to_string()
    } else {
        "standard".to_string()
    };
    RuntimeResourceAdmissionV1 {
        admitted: refusal_codes.is_empty(),
        queue_class,
        warnings,
        refusal_codes,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunQueueLifecycleStateV1 {
    Queued,
    Paused,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

impl std::fmt::Display for RunQueueLifecycleStateV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Queued => "queued",
            Self::Paused => "paused",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunQueueTransitionV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_state: Option<RunQueueLifecycleStateV1>,
    pub to_state: RunQueueLifecycleStateV1,
    pub occurred_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunQueueStateV1 {
    pub schema_version: String,
    pub run_id: String,
    pub dedup_key: String,
    pub state: RunQueueLifecycleStateV1,
    #[serde(default)]
    pub transitions: Vec<RunQueueTransitionV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_step_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunLeaseV1 {
    pub schema_version: String,
    pub run_id: String,
    pub lease_id: String,
    pub holder: String,
    pub lock_path: PathBuf,
    pub acquired_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub released_at: Option<String>,
    pub exclusive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunControlActionV1 {
    Pause,
    Resume,
    Cancel,
}

impl std::fmt::Display for RunControlActionV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Pause => "pause",
            Self::Resume => "resume",
            Self::Cancel => "cancel",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunControlAuditEntryV1 {
    pub requested_action: RunControlActionV1,
    pub observed_state: RunQueueLifecycleStateV1,
    pub occurred_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunControlStateV1 {
    pub schema_version: String,
    pub run_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_action: Option<RunControlActionV1>,
    pub observed_state: RunQueueLifecycleStateV1,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub audit_log: Vec<RunControlAuditEntryV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorHealthCheckV1 {
    pub check_id: String,
    pub ok: bool,
    pub detail: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorHealthReportV1 {
    pub schema_version: String,
    pub run_id: String,
    pub overall_ok: bool,
    #[serde(default)]
    pub checks: Vec<OperatorHealthCheckV1>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlurmJobStateV1 {
    Submitted,
    Pending,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlurmJobTransitionV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_state: Option<SlurmJobStateV1>,
    pub to_state: SlurmJobStateV1,
    pub occurred_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlurmSubmissionRecordV1 {
    pub schema_version: String,
    pub run_id: String,
    pub scheduler: String,
    pub submission_script_path: PathBuf,
    pub job_id: String,
    pub state: SlurmJobStateV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub poll_command: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cancel_command: Vec<String>,
    pub stdout_log_path: PathBuf,
    pub stderr_log_path: PathBuf,
    pub retry_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transitions: Vec<SlurmJobTransitionV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HpcExecutionProfileV1 {
    pub schema_version: String,
    pub profile_id: String,
    pub scheduler: String,
    pub submission_mode: String,
    pub scratch_layout_policy: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container_runtime: Option<String>,
    pub site_config_path: PathBuf,
}

#[must_use]
pub fn lunarc_execution_profile(site_config_path: PathBuf) -> HpcExecutionProfileV1 {
    HpcExecutionProfileV1 {
        schema_version: "bijux.hpc_execution_profile.v1".to_string(),
        profile_id: "lunarc".to_string(),
        scheduler: "slurm".to_string(),
        submission_mode: "batch".to_string(),
        scratch_layout_policy: "run_scoped_scratch".to_string(),
        container_runtime: Some("apptainer".to_string()),
        site_config_path,
    }
}

#[must_use]
pub fn executor_descriptor_from_hpc_profile(
    run_id: &str,
    mode: RunExecutionModeV1,
    profile: &HpcExecutionProfileV1,
) -> RunExecutorDescriptorV1 {
    RunExecutorDescriptorV1 {
        schema_version: "bijux.executor_descriptor.v1".to_string(),
        run_id: run_id.to_string(),
        mode,
        descriptor: ExecutorDescriptorV1::Hpc {
            scheduler: profile.scheduler.clone(),
            submission_mode: profile.submission_mode.clone(),
            scratch_layout_policy: profile.scratch_layout_policy.clone(),
            container_runtime: profile.container_runtime.clone(),
        },
    }
}

/// # Errors
/// Returns an error when the requested transition is invalid for the current state.
pub fn transition_slurm_submission(
    submission: &mut SlurmSubmissionRecordV1,
    to_state: SlurmJobStateV1,
    occurred_at: &str,
    detail: Option<String>,
) -> CoreResult<()> {
    if submission.state == to_state {
        submission.transitions.push(SlurmJobTransitionV1 {
            from_state: Some(submission.state),
            to_state,
            occurred_at: occurred_at.to_string(),
            detail: detail.or_else(|| Some("idempotent transition request".to_string())),
        });
        return Ok(());
    }

    let allowed = match submission.state {
        SlurmJobStateV1::Submitted => {
            matches!(
                to_state,
                SlurmJobStateV1::Pending
                    | SlurmJobStateV1::Running
                    | SlurmJobStateV1::Cancelled
                    | SlurmJobStateV1::Failed
            )
        }
        SlurmJobStateV1::Pending => {
            matches!(
                to_state,
                SlurmJobStateV1::Running | SlurmJobStateV1::Cancelled | SlurmJobStateV1::Failed
            )
        }
        SlurmJobStateV1::Running => {
            matches!(
                to_state,
                SlurmJobStateV1::Succeeded | SlurmJobStateV1::Failed | SlurmJobStateV1::Cancelled
            )
        }
        SlurmJobStateV1::Succeeded | SlurmJobStateV1::Failed | SlurmJobStateV1::Cancelled => false,
    };
    if !allowed {
        return Err(bijux_dna_core::prelude::BijuxError::validation(format!(
            "invalid slurm transition {:?} -> {:?}",
            submission.state, to_state
        )));
    }

    let from_state = submission.state;
    submission.state = to_state;
    submission.transitions.push(SlurmJobTransitionV1 {
        from_state: Some(from_state),
        to_state,
        occurred_at: occurred_at.to_string(),
        detail,
    });
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunCheckpointV1 {
    pub schema_version: String,
    pub run_id: String,
    pub mode: RunExecutionModeV1,
    pub updated_at: String,
    #[serde(default)]
    pub completed_stage_ids: Vec<String>,
    #[serde(default)]
    pub pending_stage_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_stage_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunFailureV1 {
    pub schema_version: String,
    pub run_id: String,
    pub mode: RunExecutionModeV1,
    pub state: RunLifecycleStateV1,
    pub failure_code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attempt: Option<u32>,
    pub observed_at: String,
    pub retryable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactScientificContextV1 {
    pub domain: String,
    pub meaning: String,
    pub safe_to_use: bool,
    #[serde(default)]
    pub advisory_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactIdentityV1 {
    pub artifact_id: String,
    pub name: String,
    pub role: String,
    pub path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub producing_stage_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub producing_command: Vec<String>,
    #[serde(default)]
    pub input_lineage: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replay_source_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scientific_context: Option<ArtifactScientificContextV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactInventoryV1 {
    pub schema_version: String,
    pub run_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replay_source_run_id: Option<String>,
    #[serde(default)]
    pub artifacts: Vec<ArtifactIdentityV1>,
}

#[derive(Debug, Clone, Deserialize)]
struct ArtifactIdentityLegacyV0 {
    pub artifact_id: String,
    pub name: String,
    pub role: String,
    pub path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub producing_stage_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub producing_command: Vec<String>,
    #[serde(default)]
    pub input_lineage: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replay_source_run_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ArtifactInventoryLegacyV0 {
    pub schema_version: String,
    pub run_id: String,
    #[serde(default)]
    pub artifacts: Vec<ArtifactIdentityLegacyV0>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheDecisionV1 {
    pub stage_id: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_key: Option<CacheKey>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayManifestV1 {
    pub schema_version: String,
    pub replay_run_id: String,
    pub original_run_id: String,
    #[serde(default)]
    pub selected_stage_ids: Vec<String>,
    #[serde(default)]
    pub reused_artifact_ids: Vec<String>,
    #[serde(default)]
    pub rerun_stage_ids: Vec<String>,
    #[serde(default)]
    pub expected_outputs: Vec<String>,
    #[serde(default)]
    pub cache_decisions: Vec<CacheDecisionV1>,
    #[serde(default)]
    pub environment_differences: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashLedgerEntryV1 {
    pub record_id: String,
    pub kind: String,
    pub path: PathBuf,
    pub sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_entry_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashLedgerV1 {
    pub schema_version: String,
    pub run_id: String,
    pub root_sha256: String,
    #[serde(default)]
    pub entries: Vec<HashLedgerEntryV1>,
}

#[derive(Debug, Clone)]
pub struct RunLayout {
    pub run_dir: PathBuf,
    pub stages_dir: PathBuf,
    pub manifests_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub reports_dir: PathBuf,
    pub summary_dir: PathBuf,
    pub run_artifacts_dir: PathBuf,
    pub checkpoints_dir: PathBuf,
    pub assessment_path: PathBuf,
    pub graph_path: PathBuf,
    pub plan_manifest_path: PathBuf,
    pub manifest_path: PathBuf,
    pub environment_path: PathBuf,
    pub metadata_path: PathBuf,
    pub events_path: PathBuf,
    pub run_state_path: PathBuf,
    pub runtime_policy_path: PathBuf,
    pub executor_descriptor_path: PathBuf,
    pub backend_descriptor_path: PathBuf,
    pub scheduling_decision_path: PathBuf,
    pub queue_state_path: PathBuf,
    pub lease_path: PathBuf,
    pub control_state_path: PathBuf,
    pub health_report_path: PathBuf,
    pub slurm_submission_path: PathBuf,
    pub checkpoint_path: PathBuf,
    pub failure_path: PathBuf,
    pub run_summary_path: PathBuf,
    pub run_summary_text_path: PathBuf,
    pub artifact_inventory_path: PathBuf,
    pub artifact_inventory_text_path: PathBuf,
    pub replay_manifest_path: PathBuf,
    pub hash_ledger_path: PathBuf,
    pub evidence_verification_path: PathBuf,
    pub evidence_bundle_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunLayoutV1 {
    pub schema_version: String,
    pub run_dir: String,
    pub stages_dir: String,
    pub manifests_dir: String,
    pub logs_dir: String,
    pub reports_dir: String,
    pub summary_dir: String,
    pub run_artifacts_dir: String,
    pub checkpoints_dir: String,
    pub assessment_path: String,
    pub graph_path: String,
    pub plan_manifest_path: String,
    pub manifest_path: String,
    pub environment_path: String,
    pub metadata_path: String,
    pub events_path: String,
    pub run_state_path: String,
    pub runtime_policy_path: String,
    pub executor_descriptor_path: String,
    pub backend_descriptor_path: String,
    pub scheduling_decision_path: String,
    pub queue_state_path: String,
    pub lease_path: String,
    pub control_state_path: String,
    pub health_report_path: String,
    pub slurm_submission_path: String,
    pub checkpoint_path: String,
    pub failure_path: String,
    pub run_summary_path: String,
    pub run_summary_text_path: String,
    pub artifact_inventory_path: String,
    pub artifact_inventory_text_path: String,
    pub replay_manifest_path: String,
    pub hash_ledger_path: String,
    pub evidence_verification_path: String,
    pub evidence_bundle_path: String,
}

impl RunLayout {
    #[must_use]
    pub fn from_run_dir(run_dir: PathBuf) -> Self {
        let stages_dir = run_dir.join("stages");
        let manifests_dir = run_dir.join("manifests");
        let logs_dir = run_dir.join("logs");
        let reports_dir = run_dir.join("reports");
        let summary_dir = run_dir.join("summary");
        let run_artifacts_dir = run_dir.join("run_artifacts");
        let checkpoints_dir = run_dir.join("checkpoints");
        Self {
            graph_path: manifests_dir.join("graph.json"),
            plan_manifest_path: manifests_dir.join("plan_manifest.json"),
            assessment_path: run_dir.join("input_assessment.json"),
            manifest_path: run_dir.join("run_manifest.json"),
            environment_path: run_dir.join("environment.json"),
            metadata_path: run_dir.join("run_metadata.json"),
            events_path: run_dir.join("events.jsonl"),
            run_state_path: run_dir.join("run_state.json"),
            runtime_policy_path: run_dir.join("runtime_policy.json"),
            executor_descriptor_path: run_dir.join("executor_descriptor.json"),
            backend_descriptor_path: run_dir.join("backend_descriptor.json"),
            scheduling_decision_path: run_dir.join("scheduling_decision.json"),
            queue_state_path: run_dir.join("queue_state.json"),
            lease_path: run_dir.join("run_lease.json"),
            control_state_path: run_dir.join("run_control.json"),
            health_report_path: run_dir.join("operator_health.json"),
            slurm_submission_path: run_dir.join("slurm_submission.json"),
            checkpoint_path: checkpoints_dir.join("checkpoint.json"),
            failure_path: run_dir.join("run_failure.json"),
            run_summary_path: summary_dir.join("run_summary.json"),
            run_summary_text_path: summary_dir.join("run_summary.txt"),
            artifact_inventory_path: run_dir.join("artifact_inventory.json"),
            artifact_inventory_text_path: run_dir.join("artifact_inventory.txt"),
            replay_manifest_path: run_dir.join("replay_manifest.json"),
            hash_ledger_path: run_dir.join("hash_ledger.json"),
            evidence_verification_path: run_dir.join("evidence_verification.json"),
            evidence_bundle_path: run_dir.join("evidence_bundle.json"),
            stages_dir,
            manifests_dir,
            logs_dir,
            reports_dir,
            summary_dir,
            run_artifacts_dir,
            checkpoints_dir,
            run_dir,
        }
    }

    #[must_use]
    pub fn contract(&self) -> RunLayoutV1 {
        RunLayoutV1 {
            schema_version: "bijux.run_layout.v1".to_string(),
            run_dir: self.run_dir.display().to_string(),
            stages_dir: self.stages_dir.display().to_string(),
            manifests_dir: self.manifests_dir.display().to_string(),
            logs_dir: self.logs_dir.display().to_string(),
            reports_dir: self.reports_dir.display().to_string(),
            summary_dir: self.summary_dir.display().to_string(),
            run_artifacts_dir: self.run_artifacts_dir.display().to_string(),
            checkpoints_dir: self.checkpoints_dir.display().to_string(),
            assessment_path: self.assessment_path.display().to_string(),
            graph_path: self.graph_path.display().to_string(),
            plan_manifest_path: self.plan_manifest_path.display().to_string(),
            manifest_path: self.manifest_path.display().to_string(),
            environment_path: self.environment_path.display().to_string(),
            metadata_path: self.metadata_path.display().to_string(),
            events_path: self.events_path.display().to_string(),
            run_state_path: self.run_state_path.display().to_string(),
            runtime_policy_path: self.runtime_policy_path.display().to_string(),
            executor_descriptor_path: self.executor_descriptor_path.display().to_string(),
            backend_descriptor_path: self.backend_descriptor_path.display().to_string(),
            scheduling_decision_path: self.scheduling_decision_path.display().to_string(),
            queue_state_path: self.queue_state_path.display().to_string(),
            lease_path: self.lease_path.display().to_string(),
            control_state_path: self.control_state_path.display().to_string(),
            health_report_path: self.health_report_path.display().to_string(),
            slurm_submission_path: self.slurm_submission_path.display().to_string(),
            checkpoint_path: self.checkpoint_path.display().to_string(),
            failure_path: self.failure_path.display().to_string(),
            run_summary_path: self.run_summary_path.display().to_string(),
            run_summary_text_path: self.run_summary_text_path.display().to_string(),
            artifact_inventory_path: self.artifact_inventory_path.display().to_string(),
            artifact_inventory_text_path: self.artifact_inventory_text_path.display().to_string(),
            replay_manifest_path: self.replay_manifest_path.display().to_string(),
            hash_ledger_path: self.hash_ledger_path.display().to_string(),
            evidence_verification_path: self.evidence_verification_path.display().to_string(),
            evidence_bundle_path: self.evidence_bundle_path.display().to_string(),
        }
    }
}

impl RunManifest {
    /// # Errors
    /// Returns an error if validation fails.
    pub fn validate(&self) -> CoreResult<()> {
        if self.graph_hash.trim().is_empty() {
            return Err(bijux_dna_core::prelude::BijuxError::validation(
                "run manifest graph_hash is empty",
            ));
        }
        if self.artifacts.is_empty() {
            return Err(bijux_dna_core::prelude::BijuxError::validation(
                "run manifest artifacts list is empty",
            ));
        }
        for artifact in &self.artifacts {
            if artifact.name.trim().is_empty() {
                return Err(bijux_dna_core::prelude::BijuxError::validation(
                    "run manifest artifact name is empty",
                ));
            }
            if artifact.sha256.trim().is_empty() {
                return Err(bijux_dna_core::prelude::BijuxError::validation(
                    "run manifest artifact hash is empty",
                ));
            }
        }
        Ok(())
    }

    /// # Errors
    /// Returns an error if canonical serialization fails.
    pub fn hash(&self) -> CoreResult<String> {
        let bytes = to_canonical_json_bytes(self)?;
        let mut hasher = sha2::Sha256::new();
        hasher.update(bytes);
        Ok(sha256_hex(hasher.finalize()))
    }
}

/// # Errors
/// Returns an error when the inventory schema is unsupported or the payload is malformed.
pub fn migrate_artifact_inventory_value(
    value: &serde_json::Value,
) -> CoreResult<(ArtifactInventoryV1, ManifestMigrationAuditV1)> {
    let schema_version = value
        .get("schema_version")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            bijux_dna_core::prelude::BijuxError::validation(
                "artifact_inventory payload missing schema_version",
            )
        })?
        .to_string();
    match schema_version.as_str() {
        "bijux.artifact_inventory.v1" => {
            let inventory: ArtifactInventoryV1 = serde_json::from_value(value.clone())?;
            Ok((
                inventory.clone(),
                migration_audit(
                    "artifact_inventory",
                    &schema_version,
                    Some(&inventory.schema_version),
                    ManifestMigrationStatusV1::Passthrough,
                    "artifact inventory already matches the governed v1 schema",
                    value,
                    Some(&inventory),
                )?,
            ))
        }
        "bijux.artifact_inventory.v0" => {
            let legacy: ArtifactInventoryLegacyV0 = serde_json::from_value(value.clone())?;
            let inventory = ArtifactInventoryV1 {
                schema_version: "bijux.artifact_inventory.v1".to_string(),
                run_id: legacy.run_id,
                replay_source_run_id: None,
                artifacts: legacy
                    .artifacts
                    .into_iter()
                    .map(|artifact| ArtifactIdentityV1 {
                        artifact_id: artifact.artifact_id,
                        name: artifact.name,
                        role: artifact.role,
                        path: artifact.path,
                        sha256: artifact.sha256,
                        producing_stage_id: artifact.producing_stage_id,
                        producing_command: artifact.producing_command,
                        input_lineage: artifact.input_lineage,
                        schema_version: None,
                        replay_source_run_id: artifact.replay_source_run_id,
                        scientific_context: None,
                    })
                    .collect(),
            };
            Ok((
                inventory.clone(),
                migration_audit(
                    "artifact_inventory",
                    &legacy.schema_version,
                    Some(&inventory.schema_version),
                    ManifestMigrationStatusV1::Upgraded,
                    "artifact inventory upgraded from governed legacy v0 by materializing explicit replay and scientific context fields",
                    value,
                    Some(&inventory),
                )?,
            ))
        }
        _ => Err(bijux_dna_core::prelude::BijuxError::validation(format!(
            "artifact_inventory schema_version {schema_version} is unsupported; supported versions: bijux.artifact_inventory.v0, bijux.artifact_inventory.v1"
        ))),
    }
}

/// # Errors
/// Returns an error when the file cannot be read, parsed, or migrated.
pub fn read_supported_artifact_inventory(
    path: &std::path::Path,
) -> CoreResult<(ArtifactInventoryV1, ManifestMigrationAuditV1)> {
    let raw = std::fs::read_to_string(path).map_err(|err| {
        bijux_dna_core::prelude::BijuxError::Io(format!("read {}: {err}", path.display()))
    })?;
    let value: serde_json::Value = serde_json::from_str(&raw)?;
    migrate_artifact_inventory_value(&value)
}

fn sha256_hex(digest: impl AsRef<[u8]>) -> String {
    let bytes = digest.as_ref();
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(&mut hex, "{byte:02x}");
    }
    hex
}

fn migration_audit<T: Serialize>(
    schema_family: &str,
    from_schema_version: &str,
    to_schema_version: Option<&str>,
    status: ManifestMigrationStatusV1,
    exact_reason: &str,
    original: &serde_json::Value,
    migrated: Option<&T>,
) -> CoreResult<ManifestMigrationAuditV1> {
    let migrated_payload_sha256 = migrated.map(payload_sha256).transpose()?;
    Ok(ManifestMigrationAuditV1 {
        schema_family: schema_family.to_string(),
        from_schema_version: from_schema_version.to_string(),
        to_schema_version: to_schema_version.map(str::to_string),
        status,
        exact_reason: exact_reason.to_string(),
        source_payload_sha256: payload_sha256(original)?,
        migrated_payload_sha256,
    })
}

fn payload_sha256<T: Serialize>(value: &T) -> CoreResult<String> {
    let bytes = to_canonical_json_bytes(value)?;
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    Ok(sha256_hex(hasher.finalize()))
}
