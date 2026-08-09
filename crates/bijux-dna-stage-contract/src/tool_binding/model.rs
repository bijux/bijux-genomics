use std::collections::BTreeMap;

use bijux_dna_core::contract::ToolConstraints;
use bijux_dna_core::ids::{StageId, ToolId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Stable schema identity for declarative tool bindings.
pub const TOOL_BINDING_SCHEMA_V1: &str = "bijux.tool_binding.v1";

/// Describes how the executable artifact is admitted without performing admission here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ToolArtifactSourceV1 {
    /// A distributable image resolved through the governed container catalog.
    CatalogImage { catalog_tool_id: ToolId },
    /// An operator-supplied artifact that must be admitted by an execution owner.
    ExternalArtifact { artifact_kind: String, version_probe: Vec<String>, checksum_required: bool },
}

/// One named input projected into a tool command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BindingInputV1 {
    pub name: String,
    pub artifact_contract_id: String,
    pub required: bool,
}

/// One scientific parameter available to command rendering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BindingParameterV1 {
    pub name: String,
    pub required: bool,
}

/// Declares how a runtime owner discovers a projected output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OutputDiscoveryV1 {
    ExactPath,
    Glob { pattern: String },
}

/// One canonical output and its discovery contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BindingOutputV1 {
    pub name: String,
    pub artifact_contract_id: String,
    pub discovery: OutputDiscoveryV1,
}

/// Values supplied by a planner or runtime owner to the pure command renderer.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ToolBindingRenderContext {
    pub executable: String,
    pub inputs: BTreeMap<String, String>,
    pub parameters: BTreeMap<String, String>,
    pub outputs: BTreeMap<String, String>,
}

/// Read-only interface consumed by generic planning and conformance code.
pub trait ToolBinding {
    fn schema_version(&self) -> &str;
    fn binding_id(&self) -> &str;
    fn stage_id(&self) -> &StageId;
    fn tool_id(&self) -> &ToolId;
    fn artifact_source(&self) -> &ToolArtifactSourceV1;
    fn capabilities(&self) -> &[String];
    fn inputs(&self) -> &[BindingInputV1];
    fn parameters(&self) -> &[BindingParameterV1];
    fn outputs(&self) -> &[BindingOutputV1];
    fn normalized_output_contract_id(&self) -> &str;
    fn command_template(&self) -> &[String];
    fn resources(&self) -> &ToolConstraints;
    fn normalizer_id(&self) -> &str;
    fn downstream_probe_id(&self) -> &str;
    fn limitations(&self) -> &[String];
}

/// Serializable registry record implementing the generic binding interface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ToolBindingV1 {
    pub schema_version: String,
    pub binding_id: String,
    pub stage_id: StageId,
    pub tool_id: ToolId,
    pub artifact_source: ToolArtifactSourceV1,
    pub capabilities: Vec<String>,
    pub inputs: Vec<BindingInputV1>,
    pub parameters: Vec<BindingParameterV1>,
    pub outputs: Vec<BindingOutputV1>,
    /// Canonical stage result emitted by `normalizer_id` after raw output preservation.
    pub normalized_output_contract_id: String,
    pub command_template: Vec<String>,
    pub resources: ToolConstraints,
    pub normalizer_id: String,
    pub downstream_probe_id: String,
    #[serde(default)]
    pub limitations: Vec<String>,
}

impl ToolBinding for ToolBindingV1 {
    fn schema_version(&self) -> &str {
        &self.schema_version
    }

    fn binding_id(&self) -> &str {
        &self.binding_id
    }

    fn stage_id(&self) -> &StageId {
        &self.stage_id
    }

    fn tool_id(&self) -> &ToolId {
        &self.tool_id
    }

    fn artifact_source(&self) -> &ToolArtifactSourceV1 {
        &self.artifact_source
    }

    fn capabilities(&self) -> &[String] {
        &self.capabilities
    }

    fn inputs(&self) -> &[BindingInputV1] {
        &self.inputs
    }

    fn parameters(&self) -> &[BindingParameterV1] {
        &self.parameters
    }

    fn outputs(&self) -> &[BindingOutputV1] {
        &self.outputs
    }

    fn normalized_output_contract_id(&self) -> &str {
        &self.normalized_output_contract_id
    }

    fn command_template(&self) -> &[String] {
        &self.command_template
    }

    fn resources(&self) -> &ToolConstraints {
        &self.resources
    }

    fn normalizer_id(&self) -> &str {
        &self.normalizer_id
    }

    fn downstream_probe_id(&self) -> &str {
        &self.downstream_probe_id
    }

    fn limitations(&self) -> &[String] {
        &self.limitations
    }
}
