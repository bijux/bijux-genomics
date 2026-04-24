use anyhow::Result;
use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};

fn dummy_tool(tool: &str) -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId::new(tool),
        tool_version: "1.0.0".to_string(),
        image: ContainerImageRefV1 { image: "bijux/test:latest".to_string(), digest: None },
        command: CommandSpecV1 { template: Vec::new() },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 3,
        },
    }
}

#[test]
fn fastp_filter_plan_preserves_paired_io() -> Result<()> {
    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::filter_reads::plan_filter(
        &dummy_tool("fastp"),
        std::path::Path::new("reads_R1.fastq.gz"),
        Some(std::path::Path::new("reads_R2.fastq.gz")),
        std::path::Path::new("out"),
        &bijux_dna_planner_fastq::tool_adapters::fastq::filter_reads::FilterPlanOptions::default(),
    )?;

    assert_eq!(plan.io.inputs.len(), 2);
    assert_eq!(plan.io.outputs.len(), 3);
    assert!(plan.command.template.iter().any(|part| part == "--in2"));
    assert!(plan.command.template.iter().any(|part| part == "--out2"));
    assert!(plan.command.template.iter().any(|part| part == "--json"));
    assert!(plan.command.template.iter().any(|part| part == "reads_R2.fastq.gz"));
    assert_eq!(
        plan.io
            .outputs
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.clone()),
        Some(std::path::PathBuf::from("out/filter_report.json"))
    );
    Ok(())
}

#[test]
fn fastp_filter_plan_rejects_kmer_reference_mode() {
    let result = bijux_dna_planner_fastq::tool_adapters::fastq::filter_reads::plan_filter(
        &dummy_tool("fastp"),
        std::path::Path::new("reads.fastq.gz"),
        None,
        std::path::Path::new("out"),
        &bijux_dna_planner_fastq::tool_adapters::fastq::filter_reads::FilterPlanOptions {
            kmer_ref: Some(std::path::PathBuf::from("contaminants.fasta")),
            ..Default::default()
        },
    );

    match result {
        Ok(_) => panic!("expected fastp k-mer filter planning to be rejected"),
        Err(error) => assert!(error.to_string().contains("contaminant k-mer reference filtering")),
    }
}

#[test]
fn filter_plan_records_backend_report_contract_for_fastp() -> Result<()> {
    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::filter_reads::plan_filter(
        &dummy_tool("fastp"),
        std::path::Path::new("reads.fastq.gz"),
        None,
        std::path::Path::new("out"),
        &bijux_dna_planner_fastq::tool_adapters::fastq::filter_reads::FilterPlanOptions::default(),
    )?;

    assert_eq!(plan.params["report_json"], serde_json::json!("out/filter_report.json"));
    assert_eq!(plan.params["raw_backend_report"], serde_json::json!("out/fastp.filter.json"));
    assert_eq!(plan.params["raw_backend_report_format"], serde_json::json!("fastp_json"));
    Ok(())
}
