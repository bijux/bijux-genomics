use std::path::PathBuf;

use bijux_core::{StageId, StageVersion};
use serde::{Deserialize, Serialize};

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

pub trait StagePlan {
    fn stage_id(&self) -> StageId;
    fn stage_version(&self) -> StageVersion;
    fn outputs(&self) -> StageIO;
    fn parameters_json(&self) -> serde_json::Value;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StagePlanJson {
    pub stage_id: String,
    pub stage_version: String,
    pub io: StageIO,
    pub parameters: serde_json::Value,
}

impl StagePlanJson {
    pub fn from_plan<T: StagePlan>(plan: &T) -> Self {
        let stage_id = plan.stage_id().0;
        let stage_version = plan.stage_version().0.to_string();
        Self {
            stage_id,
            stage_version,
            io: plan.outputs(),
            parameters: plan.parameters_json(),
        }
    }
}
