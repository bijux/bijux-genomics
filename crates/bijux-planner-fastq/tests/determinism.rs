use std::collections::BTreeMap;

use bijux_core::contract::PlanPolicy;
use bijux_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_planner_fastq::{
    default_pipeline_spec, plan_fastq_to_fastq__default__v1, DefaultPipelineOptions,
    FastqPipelineInputs,
};

fn tool_for_stage(stage: &str) -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId::new("planner-dummy"),
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
fn fastq_plan_is_deterministic() -> anyhow::Result<()> {
    let pipeline = default_pipeline_spec(DefaultPipelineOptions::default());
    let tools = pipeline
        .stages
        .iter()
        .map(|stage| tool_for_stage(stage))
        .collect::<Vec<_>>();
    let temp = bijux_infra::temp_dir("fastq-plan-determinism")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let inputs = FastqPipelineInputs {
        policy: PlanPolicy::PreferAccuracy,
        tools,
        aux_images: BTreeMap::new(),
        adapter_bank: None,
        polyx_bank: None,
        contaminant_bank: None,
        enable_contaminant_removal: false,
        r1,
        r2: None,
        out_dir: temp.path().join("out"),
        tool_reasons: None,
    };

    let graph_a = plan_fastq_to_fastq__default__v1(&inputs, DefaultPipelineOptions::default())?;
    let graph_b = plan_fastq_to_fastq__default__v1(&inputs, DefaultPipelineOptions::default())?;
    assert_eq!(graph_a.hash()?, graph_b.hash()?);
    Ok(())
}
