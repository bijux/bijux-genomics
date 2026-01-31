use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;
use uuid::Uuid;
use walkdir::WalkDir;

pub mod domain;
pub mod events;
pub mod input_assessment;
pub mod measure;
pub mod metrics;
pub mod metrics_registry;
pub mod observability;
pub mod run_index;
pub mod selection;
pub mod stage_plan;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StageId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StageVersion(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ToolId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RunId(pub String);

pub use metrics::{
    AdapterBankProvenanceV1, BankEntryV1, BankRefV1, FastqCorrectMetricsV1, FastqDeltaMetricsV1,
    FastqFilterMetricsV1, FastqMergeMetricsV1, FastqPreprocessMetricsV1, FastqQcPostMetricsV1,
    FastqTrimMetricsV1, FastqUmiMetricsV1, FastqValidateMetricsV1, MetricContextV1,
    RetentionReportMetricV1, StageMetricsV1, ToolInvocationV1,
};
pub use metrics_registry::{
    metric_semantics, metrics_schema_for_stage, MetricDirection as MetricSemanticsDirection,
    MetricSemantics, MetricsSchemaId, FASTQ_METRICS_SCHEMAS,
};
pub use observability::{
    canonicalize_json_value, parameters_json_canonicalization, AssetsProvenanceV1,
    EffectiveConfigV1, FactsRowV1, FilterReportV1, MetricSemanticsV1, ReportCompletenessV1,
    ReportContractV1, ReportProvenanceV1, ReportSchemaV1, ReportStageSummaryV1, RetentionContextV1,
    RetentionDefinitionV1, RetentionReportV1, StageObservabilityContextV1,
    StageObservabilityContractV1, StageReportV1, TelemetryEventV1,
};
pub use selection::{
    objective_spec, BenchResultRecord, BenchResultStatus, Disqualification, Objective,
    ObjectiveSpec, ObjectiveWeights, StageSelection, ToolScore,
};
pub use stage_plan::{ArtifactRef, CommandSpecV1, ContainerImageRefV1, StageIO, StagePlanV1};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolExecutionSpecV1 {
    pub tool_id: ToolId,
    pub tool_version: String,
    pub image: ContainerImageRefV1,
    pub command: CommandSpecV1,
    pub resources: ToolConstraints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunMetadataV1 {
    pub run_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub hostname: String,
    pub os: String,
    pub arch: String,
    pub cpu_model: String,
    pub cores: usize,
    pub ram_mb: u64,
    pub platform: String,
    pub platform_version: String,
    pub bijux_version: String,
    pub git_commit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolInvocationMetadataV1 {
    pub stage: String,
    pub tool: String,
    pub version: String,
    pub image: String,
    pub command: String,
    pub threads: u32,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageMetadataV1 {
    pub stage: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub tool: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolExecutionMetadataV1 {
    pub stage: String,
    pub tool: String,
    pub version: String,
    pub image: String,
    pub command: String,
    pub threads: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawFailure {
    pub stage: String,
    pub tool: String,
    pub reason: String,
}

#[must_use]
pub fn build_run_metadata_v1(
    run_id: Uuid,
    started_at: DateTime<Utc>,
    finished_at: DateTime<Utc>,
    platform: &str,
    platform_version: &str,
    bijux_version: &str,
    git_commit: &str,
) -> RunMetadataV1 {
    let mut system = sysinfo::System::new_all();
    system.refresh_all();
    let hostname = sysinfo::System::host_name().unwrap_or_else(|| "unknown".to_string());
    let os = sysinfo::System::long_os_version()
        .or_else(sysinfo::System::os_version)
        .unwrap_or_else(|| "unknown".to_string());
    let cpu_model = system
        .cpus()
        .first()
        .map_or_else(|| "unknown".to_string(), |cpu| cpu.brand().to_string());
    let cores = system.cpus().len();
    let ram_mb = system.total_memory() / 1024;
    RunMetadataV1 {
        run_id,
        started_at,
        finished_at,
        hostname,
        os,
        arch: std::env::consts::ARCH.to_string(),
        cpu_model,
        cores,
        ram_mb,
        platform: platform.to_string(),
        platform_version: platform_version.to_string(),
        bijux_version: bijux_version.to_string(),
        git_commit: git_commit.to_string(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathSpec {
    pub input: Vec<PathBuf>,
    pub output: Vec<PathBuf>,
    pub work: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSpec {
    pub stage: StageId,
    pub tool: ToolId,
    pub paths: PathSpec,
    pub params: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerSpec {
    pub image: String,
    pub runtime: String,
    pub mounts: BTreeMap<String, String>,
    pub env: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionBackend {
    Local,
    Slurm,
    K8s,
}

#[derive(Debug, Error)]
pub enum BijuxError {
    #[error("profile error: {0}")]
    Profile(String),
    #[error("manifest error: {0}")]
    Manifest(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

#[must_use]
pub fn new_run_id() -> RunId {
    RunId(Uuid::new_v4().to_string())
}

#[must_use]
pub fn run_dir(base: &Path, run_id: &RunId, stage: &StageId, tool: &ToolId) -> PathBuf {
    base.join("runs")
        .join(&run_id.0)
        .join(&stage.0)
        .join(&tool.0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunReport {
    pub schema_version: String,
    pub run_id: RunId,
    pub stage: StageId,
    pub tool: ToolId,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub status: RunStatus,
    pub inputs: Vec<PathBuf>,
    pub outputs: Vec<PathBuf>,
    pub metrics: BTreeMap<String, serde_json::Value>,
    pub provenance: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RunStatus {
    Success,
    Failed,
    Skipped,
}

impl RunReport {
    #[must_use]
    pub fn new(run_id: RunId, stage: StageId, tool: ToolId, status: RunStatus) -> Self {
        let now = Utc::now();
        let mut provenance = BTreeMap::new();
        provenance.insert("tool_version".to_string(), "unknown".to_string());
        provenance.insert("image_ref".to_string(), "unknown".to_string());
        Self {
            schema_version: "bijux.report.v0".to_string(),
            run_id,
            stage,
            tool,
            started_at: now,
            ended_at: now,
            status,
            inputs: Vec::new(),
            outputs: Vec::new(),
            metrics: BTreeMap::new(),
            provenance,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub container_runtime: String,
    pub default_threads: u32,
    pub default_mem_gb: u32,
    pub default_time_minutes: u32,
    pub run_base_dir: PathBuf,
    #[serde(default = "default_pull_policy")]
    pub image_pull_policy: ImagePullPolicy,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImagePullPolicy {
    Always,
    IfMissing,
    Never,
}

fn default_pull_policy() -> ImagePullPolicy {
    ImagePullPolicy::IfMissing
}

/// Load a profile from the given YAML file.
///
/// # Errors
/// Returns an error if the file cannot be read or parsed as YAML.
pub fn load_profile(path: &Path) -> Result<Profile, BijuxError> {
    let contents = std::fs::read_to_string(path)?;
    let profile: Profile = serde_yaml::from_str(&contents)?;
    Ok(profile)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortSpec {
    pub name: String,
    pub data_type: String,
    pub cardinality: Cardinality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Cardinality {
    One,
    Many,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParameterSpec {
    pub name: String,
    pub param_type: String,
    pub default: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricSpec {
    pub name: String,
    pub meaning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ImageRequirements {
    #[serde(default)]
    pub needs: Vec<String>,
    #[serde(default)]
    pub forbids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageManifestV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub domain: String,
    pub inputs: Vec<PortSpec>,
    pub outputs: Vec<PortSpec>,
    pub parameters: Vec<ParameterSpec>,
    pub metrics: Vec<MetricSpec>,
    pub description: String,
    #[serde(default)]
    pub mutates_fastq: bool,
    #[serde(default)]
    pub report_only: bool,
    #[serde(default)]
    pub may_change_read_count: bool,
    #[serde(default)]
    pub image_requirements: ImageRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StageManifestDoc {
    #[serde(default)]
    extends: Option<String>,
    #[serde(default)]
    schema_version: Option<String>,
    #[serde(default)]
    stage_id: Option<String>,
    #[serde(default)]
    domain: Option<String>,
    #[serde(default)]
    inputs: Option<Vec<PortSpec>>,
    #[serde(default)]
    outputs: Option<Vec<PortSpec>>,
    #[serde(default)]
    parameters: Option<Vec<ParameterSpec>>,
    #[serde(default)]
    metrics: Option<Vec<MetricSpec>>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    mutates_fastq: Option<bool>,
    #[serde(default)]
    report_only: Option<bool>,
    #[serde(default)]
    may_change_read_count: Option<bool>,
    #[serde(default)]
    image_requirements: Option<ImageRequirements>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContainerManifest {
    pub image: String,
    pub digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolConstraints {
    pub runtime: String,
    pub mem_gb: u32,
    pub tmp_gb: u32,
    pub threads: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionContract {
    pub required_inputs: Vec<String>,
    pub expected_outputs: Vec<String>,
    pub forbidden_outputs: Vec<String>,
    pub forbid_unexpected_outputs: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolManifestV1 {
    pub schema_version: String,
    pub tool_id: String,
    pub stage_id: String,
    pub role: ToolRole,
    pub authoritative: bool,
    pub strict_capable: bool,
    pub status: ToolStatus,
    pub capabilities: Vec<String>,
    pub container: ContainerManifest,
    pub command_template: Vec<String>,
    pub outputs: Vec<PortSpec>,
    pub execution_contract: ExecutionContract,
    pub metrics_parser: String,
    pub constraints: ToolConstraints,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolRole {
    #[serde(alias = "gatekeeper", alias = "transform", alias = "report")]
    Authoritative,
    Diagnostic,
    Experimental,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolStatus {
    Stable,
    Experimental,
    Deprecated,
}

#[derive(Debug, Clone)]
pub struct ToolRegistry {
    stages: BTreeMap<String, StageManifestV1>,
    tools: BTreeMap<String, BTreeMap<String, ToolManifestV1>>,
}

impl ToolRegistry {
    #[must_use]
    pub fn stages(&self) -> &BTreeMap<String, StageManifestV1> {
        &self.stages
    }

    #[must_use]
    pub fn tools_for_stage(&self, stage_id: &str) -> Vec<&ToolManifestV1> {
        self.tools
            .get(stage_id)
            .map(|tools| tools.values().collect())
            .unwrap_or_default()
    }

    #[must_use]
    pub fn tool_by_id(&self, stage_id: &str, tool_id: &str) -> Option<&ToolManifestV1> {
        self.tools
            .get(stage_id)
            .and_then(|tools| tools.get(tool_id))
    }
}

/// Load all manifests from the given domain directory and validate them.
///
/// # Errors
/// Returns an error if manifests cannot be read, parsed, or validated.
pub fn load_manifests(modules_dir: &Path) -> Result<ToolRegistry, BijuxError> {
    let mut stages = BTreeMap::new();
    let mut tools: BTreeMap<String, BTreeMap<String, ToolManifestV1>> = BTreeMap::new();
    let mut stage_ids = BTreeSet::new();
    let mut tool_keys = BTreeSet::new();

    for entry in WalkDir::new(modules_dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        let is_stage = path
            .parent()
            .and_then(|parent| parent.file_name())
            .and_then(|name| name.to_str())
            == Some("stages");
        if is_stage && path.extension().and_then(|ext| ext.to_str()) == Some("yaml") {
            let manifest = match load_stage_manifest(path) {
                Ok(Some(manifest)) => manifest,
                Ok(None) => continue,
                Err(err) => return Err(err),
            };
            validate_stage_manifest(path, &manifest)?;
            if stage_ids.contains(&manifest.stage_id) {
                return Err(BijuxError::Manifest(format!(
                    "duplicate stage_id {} at {}",
                    manifest.stage_id,
                    path.display()
                )));
            }
            stage_ids.insert(manifest.stage_id.clone());
            stages.insert(manifest.stage_id.clone(), manifest);
        }
    }

    for entry in WalkDir::new(modules_dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if path.is_file() {
            let is_tool = path
                .parent()
                .and_then(|parent| parent.file_name())
                .and_then(|name| name.to_str())
                == Some("tools");
            if is_tool && path.extension().and_then(|ext| ext.to_str()) == Some("yaml") {
                let contents = std::fs::read_to_string(path)?;
                let manifest: ToolManifestV1 = serde_yaml::from_str(&contents)
                    .map_err(|err| BijuxError::Manifest(format!("{}: {err}", path.display())))?;
                validate_tool_manifest(path, &manifest)?;
                if !stages.contains_key(&manifest.stage_id) {
                    return Err(BijuxError::Manifest(format!(
                        "tool {} references unknown stage_id {} at {}",
                        manifest.tool_id,
                        manifest.stage_id,
                        path.display()
                    )));
                }
                let key = format!("{}::{}", manifest.stage_id, manifest.tool_id);
                if tool_keys.contains(&key) {
                    return Err(BijuxError::Manifest(format!(
                        "duplicate tool_id {} for stage {} at {}",
                        manifest.tool_id,
                        manifest.stage_id,
                        path.display()
                    )));
                }
                tool_keys.insert(key);
                tools
                    .entry(manifest.stage_id.clone())
                    .or_default()
                    .insert(manifest.tool_id.clone(), manifest);
            }
        }
    }

    Ok(ToolRegistry { stages, tools })
}

fn load_stage_manifest(path: &Path) -> Result<Option<StageManifestV1>, BijuxError> {
    let contents = std::fs::read_to_string(path)?;
    let doc: StageManifestDoc = serde_yaml::from_str(&contents)
        .map_err(|err| BijuxError::Manifest(format!("{}: {err}", path.display())))?;
    if doc.stage_id.is_none() && doc.extends.is_none() {
        return Ok(None);
    }
    let resolved = resolve_stage_doc(path, doc)?;
    Ok(Some(stage_doc_to_manifest(path, resolved)?))
}

fn resolve_stage_doc(path: &Path, doc: StageManifestDoc) -> Result<StageManifestDoc, BijuxError> {
    if let Some(extends) = &doc.extends {
        let base_path = path
            .parent()
            .ok_or_else(|| BijuxError::Manifest(format!("{} has no parent", path.display())))?
            .join(extends);
        let base_contents = std::fs::read_to_string(&base_path)?;
        let base_doc: StageManifestDoc = serde_yaml::from_str(&base_contents)
            .map_err(|err| BijuxError::Manifest(format!("{}: {err}", base_path.display())))?;
        let resolved_base = resolve_stage_doc(&base_path, base_doc)?;
        return Ok(merge_stage_docs(resolved_base, doc));
    }
    Ok(doc)
}

fn merge_stage_docs(base: StageManifestDoc, overlay: StageManifestDoc) -> StageManifestDoc {
    StageManifestDoc {
        extends: None,
        schema_version: overlay.schema_version.or(base.schema_version),
        stage_id: overlay.stage_id.or(base.stage_id),
        domain: overlay.domain.or(base.domain),
        inputs: overlay.inputs.or(base.inputs),
        outputs: overlay.outputs.or(base.outputs),
        parameters: overlay.parameters.or(base.parameters),
        metrics: overlay.metrics.or(base.metrics),
        description: overlay.description.or(base.description),
        mutates_fastq: overlay.mutates_fastq.or(base.mutates_fastq),
        report_only: overlay.report_only.or(base.report_only),
        may_change_read_count: overlay.may_change_read_count.or(base.may_change_read_count),
        image_requirements: overlay.image_requirements.or(base.image_requirements),
    }
}

fn stage_doc_to_manifest(
    path: &Path,
    doc: StageManifestDoc,
) -> Result<StageManifestV1, BijuxError> {
    let Some(schema_version) = doc.schema_version else {
        return Err(BijuxError::Manifest(format!(
            "missing schema_version for stage at {}",
            path.display()
        )));
    };
    let Some(stage_id) = doc.stage_id else {
        return Err(BijuxError::Manifest(format!(
            "missing stage_id for stage at {}",
            path.display()
        )));
    };
    let Some(domain) = doc.domain else {
        return Err(BijuxError::Manifest(format!(
            "missing domain for stage at {}",
            path.display()
        )));
    };
    let Some(inputs) = doc.inputs else {
        return Err(BijuxError::Manifest(format!(
            "missing inputs for stage at {}",
            path.display()
        )));
    };
    let Some(outputs) = doc.outputs else {
        return Err(BijuxError::Manifest(format!(
            "missing outputs for stage at {}",
            path.display()
        )));
    };
    let Some(parameters) = doc.parameters else {
        return Err(BijuxError::Manifest(format!(
            "missing parameters for stage at {}",
            path.display()
        )));
    };
    let Some(metrics) = doc.metrics else {
        return Err(BijuxError::Manifest(format!(
            "missing metrics for stage at {}",
            path.display()
        )));
    };
    let Some(description) = doc.description else {
        return Err(BijuxError::Manifest(format!(
            "missing description for stage at {}",
            path.display()
        )));
    };
    Ok(StageManifestV1 {
        schema_version,
        stage_id,
        domain,
        inputs,
        outputs,
        parameters,
        metrics,
        description,
        mutates_fastq: doc.mutates_fastq.unwrap_or(false),
        report_only: doc.report_only.unwrap_or(false),
        may_change_read_count: doc.may_change_read_count.unwrap_or(false),
        image_requirements: doc.image_requirements.unwrap_or_default(),
    })
}

fn validate_stage_manifest(path: &Path, manifest: &StageManifestV1) -> Result<(), BijuxError> {
    if manifest.schema_version != "bijux.stage.v1" {
        return Err(BijuxError::Manifest(format!(
            "invalid schema_version for stage at {}",
            path.display()
        )));
    }
    if manifest.stage_id.trim().is_empty() {
        return Err(BijuxError::Manifest(format!(
            "empty stage_id at {}",
            path.display()
        )));
    }
    Ok(())
}

fn validate_tool_manifest(path: &Path, manifest: &ToolManifestV1) -> Result<(), BijuxError> {
    if manifest.schema_version != "bijux.tool.v1" {
        return Err(BijuxError::Manifest(format!(
            "invalid schema_version for tool at {}",
            path.display()
        )));
    }
    if manifest.stage_id.trim().is_empty() || manifest.tool_id.trim().is_empty() {
        return Err(BijuxError::Manifest(format!(
            "empty stage_id or tool_id at {}",
            path.display()
        )));
    }
    if manifest.execution_contract.required_inputs.is_empty() {
        return Err(BijuxError::Manifest(format!(
            "execution_contract.required_inputs empty at {}",
            path.display()
        )));
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub run_id: RunId,
    pub stage: StageManifestV1,
    pub tool: ToolManifestV1,
    pub params: BTreeMap<String, String>,
    pub container: ContainerSpec,
    pub paths: PathSpec,
    pub profile: Profile,
    pub run_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub artifacts_dir: PathBuf,
    pub tmp_dir: PathBuf,
}

/// Build an execution plan from a run spec.
///
/// # Errors
/// Returns an error if the stage or tool cannot be resolved or manifests are invalid.
pub fn build_execution_plan(
    run_spec: RunSpec,
    registry: &ToolRegistry,
    profile: Profile,
    run_id: RunId,
) -> Result<ExecutionPlan, BijuxError> {
    let stage = registry
        .stages()
        .get(&run_spec.stage.0)
        .ok_or_else(|| BijuxError::Manifest(format!("unknown stage_id {}", run_spec.stage.0)))?
        .clone();

    let tool = registry
        .tool_by_id(&run_spec.stage.0, &run_spec.tool.0)
        .ok_or_else(|| {
            BijuxError::Manifest(format!(
                "unknown tool_id {} for stage {}",
                run_spec.tool.0, run_spec.stage.0
            ))
        })?
        .clone();

    if tool.stage_id != stage.stage_id {
        return Err(BijuxError::Manifest(format!(
            "tool {} references stage {}, expected {}",
            tool.tool_id, tool.stage_id, stage.stage_id
        )));
    }

    let run_dir = run_dir(
        &profile.run_base_dir,
        &run_id,
        &run_spec.stage,
        &run_spec.tool,
    );
    let logs_dir = run_dir.join("logs");
    let artifacts_dir = run_dir.join("artifacts");
    let tmp_dir = run_dir.join("tmp");

    let container = resolve_container_spec(&tool, &run_spec.paths, &tmp_dir, &profile)?;

    Ok(ExecutionPlan {
        run_id,
        stage,
        tool,
        params: run_spec.params,
        container,
        paths: run_spec.paths,
        profile,
        run_dir,
        logs_dir,
        artifacts_dir,
        tmp_dir,
    })
}

/// Resolve container information from a tool manifest and profile.
///
/// # Errors
/// Returns an error if the container digest is missing or malformed.
pub fn resolve_container_spec(
    tool: &ToolManifestV1,
    paths: &PathSpec,
    tmp_dir: &Path,
    profile: &Profile,
) -> Result<ContainerSpec, BijuxError> {
    if !tool.container.digest.starts_with("sha256:") {
        return Err(BijuxError::Manifest(format!(
            "container digest must be sha256 for tool {}",
            tool.tool_id
        )));
    }
    let image = format!("{}@{}", tool.container.image, tool.container.digest);

    let mut mounts = BTreeMap::new();
    mounts.insert("/data/input".to_string(), path_list_to_mount(&paths.input));
    mounts.insert(
        "/data/output".to_string(),
        path_list_to_mount(&paths.output),
    );
    mounts.insert(
        "/data/tmp".to_string(),
        tmp_dir.to_string_lossy().to_string(),
    );

    let mut env = BTreeMap::new();
    env.insert("THREADS".to_string(), profile.default_threads.to_string());
    env.insert("TMPDIR".to_string(), "/data/tmp".to_string());

    Ok(ContainerSpec {
        image,
        runtime: profile.container_runtime.clone(),
        mounts,
        env,
    })
}

fn path_list_to_mount(paths: &[PathBuf]) -> String {
    let mut unique = BTreeSet::new();
    for path in paths {
        if let Some(parent) = path.parent() {
            unique.insert(parent.to_path_buf());
        }
    }
    if unique.is_empty() {
        String::new()
    } else {
        unique
            .iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(":")
    }
}

/// Create run directories for a plan.
///
/// # Errors
/// Returns an error if directories cannot be created.
pub fn ensure_run_dirs(plan: &ExecutionPlan) -> Result<(), BijuxError> {
    std::fs::create_dir_all(&plan.logs_dir)?;
    std::fs::create_dir_all(&plan.artifacts_dir)?;
    std::fs::create_dir_all(&plan.tmp_dir)?;
    Ok(())
}

pub trait Executor {
    /// Execute the plan.
    ///
    /// # Errors
    /// Returns an error if execution fails.
    fn run(&self, plan: &ExecutionPlan) -> Result<RunReport, BijuxError>;
}

pub struct DryRunExecutor;

impl Executor for DryRunExecutor {
    fn run(&self, plan: &ExecutionPlan) -> Result<RunReport, BijuxError> {
        ensure_run_dirs(plan)?;
        let rendered = plan.tool.command_template.join(" ");
        info!(
            run_id = %plan.run_id.0,
            stage = %plan.stage.stage_id,
            tool = %plan.tool.tool_id,
            command = %rendered,
            "dry-run command"
        );

        let report = RunReport::new(
            RunId(plan.run_id.0.clone()),
            StageId(plan.stage.stage_id.clone()),
            ToolId(plan.tool.tool_id.clone()),
            RunStatus::Skipped,
        );
        let report_path = plan.run_dir.join("report.json");
        std::fs::write(report_path, serde_json::to_string_pretty(&report)?)?;
        Ok(report)
    }
}
