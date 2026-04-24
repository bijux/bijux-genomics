use anyhow::Result;
use bijux_dna_core::contract::{ExecutionEdge, ExecutionGraph, PlanPolicy};
use bijux_dna_core::prelude::StepId;
use bijux_dna_stage_contract::{default_edges_for_stages, StagePlanV1};

use crate::PLANNER_VERSION;

/// # Errors
/// Returns an error if the stage plans cannot be converted into an execution graph.
pub fn from_stage_plans(
    pipeline_id: &str,
    policy: PlanPolicy,
    stages: &[StagePlanV1],
    log_message: &'static str,
) -> Result<ExecutionGraph> {
    let edges = default_edges_for_stages(stages);
    let graph = ExecutionGraph::new(
        pipeline_id,
        PLANNER_VERSION,
        policy,
        stages.iter().map(bijux_dna_stage_contract::execution_step_from_stage_plan).collect(),
        edges
            .into_iter()
            .map(|edge| {
                ExecutionEdge::new(
                    StepId::new(edge.from().to_string()),
                    StepId::new(edge.to().to_string()),
                )
            })
            .collect(),
    )?;
    tracing::info!(
        target: "plan.graph",
        pipeline_id = %graph.pipeline_id(),
        steps = graph.steps().len(),
        edges = graph.edges().len(),
        "{log_message}"
    );
    Ok(graph)
}
