use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, StepId, ToolConstraints, ToolExecutionSpecV1, ToolId,
};

fn tool(tool_id: &str) -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId::new(tool_id.to_string()),
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
fn preprocess_stage_plans_emit_tool_scoped_stage_instance_ids() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-stage-instance-ids")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let validate_plan = bijux_dna_planner_fastq::tool_adapters::fastq::validate_reads::plan(
        &tool("fastqvalidator"),
        &r1,
        None,
        temp.path(),
    )?;
    assert_eq!(
        validate_plan.stage_instance_id.as_ref().map(StepId::as_str),
        Some("fastq.validate_reads.tool.fastqvalidator")
    );

    let detect_plan = bijux_dna_planner_fastq::tool_adapters::fastq::detect_adapters::plan(
        &tool("fastqc"),
        &r1,
        None,
        temp.path(),
    )?;
    assert_eq!(
        detect_plan.stage_instance_id.as_ref().map(StepId::as_str),
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
        trim_plan.stage_instance_id.as_ref().map(StepId::as_str),
        Some("fastq.trim_reads.tool.fastp")
    );

    let merge_plan = bijux_dna_planner_fastq::tool_adapters::fastq::merge_pairs::plan_merge(
        &tool("pear"),
        &r1,
        &r2,
        temp.path(),
    )?;
    assert_eq!(
        merge_plan.stage_instance_id.as_ref().map(StepId::as_str),
        Some("fastq.merge_pairs.tool.pear")
    );

    Ok(())
}

#[test]
fn qc_stage_plans_emit_tool_scoped_stage_instance_ids() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-qc-stage-instance-ids")?;
    let r1 = temp.path().join("reads_R1.fastq");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;

    let profile_plan =
        bijux_dna_planner_fastq::tool_adapters::fastq::profile_reads::plan_stats_neutral(
            &tool("seqkit_stats"),
            &r1,
            None,
            temp.path(),
        )?;
    assert_eq!(
        profile_plan.stage_instance_id.as_ref().map(StepId::as_str),
        Some("fastq.profile_reads.tool.seqkit_stats")
    );

    let report_plan =
        bijux_dna_planner_fastq::tool_adapters::fastq::report_qc::plan_qc_post_with_qc_inputs(
            &tool("multiqc"),
            &[bijux_dna_core::prelude::ArtifactRef::required(
                bijux_dna_core::prelude::ArtifactId::from_static(
                    "fastq.profile_reads.tool.seqkit_stats.qc_json",
                ),
                temp.path().join("profile_reads/qc.json"),
                bijux_dna_core::prelude::ArtifactRole::MetricsJson,
            )],
            temp.path(),
            std::collections::BTreeMap::new(),
            bijux_dna_domain_fastq::params::PairedMode::SingleEnd,
            bijux_dna_domain_fastq::params::qc_post::QcAggregationEngine::Multiqc,
            bijux_dna_domain_fastq::params::qc_post::QcAggregationScope::GovernedQcArtifacts,
            Some(&r1),
            None,
        )?;
    assert_eq!(
        report_plan.stage_instance_id.as_ref().map(StepId::as_str),
        Some("fastq.report_qc.tool.multiqc")
    );

    Ok(())
}

#[test]
fn reference_aware_stage_plans_emit_tool_scoped_stage_instance_ids() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-reference-stage-instance-ids")?;
    let r1 = temp.path().join("reads_R1.fastq");
    let index = temp.path().join("host_index");
    std::fs::write(&r1, b"@r1\nA\n+\n#\n")?;
    std::fs::create_dir_all(&index)?;

    let host_plan =
        bijux_dna_planner_fastq::tool_adapters::fastq::deplete_host::plan_host_depletion(
            &tool("bowtie2"),
            &r1,
            None,
            &index,
            temp.path(),
        )?;
    assert_eq!(
        host_plan.stage_instance_id.as_ref().map(StepId::as_str),
        Some("fastq.deplete_host.tool.bowtie2")
    );

    let contaminant_plan = bijux_dna_planner_fastq::tool_adapters::fastq::deplete_reference_contaminants::plan_contaminant_screen(
        &tool("bowtie2"),
        &r1,
        None,
        &index,
        temp.path(),
    )?;
    assert_eq!(
        contaminant_plan.stage_instance_id.as_ref().map(StepId::as_str),
        Some("fastq.deplete_reference_contaminants.tool.bowtie2")
    );

    Ok(())
}

#[test]
fn amplicon_stage_plans_emit_tool_scoped_stage_instance_ids() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("fastq-amplicon-stage-instance-ids")?;
    let reads = temp.path().join("merged.fastq");
    std::fs::write(&reads, b"@r1\nA\n+\n#\n")?;

    let otu_plan = bijux_dna_planner_fastq::tool_adapters::fastq::cluster_otus::plan(
        &tool("vsearch"),
        &reads,
        None,
        temp.path(),
    )?;
    assert_eq!(
        otu_plan.stage_instance_id.as_ref().map(StepId::as_str),
        Some("fastq.cluster_otus.tool.vsearch")
    );

    let abundance_input = temp.path().join("otu_abundance.tsv");
    std::fs::write(&abundance_input, b"otu\tcount\nOTU_1\t1\n")?;
    let abundance_plan = bijux_dna_planner_fastq::tool_adapters::fastq::normalize_abundance::plan(
        &tool("seqkit"),
        &abundance_input,
        temp.path(),
    )?;
    assert_eq!(
        abundance_plan.stage_instance_id.as_ref().map(StepId::as_str),
        Some("fastq.normalize_abundance.tool.seqkit")
    );

    Ok(())
}
