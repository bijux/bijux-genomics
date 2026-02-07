use std::collections::BTreeMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::stage_plan::StagePlanV1;
use bijux_core::contract::ArtifactRef;
use bijux_core::prelude::invariants::{InvariantResultV1, StageVerdictV1};
use bijux_core::metrics::MetricsEnvelope;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageInvocationV1 {
    pub command: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub expected_outputs: Vec<ArtifactRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StagePluginOutputV1 {
    pub metrics: MetricsEnvelope<serde_json::Value>,
    pub artifacts: Vec<ArtifactRef>,
    #[serde(default)]
    pub report_parts: Vec<StageReportPartV1>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub invariants: Vec<InvariantResultV1>,
    #[serde(default)]
    pub verdict: Option<StageVerdictV1>,
    #[serde(default)]
    pub event_hints: Vec<StageEventHintV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageReportPartV1 {
    pub name: String,
    pub file_name: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageEventHintV1 {
    pub event_name: String,
    pub status: String,
    pub attrs: serde_json::Value,
}

pub trait StagePlugin {
    fn handles_stage(&self, stage_id: &str) -> bool;
    /// # Errors
    /// Returns an error if the stage invocation cannot be materialized.
    fn materialize(&self, plan: &StagePlanV1) -> Result<StageInvocationV1>;
    /// # Errors
    /// Returns an error if outputs cannot be parsed into metrics/artifacts.
    fn parse_outputs(
        &self,
        plan: &StagePlanV1,
        outputs: &[ArtifactRef],
    ) -> Result<StagePluginOutputV1>;
}
