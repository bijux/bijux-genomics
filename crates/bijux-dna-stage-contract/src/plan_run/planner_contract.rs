use serde::{Deserialize, Serialize};

use bijux_dna_core::contract::{ArtifactRef, ToolConstraints};
use bijux_dna_core::prelude::ContainerImageRefV1;

use crate::{PlanDecisionReason, StagePlanV1};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlannerContractV1 {
    pub stage_id: String,
    pub tool_id: String,
    pub tool_version: Option<String>,
    pub image_ref: Option<ContainerImageRefV1>,
    pub parameters_json: serde_json::Value,
    pub effective_parameters_json: serde_json::Value,
    pub inputs: Vec<ArtifactRef>,
    pub outputs: Vec<ArtifactRef>,
    pub resources: ToolConstraints,
    pub reason: PlanDecisionReason,
}

impl From<&StagePlanV1> for PlannerContractV1 {
    fn from(stage: &StagePlanV1) -> Self {
        let tool_version = if stage.tool_version.trim().is_empty() {
            None
        } else {
            Some(stage.tool_version.trim().to_string())
        };
        let image_ref =
            if stage.image.image.trim().is_empty() { None } else { Some(stage.image.clone()) };
        Self {
            stage_id: stage.stage_id.to_string(),
            tool_id: stage.tool_id.to_string(),
            tool_version,
            image_ref,
            parameters_json: stage.params.clone(),
            effective_parameters_json: stage.effective_params.clone(),
            inputs: stage.io.inputs.clone(),
            outputs: stage.io.outputs.clone(),
            resources: stage.resources.clone(),
            reason: stage.reason.clone(),
        }
    }
}
