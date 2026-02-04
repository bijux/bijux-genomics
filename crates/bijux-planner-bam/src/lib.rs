use anyhow::Result;
use bijux_core::execution_plan::{default_edges_for_stages, ExecutionPlan, PlanPolicy};
use bijux_core::StagePlanV1;

pub const PLANNER_VERSION: &str = "bijux-planner-bam.v1";

#[derive(Debug, Clone)]
pub struct BamPlanConfig {
    pub pipeline_id: String,
    pub policy: PlanPolicy,
    pub stages: Vec<StagePlanV1>,
}

pub struct BamPlanner;

impl BamPlanner {
    /// # Errors
    /// Returns an error if the plan lint fails.
    pub fn plan(config: &BamPlanConfig) -> Result<ExecutionPlan> {
        let edges = default_edges_for_stages(&config.stages);
        ExecutionPlan::new(
            config.pipeline_id.clone(),
            PLANNER_VERSION,
            config.policy,
            config.stages.clone(),
            edges,
        )
    }
}
