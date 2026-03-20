use anyhow::Result;
use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};

fn dummy_tool(tool: &str) -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId::new(tool),
        tool_version: "1.0.0".to_string(),
        image: ContainerImageRefV1 {
            image: "bijux/test:latest".to_string(),
            digest: None,
        },
        command: CommandSpecV1 {
            template: Vec::new(),
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
fn trim_output_names_are_defined_for_known_tools() {
    assert_eq!(
        bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::trim_output_name("fastp"),
        Some("fastp.fastq.gz")
    );
    assert_eq!(
        bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::trim_output_name("trimmomatic"),
        Some("trimmomatic.fastq.gz")
    );
    assert_eq!(
        bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::trim_output_name("unknown"),
        None
    );
    assert_eq!(
        bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::trim_output_name("seqpurge"),
        None
    );
}

#[test]
fn plan_trim_builds_expected_paths() -> Result<()> {
    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::plan(
        &dummy_tool("fastp"),
        std::path::Path::new("reads.fastq.gz"),
        None,
        std::path::Path::new("out"),
        None,
        None,
        None,
    )?;
    assert_eq!(
        plan.io.outputs[0].path.to_string_lossy(),
        "out/fastp.fastq.gz"
    );
    assert_eq!(plan.io.outputs[0].name.as_str(), "trimmed_reads_r1");
    assert_eq!(plan.io.outputs[1].name.as_str(), "report_json");
    Ok(())
}

#[test]
fn plan_trim_rejects_unknown_tool() {
    match bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::plan(
        &dummy_tool("mystery"),
        std::path::Path::new("reads.fastq.gz"),
        None,
        std::path::Path::new("out"),
        None,
        None,
        None,
    ) {
        Ok(_) => panic!("expected unsupported trim tool"),
        Err(err) => assert!(err.to_string().contains("unsupported trim tool")),
    }
}

#[test]
fn plan_from_config_preserves_layout_and_bank_policies() -> Result<()> {
    let config = bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::resolve_config(
        bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::TrimUserConfig {
            tool: "fastp".to_string(),
            r1: std::path::PathBuf::from("reads_R1.fastq.gz"),
            r2: Some(std::path::PathBuf::from("reads_R2.fastq.gz")),
            out_dir: std::path::PathBuf::from("out"),
            adapter_bank: Some(serde_json::json!({"preset": "illumina"})),
            polyx_bank: Some(serde_json::json!({"enabled": true})),
            contaminant_bank: Some(serde_json::json!({"catalog": "decoys"})),
        },
    );

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::trim_reads::plan_from_config(
        &dummy_tool("fastp"),
        &config,
    )?;

    assert_eq!(plan.io.inputs.len(), 2);
    assert_eq!(plan.io.outputs[1].name.as_str(), "trimmed_reads_r2");
    assert_eq!(plan.io.outputs[2].name.as_str(), "report_json");
    assert_eq!(plan.params["adapter_bank"]["preset"], "illumina");
    assert_eq!(plan.params["polyx_bank"]["enabled"], true);
    assert_eq!(plan.params["contaminant_bank"]["catalog"], "decoys");
    assert_eq!(plan.effective_params["paired_mode"], "paired_end");
    assert_eq!(plan.effective_params["adapter_policy"], "bank");
    assert_eq!(plan.effective_params["polyx_policy"], "bank");
    assert_eq!(plan.effective_params["contaminant_policy"], "bank");
    assert!(plan.command.template.iter().any(|part| part == "--in2"));
    assert!(plan.command.template.iter().any(|part| part == "--out2"));
    assert!(plan
        .command
        .template
        .iter()
        .any(|part| part == "--detect_adapter_for_pe"));
    Ok(())
}

#[test]
fn plan_trim_polyg_preserves_paired_output_names() -> Result<()> {
    let plan =
        bijux_dna_planner_fastq::tool_adapters::fastq::trim_polyg_tails::plan_trim_polyg_tails(
            &dummy_tool("fastp"),
            std::path::Path::new("reads_R1.fastq.gz"),
            Some(std::path::Path::new("reads_R2.fastq.gz")),
            std::path::Path::new("out"),
        )?;

    assert_eq!(plan.io.outputs[0].name.as_str(), "trimmed_reads_r1");
    assert_eq!(plan.io.outputs[1].name.as_str(), "trimmed_reads_r2");
    assert_eq!(plan.io.outputs[2].name.as_str(), "report_json");
    assert!(plan.command.template.iter().any(|part| part == "--in2"));
    assert!(plan.command.template.iter().any(|part| part == "--out2"));
    assert_eq!(
        plan.effective_params["schema_version"],
        "bijux.fastq.params.trim_polyg_tails.v1"
    );
    assert_eq!(plan.effective_params["trim_polyg"], true);
    assert_eq!(plan.effective_params["min_polyg_run"], 10);
    Ok(())
}

#[test]
fn plan_trim_terminal_damage_preserves_paired_output_names() -> Result<()> {
    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::trim_terminal_damage::plan_trim_terminal_damage(
        &dummy_tool("cutadapt"),
        std::path::Path::new("reads_R1.fastq.gz"),
        Some(std::path::Path::new("reads_R2.fastq.gz")),
        std::path::Path::new("out"),
        "ancient",
        2,
        2,
    )?;

    assert_eq!(plan.io.outputs[0].name.as_str(), "trimmed_reads_r1");
    assert_eq!(plan.io.outputs[1].name.as_str(), "trimmed_reads_r2");
    assert_eq!(plan.io.outputs[2].name.as_str(), "report_json");
    assert!(plan.command.template.iter().any(|part| part == "-p"));
    assert_eq!(
        plan.effective_params["schema_version"],
        "bijux.fastq.params.trim_terminal_damage.v1"
    );
    assert_eq!(plan.effective_params["damage_mode"], "ancient");
    assert_eq!(plan.effective_params["trim_5p_bases"], 2);
    assert_eq!(plan.effective_params["trim_3p_bases"], 2);
    Ok(())
}
