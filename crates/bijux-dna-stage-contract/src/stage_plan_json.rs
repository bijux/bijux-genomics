use serde::{Deserialize, Serialize};

use bijux_dna_core::contract::StageIO;

use crate::{PlanDecisionReason, StagePlanV1};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StagePlanJsonV1 {
    pub stage_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_instance_id: Option<String>,
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
            stage_instance_id: plan.stage_instance_id.as_ref().map(ToString::to_string),
            stage_version,
            io: plan.io.clone(),
            parameters: plan.params.clone(),
            effective_params: plan.effective_params.clone(),
            reason: plan.reason.clone(),
        }
    }
}
