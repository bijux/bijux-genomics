use bijux_core::{StageIO, StagePlan};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StagePlanJson {
    pub stage_id: String,
    pub stage_version: String,
    pub io: StageIO,
    pub parameters: serde_json::Value,
}

impl StagePlanJson {
    pub fn from_plan(plan: &StagePlan) -> Self {
        let stage_id = plan.stage_id.0.clone();
        let stage_version = plan.stage_version.0.to_string();
        Self {
            stage_id,
            stage_version,
            io: plan.io.clone(),
            parameters: plan.params.clone(),
        }
    }
}
