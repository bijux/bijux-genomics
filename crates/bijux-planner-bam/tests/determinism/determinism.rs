use std::collections::BTreeMap;

use bijux_core::contract::PlanPolicy;
use bijux_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_domain_bam::BamStage;
use bijux_planner_bam::{plan_bam_to_bam__adna_shotgun__v1, BamPipelineInputs};

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
fn bam_plan_is_deterministic() -> anyhow::Result<()> {
    let mut tool_specs = BTreeMap::new();
    for stage in BamStage::all() {
        tool_specs.insert(stage.as_str().to_string(), dummy_tool(stage.as_str()));
    }

    let temp = bijux_infra::temp_dir("bam-plan-determinism")?;
    let bam = temp.path().join("sample.bam");
    std::fs::write(&bam, b"")?;

    let inputs = BamPipelineInputs {
        policy: PlanPolicy::PreferAccuracy,
        tool_specs,
        params_overrides: BTreeMap::new(),
        bam: bam.clone(),
        bam_index: None,
        reference: None,
        sample_id: Some("sample".to_string()),
        out_dir: temp.path().join("out"),
    };

    let graph_a = plan_bam_to_bam__adna_shotgun__v1(&inputs)?;
    let graph_b = plan_bam_to_bam__adna_shotgun__v1(&inputs)?;
    assert_eq!(graph_a.hash()?, graph_b.hash()?);
    Ok(())
}
