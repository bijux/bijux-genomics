use std::collections::{BTreeMap, HashSet};

use bijux_dna_core::contract::PlanPolicy;
use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, StageId, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_dna_planner_fastq::stage_api::default_tool_for_stage;
use bijux_dna_planner_fastq::{
    compose_fastq_pipeline_steps, default_pipeline_spec, DefaultPipelineOptions,
};
use bijux_dna_stage_contract::{default_edges_for_stages, ExecutionPlan, PlanValidationContext};

fn tool_for_stage(stage: &str) -> ToolExecutionSpecV1 {
    let stage_id = StageId::new(stage);
    let tool_id = default_tool_for_stage(&stage_id)
        .map_or_else(|| "fastp".to_string(), |tool| tool.to_string());
    ToolExecutionSpecV1 {
        tool_id: ToolId::new(tool_id),
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
fn fastq_plan_validates_against_contracts() -> anyhow::Result<()> {
    let pipeline = default_pipeline_spec(DefaultPipelineOptions::default());
    let stages = pipeline.stages;
    let tools = stages
        .iter()
        .map(|stage| tool_for_stage(stage))
        .collect::<Vec<_>>();
    let temp = bijux_dna_infra::temp_dir("fastq-plan-handshake")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let plans = compose_fastq_pipeline_steps(
        &stages,
        &tools,
        &BTreeMap::new(),
        None,
        None,
        None,
        None,
        false,
        &r1,
        None,
        |stage_id, tool, _r1, _r2| Ok(temp.path().join(stage_id).join(tool.tool_id.as_str())),
    )?;
    let edges = default_edges_for_stages(&plans);
    let plan = ExecutionPlan::new(
        "fastq-to-fastq__default__v1",
        bijux_dna_planner_fastq::PLANNER_VERSION,
        PlanPolicy::PreferAccuracy,
        plans,
        edges,
    )?;
    let allowed_id_catalog = stages.into_iter().collect::<HashSet<_>>();
    let allowed_tool_ids = tools
        .into_iter()
        .map(|tool| tool.tool_id.to_string())
        .collect::<HashSet<_>>();
    let context = PlanValidationContext {
        allowed_id_catalog: Some(&allowed_id_catalog),
        allowed_tool_ids: Some(&allowed_tool_ids),
    };
    plan.validate_strict(&context)?;
    Ok(())
}
