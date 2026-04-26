use std::collections::HashSet;

use bijux_dna_core::contract::PlanPolicy;
use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_dna_domain_bam::BamStage;
use bijux_dna_planner_bam::{pipeline_id_catalog, plan_stage, StagePlanRequest};
use bijux_dna_stage_contract::{default_edges_for_stages, ExecutionPlan, PlanValidationContext};

fn dummy_tool(tool_id: &str) -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId::new(tool_id),
        tool_version: "99.99.99+fixture".to_string(),
        image: ContainerImageRefV1 { image: "bijux/dummy:latest".to_string(), digest: None },
        command: CommandSpecV1 { template: vec!["echo".to_string(), tool_id.to_string()] },
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
    let temp = bijux_dna_infra::temp_dir("bam-plan-handshake")?;
    let bam = temp.path().join("sample.bam");
    let reference = temp.path().join("reference.fasta");
    std::fs::write(&bam, b"")?;
    std::fs::write(&reference, b">chrM\nACGT\n")?;

    let mut plans = Vec::new();
    let mut tool_ids = HashSet::new();
    for stage_id in &stages {
        let stage = BamStage::try_from(stage_id.as_str())?;
        let tool_id = bijux_dna_planner_bam::stage_api::default_tool_for_stage(stage);
        let tool = dummy_tool(tool_id.as_str());
        tool_ids.insert(tool.tool_id.to_string());
        let plan = plan_stage(StagePlanRequest {
            stage_id,
            tool: &tool,
            out_dir: temp.path(),
            bam: Some(&bam),
            bam_index: None,
            r1: None,
            r2: None,
            reference: Some(&reference),
            sample_id: Some("sample"),
            params: None,
        })?;
        plans.push(plan);
    }

    let edges = default_edges_for_stages(&plans);
    let plan = ExecutionPlan::new(
        "bam-to-bam__default__v1",
        bijux_dna_planner_bam::PLANNER_VERSION,
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
