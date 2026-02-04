use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{StageId, StageVersion, ToolConstraints, ToolId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactRef {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StageIO {
    pub inputs: Vec<ArtifactRef>,
    pub outputs: Vec<ArtifactRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommandSpecV1 {
    pub template: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContainerImageRefV1 {
    pub image: String,
    pub digest: Option<String>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StagePlanJsonV1 {
    pub stage_id: String,
    pub stage_version: String,
    pub io: StageIO,
    pub parameters: serde_json::Value,
    pub effective_params: serde_json::Value,
}

impl StagePlanJsonV1 {
    #[must_use]
    pub fn from_plan(plan: &StagePlanV1) -> Self {
        let stage_id = plan.stage_id.0.clone();
        let stage_version = plan.stage_version.0.to_string();
        Self {
            stage_id,
            stage_version,
            io: plan.io.clone(),
            parameters: plan.params.clone(),
            effective_params: plan.effective_params.clone(),
        }
    }
}
