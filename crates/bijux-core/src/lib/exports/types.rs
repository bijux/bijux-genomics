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
    EffectiveConfigV1, FactsRowV1, FilterReportV1, InvariantResultV1, InvariantStatusV1,
    MetricSemanticsV1, PipelineVerdictV1, ReportCompletenessV1, ReportContractV1,
    ReportProvenanceV1, ReportSchemaV1, ReportStageSummaryV1, RetentionContextV1,
    RetentionDefinitionV1, RetentionReportV1, StageObservabilityContextV1,
    StageObservabilityContractV1, StageReportV1, StageVerdictV1, TelemetryEventV1,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolTier {
    Gold,
    Silver,
    Experimental,
}
