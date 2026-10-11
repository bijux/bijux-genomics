use bijux_dna_core::ids::{StageId, ToolId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const DOWNSTREAM_PROBE_SCHEMA_V1: &str = "bijux.downstream_probe.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DownstreamProbeAssertionV1 {
    NormalizedIdentity,
    ScientificMetrics,
    DeclaredArtifacts,
    ScientificInvariants,
}

pub trait DownstreamProbe {
    fn schema_version(&self) -> &str;
    fn probe_id(&self) -> &str;
    fn producer_stage_id(&self) -> &StageId;
    fn input_contract_id(&self) -> &str;
    fn assertions(&self) -> &[DownstreamProbeAssertionV1];
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DownstreamProbeV1 {
    pub schema_version: String,
    pub probe_id: String,
    pub producer_stage_id: StageId,
    pub input_contract_id: String,
    pub assertions: Vec<DownstreamProbeAssertionV1>,
}

impl DownstreamProbe for DownstreamProbeV1 {
    fn schema_version(&self) -> &str {
        &self.schema_version
    }

    fn probe_id(&self) -> &str {
        &self.probe_id
    }

    fn producer_stage_id(&self) -> &StageId {
        &self.producer_stage_id
    }

    fn input_contract_id(&self) -> &str {
        &self.input_contract_id
    }

    fn assertions(&self) -> &[DownstreamProbeAssertionV1] {
        &self.assertions
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownstreamProbeCheck {
    pub assertion: DownstreamProbeAssertionV1,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownstreamProbeEvaluation {
    pub probe_id: String,
    pub binding_id: String,
    pub producer_stage_id: StageId,
    pub tool_id: ToolId,
    pub passed: bool,
    pub checks: Vec<DownstreamProbeCheck>,
}
