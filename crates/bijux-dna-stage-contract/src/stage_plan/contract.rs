use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use bijux_dna_core::contract::{StageIO, ToolConstraints};
use bijux_dna_core::ids::{StageId, StageVersion, StepId, ToolId};
use bijux_dna_core::prelude::{CommandSpecV1, ContainerImageRefV1};

use super::reason::PlanDecisionReason;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StagePlanV1 {
    pub stage_id: StageId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_instance_id: Option<StepId>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PlannedArtifactV1 {
    pub artifact_id: String,
    pub role: String,
    pub path: String,
    pub kind: String,
    pub schema: String,
}
