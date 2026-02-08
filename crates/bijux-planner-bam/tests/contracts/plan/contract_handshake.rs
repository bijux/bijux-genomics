use std::collections::HashSet;

use bijux_core::contract::PlanPolicy;
use bijux_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_planner_bam::{pipeline_id_catalog, plan_stage, StagePlanRequest};
use bijux_stage_contract::{default_edges_for_stages, ExecutionPlan, PlanValidationContext};

fn dummy_tool(stage: &str) -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId::new(format!("tool.{stage}")),
        tool_version: "0.0.0".to_string(),
        image: ContainerImageRefV1 {
            image: "bijux/dummy:latest".to_string(),
            digest: None,
        },
        command: CommandSpecV1 {
            template: vec!["echo".to_string(), stage.to_string()],
        },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
    }
}

#[test]
fn bam_plan_validates_against_contracts() -> anyhow::Result<()> {
    let stages = pipeline_id_catalog("bam-to-bam__default__v1");
    let temp = bijux_infra::temp_dir("bam-plan-handshake")?;
    let bam = temp.path().join("sample.bam");
    std::fs::write(&bam, b"")?;

    let mut plans = Vec::new();
    let mut tool_ids = HashSet::new();
    for stage_id in &stages {
        let tool = dummy_tool(stage_id);
        tool_ids.insert(tool.tool_id.to_string());
        let plan = plan_stage(StagePlanRequest {
            stage_id,
            tool: &tool,
            out_dir: temp.path(),
            bam: Some(&bam),
            bam_index: None,
            r1: None,
            r2: None,
            reference: None,
            sample_id: Some("sample"),
            params: None,
        })?;
        plans.push(plan);
    }

    let edges = default_edges_for_stages(&plans);
    let plan = ExecutionPlan::new(
        "bam-to-bam__default__v1",
        bijux_planner_bam::PLANNER_VERSION,
        PlanPolicy::PreferAccuracy,
        plans,
        edges,
    )?;
    let allowed_id_catalog = stages.into_iter().collect::<HashSet<_>>();
    let context = PlanValidationContext {
        allowed_id_catalog: Some(&allowed_id_catalog),
        allowed_tool_ids: Some(&tool_ids),
    };
    plan.validate_strict(&context)?;
    Ok(())
}
