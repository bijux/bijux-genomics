use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExplainExclusion {
    pub tool: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExplainPlan {
    pub stage: String,
    pub selected_tools: Vec<String>,
    pub excluded_tools: Vec<ExplainExclusion>,
    pub policy: Option<String>,
    pub invariants: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlanExplainStageV1 {
    pub stage_id: String,
    pub tool_id: String,
    pub tool_version: String,
    pub image: Option<String>,
    pub reason: crate::plan::stage_plan::PlanDecisionReason,
    pub parameters_json: serde_json::Value,
    pub effective_parameters_json: serde_json::Value,
    pub inputs: Vec<crate::plan::stage_plan::ArtifactRef>,
    pub outputs: Vec<crate::plan::stage_plan::ArtifactRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlanExplainV1 {
    pub schema_version: String,
    pub pipeline_id: String,
    pub planner_version: String,
    pub policy: crate::plan::execution_plan::PlanPolicy,
    pub stages: Vec<PlanExplainStageV1>,
}

impl PlanExplainV1 {
    #[must_use]
    pub fn from_plan(plan: &crate::plan::execution_plan::ExecutionPlan) -> Self {
        let stages = plan
            .stages()
            .iter()
            .map(|stage| PlanExplainStageV1 {
                stage_id: stage.stage_id.to_string(),
                tool_id: stage.tool_id.to_string(),
                tool_version: stage.tool_version.clone(),
                image: if stage.image.image.is_empty() {
                    None
                } else {
                    Some(stage.image.image.clone())
                },
                reason: stage.reason.clone(),
                parameters_json: stage.params.clone(),
                effective_parameters_json: stage.effective_params.clone(),
                inputs: stage.io.inputs.clone(),
                outputs: stage.io.outputs.clone(),
            })
            .collect();
        Self {
            schema_version: "bijux.plan_explain.v1".to_string(),
            pipeline_id: plan.pipeline_id().to_string(),
            planner_version: plan.planner_version().to_string(),
            policy: plan.policy(),
            stages,
        }
    }
}
