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
        tool_version: "99.99.99+fixture".to_string(),
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
    let stages = pipeline.ordered_stage_ids();
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
        None,
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

#[test]
fn reference_guided_plan_validates_index_to_depletion_flow() -> anyhow::Result<()> {
    let stages = vec![
        "fastq.index_reference".to_string(),
        "fastq.deplete_host".to_string(),
    ];
    let tools = stages
        .iter()
        .map(|stage| tool_for_stage(stage))
        .collect::<Vec<_>>();
    let temp = bijux_dna_infra::temp_dir("fastq-plan-reference-handshake")?;
    let r1 = temp.path().join("reads_R1.fastq");
    let reference = temp.path().join("reference.fa");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;
    std::fs::write(&reference, b">chr1\nA\n")?;

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
        Some(&reference),
        None,
        |stage_id, tool, _r1, _r2| Ok(temp.path().join(stage_id).join(tool.tool_id.as_str())),
    )?;

    assert_eq!(plans[0].stage_id.as_str(), "fastq.index_reference");
    assert_eq!(plans[1].stage_id.as_str(), "fastq.deplete_host");
    assert_eq!(plans[1].io.inputs[1].path, plans[0].io.outputs[0].path);
    Ok(())
}

#[test]
fn compose_fastq_pipeline_steps_rejects_mismatched_stage_and_tool_counts() {
    let error = compose_fastq_pipeline_steps(
        &[
            "fastq.validate_reads".to_string(),
            "fastq.trim_reads".to_string(),
        ],
        &[tool_for_stage("fastq.validate_reads")],
        &BTreeMap::new(),
        None,
        None,
        None,
        None,
        false,
        std::path::Path::new("reads_R1.fastq"),
        None,
        None,
        None,
        |stage_id, tool, _r1, _r2| {
            Ok(std::path::PathBuf::from(stage_id).join(tool.tool_id.as_str()))
        },
    )
    .expect_err("mismatched stage/tool lists must fail loudly");

    assert!(error
        .to_string()
        .contains("matching stage/tool lengths"));
}

#[test]
fn reference_guided_plan_rejects_incompatible_index_backend() -> anyhow::Result<()> {
    let stages = vec![
        "fastq.index_reference".to_string(),
        "fastq.deplete_host".to_string(),
    ];
    let tools = vec![
        ToolExecutionSpecV1 {
            tool_id: ToolId::new("star"),
            tool_version: "99.99.99+fixture".to_string(),
            image: ContainerImageRefV1 {
                image: "bijux/dummy:latest".to_string(),
                digest: None,
            },
            command: CommandSpecV1 {
                template: vec!["echo".to_string(), "fastq.index_reference".to_string()],
            },
            resources: ToolConstraints {
                runtime: "docker".to_string(),
                mem_gb: 1,
                tmp_gb: 1,
                threads: 1,
            },
        },
        tool_for_stage("fastq.deplete_host"),
    ];
    let temp = bijux_dna_infra::temp_dir("fastq-plan-reference-mismatch")?;
    let r1 = temp.path().join("reads_R1.fastq");
    let reference = temp.path().join("reference.fa");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;
    std::fs::write(&reference, b">chr1\nA\n")?;

    let error = compose_fastq_pipeline_steps(
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
        Some(&reference),
        None,
        |stage_id, tool, _r1, _r2| Ok(temp.path().join(stage_id).join(tool.tool_id.as_str())),
    )
    .expect_err("STAR index must not satisfy bowtie2 depletion");

    let message = error.to_string();
    assert!(
        message.contains("requires one of [bowtie2_build]"),
        "planner must explain the governed compatible index backends: {message}"
    );
    Ok(())
}
