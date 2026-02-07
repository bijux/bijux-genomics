use bijux_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_planner_bam::{pipeline_stage_ids, plan_stage, StagePlanRequest};

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
fn bam_plan_reasons_include_defaults_and_contract_hash() -> anyhow::Result<()> {
    let stages = pipeline_stage_ids("bam-to-bam__default__v1");
    let temp = bijux_infra::temp_dir("bam-plan-reasons")?;
    let bam = temp.path().join("sample.bam");
    std::fs::write(&bam, b"")?;

    for stage_id in stages {
        let plan = plan_stage(StagePlanRequest {
            stage_id: &stage_id,
            tool: &dummy_tool(&stage_id),
            out_dir: temp.path(),
            bam: Some(&bam),
            bam_index: None,
            r1: None,
            r2: None,
            reference: None,
            sample_id: Some("sample"),
            params: None,
        })?;
        assert!(!plan.reason.summary.trim().is_empty());
        assert!(plan
            .reason
            .details
            .get("defaults_diff")
            .is_some_and(|value| value.is_object()));
        assert!(plan
            .reason
            .details
            .get("contract_hash")
            .is_some_and(|value| value.as_str().is_some()));
    }
    Ok(())
}
