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
    pub policy: bijux_core::contract::PlanPolicy,
    pub stages: Vec<PlanExplainStageV1>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExplainToolSelection {
    pub stage_id: String,
    pub tool_id: String,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExplainResponse {
    pub selected_tools: Vec<ExplainToolSelection>,
    pub defaults_ledger_diff: serde_json::Value,
    pub stage_contracts: serde_json::Value,
}

impl PlanExplainV1 {
    #[must_use]
    pub fn from_plan(plan: &bijux_core::contract::ExecutionGraph) -> Self {
        let stages = plan
            .steps()
            .iter()
            .map(|step| PlanExplainStageV1 {
                step_id: step.step_id.to_string(),
                image: step.image.image.clone(),
                command: step.command.template.clone(),
                inputs: step.io.inputs.clone(),
                outputs: step.io.outputs.clone(),
                expected_artifact_ids: step
                    .expected_artifact_ids
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect(),
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

#[must_use]
pub fn explain_bundle(
    plan: &bijux_core::contract::ExecutionGraph,
    defaults_ledger: Option<&serde_json::Value>,
) -> ExplainResponse {
    let selected_tools = plan
        .steps()
        .iter()
        .map(|step| ExplainToolSelection {
            stage_id: step.stage_id.to_string(),
            tool_id: step.image.image.clone(),
            reason: None,
        })
        .collect::<Vec<_>>();
    let stage_contracts = plan
        .steps()
        .iter()
        .filter_map(|step| {
            let stage_id = step.stage_id.to_string();
            let hash = if stage_id.starts_with("fastq.") || stage_id.starts_with("core.") {
                bijux_domain_fastq::stage_contract_hash(&stage_id).and_then(std::result::Result::ok)
            } else if stage_id.starts_with("bam.") {
                bijux_domain_bam::stage_contract_hash(&stage_id).and_then(std::result::Result::ok)
            } else {
                None
            };
            hash.map(|hash| (stage_id, serde_json::Value::String(hash)))
        })
        .collect::<serde_json::Map<_, _>>();
    ExplainResponse {
        selected_tools,
        defaults_ledger_diff: defaults_ledger
            .cloned()
            .unwrap_or_else(|| serde_json::json!({})),
        stage_contracts: serde_json::Value::Object(stage_contracts),
    }
}
