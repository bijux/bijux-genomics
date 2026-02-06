use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct ExplainExclusion {
    pub tool: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct ExplainPlan {
    pub stage: String,
    pub selected_tools: Vec<String>,
    pub excluded_tools: Vec<ExplainExclusion>,
    pub policy: Option<String>,
    pub invariants: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Stability: v1
pub struct PlanExplainStageV1 {
    pub step_id: String,
    pub image: String,
    pub command: Vec<String>,
    pub inputs: Vec<bijux_stage_contract::ArtifactRef>,
    pub outputs: Vec<bijux_stage_contract::ArtifactRef>,
    pub expected_artifact_ids: Vec<String>,
    pub metrics_schema_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// Stability: v1
pub struct PlanExplainV1 {
    pub schema_version: String,
    pub pipeline_id: String,
    pub planner_version: String,
    pub policy: bijux_core::execution::PlanPolicy,
    pub stages: Vec<PlanExplainStageV1>,
}

impl PlanExplainV1 {
    #[must_use]
    pub fn from_plan(plan: &bijux_core::execution::execution_graph::ExecutionGraph) -> Self {
        let stages = plan
            .steps()
            .iter()
            .map(|step| PlanExplainStageV1 {
                step_id: step.step_id.to_string(),
                image: step.image.image.clone(),
                command: step.command.template.clone(),
                inputs: step.io.inputs.clone(),
                outputs: step.io.outputs.clone(),
                expected_artifact_ids: step.expected_artifact_ids.clone(),
                metrics_schema_ids: step.metrics_schema_ids.clone(),
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
