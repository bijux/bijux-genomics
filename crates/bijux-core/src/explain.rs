use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainExclusion {
    pub tool: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainPlan {
    pub stage: String,
    pub selected_tools: Vec<String>,
    pub excluded_tools: Vec<ExplainExclusion>,
    pub policy: Option<String>,
    pub invariants: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanExplainStageV1 {
    pub stage_id: String,
    pub tool_id: String,
    pub tool_version: String,
    pub image: Option<String>,
    pub reason: crate::PlanDecisionReason,
    pub parameters_json: serde_json::Value,
    pub effective_parameters_json: serde_json::Value,
    pub inputs: Vec<crate::ArtifactRef>,
    pub outputs: Vec<crate::ArtifactRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanExplainV1 {
    pub schema_version: String,
    pub pipeline_id: String,
    pub planner_version: String,
    pub policy: crate::execution_plan::PlanPolicy,
    pub stages: Vec<PlanExplainStageV1>,
}
