use super::*;

pub const DEFAULT_DOMAIN_DIR: &str = "domain";
pub const DEFAULT_CONFIGS_DIR: &str = "configs";
pub const DEFAULT_COMPILE_SCOPE: &str = "pre_hpc_pre_vcf";

#[derive(Debug, Clone)]
pub struct CompileOptions {
    pub domain_dir: PathBuf,
    pub configs_dir: PathBuf,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub struct ValidateOptions {
    pub domain_dir: PathBuf,
}

#[derive(Debug, Deserialize, Default)]
pub(super) struct DomainTool {
    pub(super) tool_id: String,
    #[serde(default)]
    pub(super) stage_ids: Vec<String>,
    #[serde(default)]
    pub(super) planned_stage_ids: Vec<String>,
    pub(super) status: String,
    pub(super) scope: String,
    pub(super) default_version: String,
    pub(super) upstream: String,
    pub(super) versioning_strategy: String,
    #[serde(default)]
    pub(super) pin_strategy: String,
    pub(super) license: String,
    pub(super) citation: String,
    pub(super) version_cmd: String,
    pub(super) help_cmd: String,
    pub(super) expected_artifacts: Vec<String>,
    #[serde(default)]
    pub(super) capabilities: Vec<String>,
    pub(super) metrics_schema_id: String,
    #[serde(default)]
    pub(super) metrics_schema: String,
    #[serde(default)]
    pub(super) comparability_notes: String,
    #[serde(default)]
    pub(super) container: Option<DomainToolContainer>,
}

#[derive(Debug, Deserialize, Default)]
pub(super) struct DomainToolLoose {
    #[serde(default)]
    pub(super) tool_id: String,
    #[serde(default)]
    pub(super) stage_ids: Vec<String>,
    #[serde(default)]
    pub(super) planned_stage_ids: Vec<String>,
    #[serde(default)]
    pub(super) status: String,
    #[serde(default)]
    pub(super) scope: String,
    #[serde(default)]
    pub(super) default_version: String,
    #[serde(default)]
    pub(super) upstream: String,
    #[serde(default)]
    pub(super) pin_strategy: String,
    #[serde(default)]
    pub(super) license: String,
    #[serde(default)]
    pub(super) citation: String,
    #[serde(default)]
    pub(super) version_cmd: String,
    #[serde(default)]
    pub(super) help_cmd: String,
    #[serde(default)]
    pub(super) expected_artifacts: Vec<String>,
    #[serde(default)]
    pub(super) capabilities: Vec<String>,
    #[serde(default)]
    pub(super) metrics_schema_id: String,
    #[serde(default)]
    pub(super) comparability_notes: String,
    #[serde(default)]
    pub(super) container: Option<DomainToolContainer>,
}

impl DomainTool {
    pub(super) fn declared_stage_ids(&self) -> impl Iterator<Item = &String> {
        self.stage_ids.iter().chain(self.planned_stage_ids.iter())
    }
}

impl DomainToolLoose {
    pub(super) fn declared_stage_ids(&self) -> impl Iterator<Item = &String> {
        self.stage_ids.iter().chain(self.planned_stage_ids.iter())
    }
}

#[derive(Debug, Deserialize, Default, Clone)]
pub(super) struct DomainToolContainer {
    #[serde(default)]
    pub(super) image: String,
    #[serde(default)]
    pub(super) digest: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, Eq)]
pub(super) struct StagePort {
    pub(super) name: String,
    pub(super) data_type: String,
    pub(super) cardinality: String,
}

#[derive(Debug, Deserialize, Default)]
pub(super) struct StageMetric {
    pub(super) name: String,
}

#[derive(Debug, Deserialize, Default)]
pub(super) struct DomainStage {
    pub(super) stage_id: String,
    pub(super) status: String,
    pub(super) scope: String,
    pub(super) domain: String,
    #[serde(default)]
    pub(super) inputs: Vec<StagePort>,
    #[serde(default)]
    pub(super) outputs: Vec<StagePort>,
    #[serde(default)]
    pub(super) required_inputs: Vec<String>,
    #[serde(default)]
    pub(super) required_outputs: Vec<String>,
    #[serde(default)]
    pub(super) metrics: Vec<StageMetric>,
    #[serde(default)]
    pub(super) compatible_tools: Vec<String>,
    #[serde(default)]
    pub(super) tool_capability_requirements: Vec<String>,
    #[serde(default)]
    pub(super) assumptions: Vec<String>,
    #[serde(default)]
    pub(super) bank_hooks: Vec<String>,
    #[serde(default)]
    pub(super) invariants: Vec<String>,
    #[serde(default)]
    pub(super) allowed_missingness: Vec<String>,
    #[serde(default)]
    pub(super) planned_out_of_scope: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
pub(super) struct DomainIndex {
    pub(super) domain: String,
    #[serde(default)]
    pub(super) domain_version: String,
    #[serde(default)]
    pub(super) stage_ids: Vec<String>,
    #[serde(default)]
    pub(super) tool_ids: Vec<String>,
    #[serde(default)]
    pub(super) stage_tool_compatibility: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub(super) active_defaults: BTreeMap<String, String>,
    #[serde(default)]
    pub(super) active_default_rationale: BTreeMap<String, String>,
    #[serde(default)]
    pub(super) stage_completeness_checklist: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub(super) stage_default_settings: BTreeMap<String, BTreeMap<String, BTreeMap<String, String>>>,
    #[serde(default)]
    pub(super) stage_comparability_mapping: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub(super) stage_min_quality_gates: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub(super) stage_failure_diagnosis_hints: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub(super) pipeline_compositions: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub(super) stage_ordering_constraints: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub(super) stage_prerequisites: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub(super) stage_resource_hints: BTreeMap<String, StageResourceHint>,
    #[serde(default)]
    pub(super) stage_output_size_estimates_mb: BTreeMap<String, BTreeMap<String, f64>>,
    #[serde(default)]
    pub(super) stage_sanity_metrics: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub(super) stage_qc_thresholds: BTreeMap<String, BTreeMap<String, ThresholdBand>>,
    #[serde(default)]
    pub(super) stage_contamination_thresholds: BTreeMap<String, BTreeMap<String, ThresholdBand>>,
    #[serde(default)]
    pub(super) stage_authenticity_thresholds: BTreeMap<String, BTreeMap<String, ThresholdBand>>,
    #[serde(default)]
    pub(super) stage_duplication_thresholds: BTreeMap<String, BTreeMap<String, ThresholdBand>>,
    #[serde(default)]
    pub(super) stage_coverage_sufficiency: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub(super) stage_sex_kinship_sufficiency: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub(super) benchmark_scenarios: BTreeMap<String, BenchmarkScenario>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub(super) struct StageResourceHint {
    #[serde(default)]
    pub(super) memory_gb: f64,
    #[serde(default)]
    pub(super) time_minutes: u64,
    #[serde(default)]
    pub(super) threads: u32,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub(super) struct ThresholdBand {
    #[serde(default)]
    pub(super) warn: String,
    #[serde(default)]
    pub(super) fail: String,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub(super) struct BenchmarkScenario {
    #[serde(default)]
    pub(super) stage_id: String,
    #[serde(default)]
    pub(super) description: String,
    #[serde(default)]
    pub(super) fairness_rules: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
pub(super) struct DomainArtifactVocabulary {
    #[serde(default)]
    pub(super) domain: String,
    #[serde(default)]
    pub(super) artifact_ids: Vec<String>,
    #[serde(default)]
    pub(super) artifacts: Vec<DomainArtifactEntry>,
}

#[derive(Debug, Deserialize, Default)]
pub(super) struct DomainMetricVocabulary {
    #[serde(default)]
    pub(super) domain: String,
    #[serde(default)]
    pub(super) metric_ids: Vec<String>,
    #[serde(default)]
    pub(super) metrics: Vec<DomainMetricEntry>,
}

#[derive(Debug, Deserialize, Default)]
pub(super) struct DomainArtifactEntry {
    pub(super) id: String,
}

#[derive(Debug, Deserialize, Default)]
pub(super) struct DomainMetricEntry {
    pub(super) id: String,
}

#[derive(Debug, Deserialize, Default)]
pub(super) struct AdapterBank {
    pub(super) schema_version: String,
    pub(super) bank_id: String,
    pub(super) version: String,
    #[serde(default)]
    pub(super) provenance_status: String,
    #[serde(default)]
    pub(super) adapters: Vec<AdapterEntry>,
}

#[derive(Debug, Deserialize, Default)]
pub(super) struct AdapterEntry {
    pub(super) id: String,
    pub(super) rationale: String,
    pub(super) source: String,
}

#[derive(Debug, Deserialize, Default)]
pub(super) struct ReferenceBank {
    pub(super) schema_version: String,
    pub(super) bank_id: String,
    pub(super) version: String,
    #[serde(default)]
    pub(super) provenance_status: String,
    #[serde(default)]
    pub(super) references: Vec<ReferenceEntry>,
}

#[derive(Debug, Deserialize, Default)]
pub(super) struct ReferenceEntry {
    pub(super) id: String,
    pub(super) kind: String,
    pub(super) source: String,
    pub(super) rationale: String,
}

#[derive(Debug, Deserialize, Default)]
pub(super) struct ContaminationDbBank {
    pub(super) schema_version: String,
    pub(super) bank_id: String,
    pub(super) version: String,
    #[serde(default)]
    pub(super) provenance_status: String,
    #[serde(default)]
    pub(super) databases: Vec<ContaminationDbEntry>,
}

#[derive(Debug, Deserialize, Default)]
pub(super) struct ContaminationDbEntry {
    pub(super) id: String,
    pub(super) db_version: String,
    pub(super) digest: String,
    pub(super) source: String,
    pub(super) rationale: String,
}

#[derive(Debug, Clone)]
pub(super) struct ToolRow {
    pub(super) id: String,
    pub(super) domain: String,
    pub(super) domains: Vec<String>,
    pub(super) stage_ids: Vec<String>,
    pub(super) bindings: Vec<String>,
    pub(super) tool_role: String,
    pub(super) default_version: String,
    pub(super) upstream: String,
    pub(super) pin_strategy: String,
    pub(super) version_cmd: String,
    pub(super) help_cmd: String,
    pub(super) expected_artifacts: Vec<String>,
    pub(super) metrics_schema: String,
    pub(super) status: String,
    pub(super) comparability_notes: String,
    pub(super) version_rule: String,
    pub(super) license: String,
    pub(super) citation: String,
    pub(super) container_image: String,
    pub(super) container_digest: String,
    pub(super) expected_version_regex: String,
    pub(super) healthcheck_cmd: String,
}

pub(super) type ToolMap = BTreeMap<String, ToolRow>;
pub(super) type StageToolMap = BTreeMap<String, BTreeSet<String>>;
pub(super) type StagePlannedMap = BTreeMap<String, Vec<String>>;
pub(super) type StageDefaultMap = BTreeMap<String, String>;
pub(super) type StageStatusMap = BTreeMap<String, String>;
pub(super) type StageOutputKindsMap = BTreeMap<String, Vec<String>>;
pub(super) type StageDefaultRationaleMap = BTreeMap<String, String>;
