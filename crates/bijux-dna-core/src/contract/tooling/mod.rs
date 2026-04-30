#![allow(missing_docs)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::fmt::Write as _;

use crate::contract::canonical::to_canonical_json_bytes;
use crate::contract::ArtifactRole;
use crate::foundation::Result;
use crate::ids::{StageId, ToolId, ToolVersion};

mod selection;
pub use selection::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConstraints {
    pub runtime: String,
    pub mem_gb: u32,
    pub tmp_gb: u32,
    pub threads: u32,
}

impl Default for ToolConstraints {
    fn default() -> Self {
        Self { runtime: "local".to_string(), mem_gb: 1, tmp_gb: 1, threads: 1 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Cardinality {
    One,
    Many,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortSpec {
    pub name: String,
    pub data_type: String,
    pub cardinality: Cardinality,
    #[serde(default)]
    pub artifact_role: ArtifactRole,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StageParameterSpec {
    pub name: String,
    pub param_type: String,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageMetricSpec {
    pub name: String,
    #[serde(default)]
    pub meaning: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StageSemanticKind {
    #[default]
    Transform,
    Filter,
    Annotate,
    Qc,
    Report,
    Index,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StageFamily {
    #[default]
    Fastq,
    Bam,
    Vcf,
    Cross,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    Fastq,
    Bam,
    Vcf,
    Report,
    Index,
    Metrics,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeScale {
    Tiny,
    #[default]
    Small,
    Medium,
    Large,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StageOperatingMode {
    Simulation,
    Advisory,
    #[default]
    Enforced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BackendVersionPolicy {
    Floating,
    #[default]
    Pinned,
    DigestPinned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ReportSeverity {
    Info,
    #[default]
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StageReportKind {
    #[default]
    Qc,
    Contamination,
    Damage,
    Coverage,
    Normalization,
    Imputation,
    PopulationSummary,
    Generic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, Ord, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub enum ReadLayoutMode {
    SingleEnd,
    PairedEnd,
    Interleaved,
    Deinterleaved,
    Merged,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, Ord, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub enum CompressionSupport {
    None,
    Gzip,
    Bgzf,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceRequirement {
    #[default]
    None,
    Optional,
    Required,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum IndexRequirement {
    #[default]
    None,
    Optional,
    Required,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Ord, PartialOrd)]
pub struct UnsupportedParameterCombination {
    #[serde(default)]
    pub parameters: BTreeMap<String, String>,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct StageCapabilitySpec {
    #[serde(default)]
    pub layouts: Vec<ReadLayoutMode>,
    #[serde(default)]
    pub compression: Vec<CompressionSupport>,
    #[serde(default)]
    pub reference: ReferenceRequirement,
    #[serde(default)]
    pub index: IndexRequirement,
    #[serde(default)]
    pub output_formats: Vec<String>,
    #[serde(default)]
    pub unsupported_parameter_combinations: Vec<UnsupportedParameterCombination>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct StageEnvironmentRequirements {
    #[serde(default)]
    pub variables: Vec<String>,
    #[serde(default)]
    pub mounts: Vec<String>,
    #[serde(default)]
    pub executables: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct StageReportContract {
    pub report_id: String,
    #[serde(default)]
    pub kind: StageReportKind,
    pub schema_version: String,
    #[serde(default)]
    pub required_fields: Vec<String>,
    #[serde(default)]
    pub advisory_fields: Vec<String>,
    #[serde(default)]
    pub severity: ReportSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StageRefusalCode {
    IncompatibleInputs,
    MissingReference,
    UnsupportedLayout,
    MissingIndex,
    UnsafeOverride,
    BackendUnavailable,
    ScientificIncoherence,
    UntypedArtifactRole,
    MalformedReport,
    UnsupportedMode,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImageRequirements {
    #[serde(default)]
    pub needs: Vec<String>,
    #[serde(default)]
    pub forbids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageSpec {
    pub stage_id: StageId,
    #[serde(default)]
    pub stage_family: StageFamily,
    #[serde(default)]
    pub semantic_kind: StageSemanticKind,
    #[serde(default)]
    pub input_kind: ArtifactKind,
    #[serde(default)]
    pub output_kind: ArtifactKind,
    #[serde(default)]
    pub produced_artifacts: Vec<String>,
    #[serde(default = "default_stage_semver")]
    pub stage_semver: String,
    #[serde(default = "default_runtime_scale")]
    pub runtime_scale: RuntimeScale,
    #[serde(default)]
    pub inputs: Vec<PortSpec>,
    #[serde(default)]
    pub outputs: Vec<PortSpec>,
    #[serde(default)]
    pub parameters: Vec<StageParameterSpec>,
    #[serde(default)]
    pub metrics: Vec<StageMetricSpec>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub environment_requirements: StageEnvironmentRequirements,
    #[serde(default)]
    pub report_contracts: Vec<StageReportContract>,
    #[serde(default)]
    pub capability_contract: StageCapabilitySpec,
    #[serde(default)]
    pub refusal_codes: Vec<StageRefusalCode>,
    #[serde(default)]
    pub operating_mode: StageOperatingMode,
    #[serde(flatten, default)]
    pub behavior: StageBehavior,
    #[serde(default)]
    pub image_requirements: Option<ImageRequirements>,
    #[serde(default)]
    pub extends: Option<StageId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StageBehavior {
    #[serde(default)]
    pub idempotent: bool,
    #[serde(default)]
    pub mutates_fastq: bool,
    #[serde(default)]
    pub report_only: bool,
    #[serde(default, with = "read_count_change_policy_bool")]
    pub read_count_change: ReadCountChangePolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum ReadCountChangePolicy {
    #[default]
    Stable,
    MayChange,
}

impl ReadCountChangePolicy {
    #[must_use]
    pub const fn from_bool(may_change: bool) -> Self {
        if may_change {
            Self::MayChange
        } else {
            Self::Stable
        }
    }
}

mod read_count_change_policy_bool {
    use serde::{Deserialize, Deserializer, Serializer};

    use super::ReadCountChangePolicy;

    pub fn serialize<S>(value: &ReadCountChangePolicy, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bool(matches!(value, ReadCountChangePolicy::MayChange))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<ReadCountChangePolicy, D::Error>
    where
        D: Deserializer<'de>,
    {
        let may_change = bool::deserialize(deserializer)?;
        Ok(ReadCountChangePolicy::from_bool(may_change))
    }
}

fn default_stage_semver() -> String {
    "1.0.0".to_string()
}

const fn default_runtime_scale() -> RuntimeScale {
    RuntimeScale::Small
}

impl StageSpec {
    /// # Errors
    /// Returns an error if serialization fails.
    pub fn hash(&self) -> Result<String> {
        let bytes = to_canonical_json_bytes(self)?;
        let mut hasher = sha2::Sha256::new();
        hasher.update(bytes);
        let digest = hasher.finalize();
        let mut hex = String::with_capacity(digest.len() * 2);
        for byte in digest {
            let _ = write!(&mut hex, "{byte:02x}");
        }
        Ok(hex)
    }

    #[must_use]
    pub fn canonical_contract(&self, tool_manifest: &ToolManifest) -> CanonicalStageContractV1 {
        CanonicalStageContractV1 {
            stage_id: self.stage_id.clone(),
            stage_family: self.stage_family,
            semantic_kind: self.semantic_kind,
            backend_tool_id: tool_manifest.tool_id.clone(),
            backend_version_policy: tool_manifest.backend_version_policy,
            input_artifacts: self.inputs.clone(),
            output_artifacts: self.outputs.clone(),
            parameters: self.parameters.clone(),
            environment_requirements: self.environment_requirements.clone(),
            report_contracts: self.report_contracts.clone(),
            refusal_codes: self.refusal_codes.clone(),
            operating_mode: self.operating_mode,
            capability_contract: merged_capability_contract(
                &self.capability_contract,
                &tool_manifest.capability_contract,
            ),
        }
    }

    /// # Errors
    /// Returns an error if canonical JSON serialization fails.
    pub fn canonicalize_parameters(
        &self,
        params: &BTreeMap<String, String>,
    ) -> Result<CanonicalStageParametersV1> {
        let mut normalized = BTreeMap::<String, String>::new();
        let mut applied_defaults = Vec::new();
        let mut resolved_aliases = BTreeMap::new();
        let alias_to_name = self
            .parameters
            .iter()
            .flat_map(|spec| {
                spec.aliases.iter().map(|alias| (alias.to_ascii_lowercase(), spec.name.clone()))
            })
            .collect::<BTreeMap<_, _>>();

        for (raw_name, raw_value) in params {
            let canonical_name = alias_to_name
                .get(&raw_name.to_ascii_lowercase())
                .cloned()
                .unwrap_or_else(|| raw_name.clone());
            if canonical_name != *raw_name {
                resolved_aliases.insert(raw_name.clone(), canonical_name.clone());
            }
            normalized.insert(canonical_name, raw_value.clone());
        }

        for spec in &self.parameters {
            if normalized.contains_key(&spec.name) {
                continue;
            }
            if let Some(default) = &spec.default {
                normalized.insert(spec.name.clone(), default.clone());
                applied_defaults.push(spec.name.clone());
            }
        }

        let normalized_json = crate::contract::canonical::parameters_json_canonicalization(
            &serde_json::json!(normalized),
        );
        let hash = crate::foundation::hashing::params_hash(&normalized_json)?;
        Ok(CanonicalStageParametersV1 { normalized_json, hash, applied_defaults, resolved_aliases })
    }
}

#[must_use]
pub fn merged_capability_contract(
    stage: &StageCapabilitySpec,
    tool: &StageCapabilitySpec,
) -> StageCapabilitySpec {
    StageCapabilitySpec {
        layouts: merge_unique(&stage.layouts, &tool.layouts),
        compression: merge_unique(&stage.compression, &tool.compression),
        reference: if matches!(tool.reference, ReferenceRequirement::None) {
            stage.reference
        } else {
            tool.reference
        },
        index: if matches!(tool.index, IndexRequirement::None) { stage.index } else { tool.index },
        output_formats: merge_unique(&stage.output_formats, &tool.output_formats),
        unsupported_parameter_combinations: stage
            .unsupported_parameter_combinations
            .iter()
            .cloned()
            .chain(tool.unsupported_parameter_combinations.iter().cloned())
            .collect(),
    }
}

fn merge_unique<T>(left: &[T], right: &[T]) -> Vec<T>
where
    T: Clone + Ord + PartialEq,
{
    let mut merged = left.iter().cloned().collect::<Vec<_>>();
    for item in right {
        if merged.contains(item) {
            continue;
        }
        merged.push(item.clone());
    }
    merged.sort();
    merged
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ToolRole {
    #[default]
    Authoritative,
    Diagnostic,
    Experimental,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutionContract {
    #[serde(default)]
    pub required_inputs: Vec<String>,
    #[serde(default)]
    pub optional_inputs: Vec<String>,
    #[serde(default)]
    pub expected_outputs: Vec<String>,
    #[serde(default)]
    pub optional_outputs: Vec<String>,
    #[serde(default)]
    pub forbidden_outputs: Vec<String>,
    #[serde(default)]
    pub forbid_unexpected_outputs: bool,
    #[serde(default)]
    pub requires_provenance: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolManifest {
    pub tool_id: ToolId,
    pub stage_id: StageId,
    #[serde(default)]
    pub role: ToolRole,
    #[serde(default)]
    pub command_template: Vec<String>,
    #[serde(default)]
    pub outputs: Vec<PortSpec>,
    #[serde(default)]
    pub metrics_parser: Option<String>,
    #[serde(default)]
    pub constraints: ToolConstraints,
    #[serde(default)]
    pub execution_contract: ExecutionContract,
    #[serde(default)]
    pub supported_modes: Vec<StageOperatingMode>,
    #[serde(default)]
    pub backend_version_policy: BackendVersionPolicy,
    #[serde(default)]
    pub capability_contract: StageCapabilitySpec,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalStageContractV1 {
    pub stage_id: StageId,
    pub stage_family: StageFamily,
    pub semantic_kind: StageSemanticKind,
    pub backend_tool_id: ToolId,
    pub backend_version_policy: BackendVersionPolicy,
    pub input_artifacts: Vec<PortSpec>,
    pub output_artifacts: Vec<PortSpec>,
    pub parameters: Vec<StageParameterSpec>,
    pub environment_requirements: StageEnvironmentRequirements,
    pub report_contracts: Vec<StageReportContract>,
    pub refusal_codes: Vec<StageRefusalCode>,
    pub operating_mode: StageOperatingMode,
    pub capability_contract: StageCapabilitySpec,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalStageParametersV1 {
    pub normalized_json: serde_json::Value,
    pub hash: String,
    #[serde(default)]
    pub applied_defaults: Vec<String>,
    #[serde(default)]
    pub resolved_aliases: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionSpecV1 {
    pub tool_id: ToolId,
    pub tool_version: ToolVersion,
    pub image: crate::foundation::ContainerImageRefV1,
    pub command: crate::foundation::CommandSpecV1,
    pub resources: ToolConstraints,
}

#[derive(Debug, Clone, Default)]
pub struct ToolRegistry {
    stages: BTreeMap<StageId, StageSpec>,
    tools: BTreeMap<StageId, Vec<ToolManifest>>,
}

impl ToolRegistry {
    #[must_use]
    pub fn stages(&self) -> &BTreeMap<StageId, StageSpec> {
        &self.stages
    }

    #[must_use]
    pub fn tools_for_stage(&self, stage_id: &StageId) -> &[ToolManifest] {
        self.tools.get(stage_id).map_or(&[], Vec::as_slice)
    }

    #[must_use]
    pub fn tool_by_id(&self, stage_id: &StageId, tool_id: &ToolId) -> Option<&ToolManifest> {
        self.tools_for_stage(stage_id).iter().find(|tool| &tool.tool_id == tool_id)
    }

    pub fn insert_stage(&mut self, stage: StageSpec) {
        self.stages.insert(stage.stage_id.clone(), stage);
    }

    pub fn insert_tool(&mut self, tool: ToolManifest) {
        self.tools.entry(tool.stage_id.clone()).or_default().push(tool);
    }

    pub fn sort_tools_for_determinism(&mut self) {
        for tools in self.tools.values_mut() {
            tools.sort_by(|a, b| a.tool_id.cmp(&b.tool_id));
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathSpec {
    pub input: Vec<PathBuf>,
    pub output: Vec<PathBuf>,
    pub work: PathBuf,
}
