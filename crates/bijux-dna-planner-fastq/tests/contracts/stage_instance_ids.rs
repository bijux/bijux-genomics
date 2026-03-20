use bijux_dna_core::prelude::{CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId};

fn tool(tool_id: &str) -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId::new(tool_id.to_string()),
        tool_version: "99.99.99+fixture".to_string(),
        image: ContainerImageRefV1 {
            image: "bijux/dummy:latest".to_string(),
            digest: None,
        },
        command: CommandSpecV1 {
            template: vec!["echo".to_string(), tool_id.to_string()],
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
fn preprocess_stage_plans_emit_tool_scoped_stage_instance_ids() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-stage-instance-ids")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let validate_plan =
        bijux_dna_planner_fastq::tool_adapters::fastq::validate_reads::plan(
            &tool("fastqvalidator"),
            &r1,
            None,
            temp.path(),
        )?;
    assert_eq!(
        validate_plan
            .stage_instance_id
            .as_ref()
            .map(|step_id| step_id.as_str()),
        Some("fastq.validate_reads.tool.fastqvalidator")
    );

    let detect_plan =
        bijux_dna_planner_fastq::tool_adapters::fastq::detect_adapters::plan(
            &tool("fastqc"),
            &r1,
            None,
            temp.path(),
        )?;
    assert_eq!(
        detect_plan
            .stage_instance_id
            .as_ref()
            .map(|step_id| step_id.as_str()),
        Some("fastq.detect_adapters.tool.fastqc")
    );

    Ok(())
}

#[test]
fn transform_stage_plans_emit_tool_scoped_stage_instance_ids() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-transform-stage-instance-ids")?;
    let r1 = temp.path().join("reads_R1.fastq");
    let r2 = temp.path().join("reads_R2.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;
    std::fs::write(&r2, b"@r2\nT\n+\n#\n")?;

    let trim_plan = bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::plan(
        &tool("fastp"),
        &r1,
        Some(&r2),
        temp.path(),
        None,
        None,
        None,
    )?;
    assert_eq!(
        trim_plan
            .stage_instance_id
            .as_ref()
            .map(|step_id| step_id.as_str()),
        Some("fastq.trim_reads.tool.fastp")
    );

    let merge_plan = bijux_dna_planner_fastq::tool_adapters::fastq::merge_pairs::plan_merge(
        &tool("pear"),
        &r1,
        &r2,
        temp.path(),
    )?;
    assert_eq!(
        merge_plan
            .stage_instance_id
            .as_ref()
            .map(|step_id| step_id.as_str()),
        Some("fastq.merge_pairs.tool.pear")
    );

    Ok(())
}
