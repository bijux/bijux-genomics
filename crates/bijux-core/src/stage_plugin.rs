use std::collections::BTreeMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{ArtifactRef, StagePlanV1};

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
    pub metrics: serde_json::Value,
    pub artifacts: Vec<ArtifactRef>,
}

pub trait StagePlugin {
    fn handles_stage(&self, stage_id: &str) -> bool;
    fn materialize(&self, plan: &StagePlanV1) -> Result<StageInvocationV1>;
    fn parse_outputs(&self, plan: &StagePlanV1, outputs: &[ArtifactRef]) -> Result<StagePluginOutputV1>;
}
