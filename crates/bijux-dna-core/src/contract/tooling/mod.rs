#![allow(missing_docs)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::fmt::Write as _;

use crate::contract::canonical::to_canonical_json_bytes;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortSpec {
    pub name: String,
    pub data_type: String,
    pub cardinality: Cardinality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageParameterSpec {
    pub name: String,
    pub param_type: String,
    #[serde(default)]
    pub default: Option<String>,
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
