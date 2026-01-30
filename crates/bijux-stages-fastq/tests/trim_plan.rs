use anyhow::Result;
use bijux_core::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};

fn dummy_tool(tool: &str) -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId(tool.to_string()),
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
        bijux_stages_fastq::fastq::trim::trim_output_name("fastp"),
        Some("fastp.fastq.gz")
    );
    assert_eq!(
        bijux_stages_fastq::fastq::trim::trim_output_name("trimmomatic"),
        Some("trimmomatic.fastq.gz")
    );
    assert_eq!(
        bijux_stages_fastq::fastq::trim::trim_output_name("unknown"),
        None
    );
}

#[test]
fn plan_trim_builds_expected_paths() -> Result<()> {
    let plan = bijux_stages_fastq::fastq::trim::plan(
        &dummy_tool("fastp"),
        std::path::Path::new("reads.fastq.gz"),
        std::path::Path::new("out"),
        None,
    )?;
    assert_eq!(
        plan.io.outputs[0].path.to_string_lossy(),
        "out/fastp.fastq.gz"
    );
    Ok(())
}

#[test]
fn plan_trim_rejects_unknown_tool() {
    match bijux_stages_fastq::fastq::trim::plan(
        &dummy_tool("mystery"),
        std::path::Path::new("reads.fastq.gz"),
        std::path::Path::new("out"),
        None,
    ) {
        Ok(_) => panic!("expected unsupported trim tool"),
        Err(err) => assert!(err.to_string().contains("unsupported trim tool")),
    }
}
