use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use bijux_core::contract::{StageIO, ToolConstraints};
use bijux_core::ids::{StageId, StageVersion, ToolId};
use bijux_core::primitives::{CommandSpecV1, ContainerImageRefV1};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanReasonKind {
    Default,
    Profile,
    Override,
    Fallback,
    Compatibility,
    InputAssessed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PlanDecisionReason {
    pub kind: PlanReasonKind,
    pub summary: String,
    #[serde(default)]
    pub details: serde_json::Value,
}

impl PlanDecisionReason {
    #[must_use]
    pub fn new(kind: PlanReasonKind, summary: impl Into<String>) -> Self {
        Self {
            kind,
            summary: summary.into(),
            details: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
}

impl Default for PlanDecisionReason {
    fn default() -> Self {
        Self {
            kind: PlanReasonKind::Default,
            summary: "planner default".to_string(),
            details: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StagePlanV1 {
    pub stage_id: StageId,
    pub stage_version: StageVersion,
    pub tool_id: ToolId,
    pub tool_version: String,
    pub image: ContainerImageRefV1,
    pub command: CommandSpecV1,
    pub resources: ToolConstraints,
    pub io: StageIO,
    pub out_dir: PathBuf,
    pub params: serde_json::Value,
    #[serde(default)]
    pub effective_params: serde_json::Value,
    #[serde(default)]
    pub aux_images: BTreeMap<String, ContainerImageRefV1>,
    #[serde(default)]
    pub reason: PlanDecisionReason,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StagePlanJsonV1 {
    pub stage_id: String,
    pub stage_version: String,
    pub io: StageIO,
    pub parameters: serde_json::Value,
    pub effective_params: serde_json::Value,
    #[serde(default)]
    pub reason: PlanDecisionReason,
}

impl StagePlanJsonV1 {
    #[must_use]
    pub fn from_plan(plan: &StagePlanV1) -> Self {
        let stage_id = plan.stage_id.to_string();
        let stage_version = plan.stage_version.0.to_string();
        Self {
            stage_id,
            stage_version,
            io: plan.io.clone(),
            parameters: plan.params.clone(),
            effective_params: plan.effective_params.clone(),
            reason: plan.reason.clone(),
        }
    }
}
