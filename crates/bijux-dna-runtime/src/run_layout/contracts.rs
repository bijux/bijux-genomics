use std::fmt::Write as _;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use sha2::Digest;

use bijux_dna_core::contract::canonical::to_canonical_json_bytes;
use bijux_dna_core::contract::{ContractVersion, RetryPolicy};
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
    pub checkpoint_path: PathBuf,
    pub failure_path: PathBuf,
    pub run_summary_path: PathBuf,
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
    pub checkpoint_path: String,
    pub failure_path: String,
    pub run_summary_path: String,
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
            checkpoint_path: checkpoints_dir.join("checkpoint.json"),
            failure_path: run_dir.join("run_failure.json"),
            run_summary_path: summary_dir.join("run_summary.json"),
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
            checkpoint_path: self.checkpoint_path.display().to_string(),
            failure_path: self.failure_path.display().to_string(),
            run_summary_path: self.run_summary_path.display().to_string(),
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

fn sha256_hex(digest: impl AsRef<[u8]>) -> String {
    let bytes = digest.as_ref();
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(&mut hex, "{byte:02x}");
    }
    hex
}
