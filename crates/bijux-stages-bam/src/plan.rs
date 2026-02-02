use bijux_core::StagePlanV1;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StagePlanJson {
    pub stage_id: String,
    pub stage_version: String,
    pub io: bijux_core::StageIO,
    pub parameters: serde_json::Value,
    pub effective_params: serde_json::Value,
}

impl StagePlanJson {
    #[must_use]
    pub fn from_plan(plan: &StagePlanV1) -> Self {
        Self {
            stage_id: plan.stage_id.0.clone(),
            stage_version: plan.stage_version.0.to_string(),
            io: plan.io.clone(),
            parameters: plan.params.clone(),
            effective_params: plan.effective_params.clone(),
        }
    }
}
