use std::collections::BTreeMap;
use std::path::Path;

use bijux_dna_core::prelude::{
    ArtifactId, ArtifactRef, ArtifactRole, CommandSpecV1, ContainerImageRefV1, ToolConstraints,
    ToolExecutionSpecV1, ToolId,
};
use bijux_dna_domain_fastq::params::{qc_post::QcAggregationScope, PairedMode};

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
fn report_qc_can_plan_from_governed_qc_artifacts() -> anyhow::Result<()> {
    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::report_qc::plan_qc_post_with_qc_inputs(
        &tool("multiqc"),
        &[
            ArtifactRef::required(
                ArtifactId::from_static("qc_json"),
                Path::new("profile_reads/qc.json").to_path_buf(),
                ArtifactRole::MetricsJson,
            ),
            ArtifactRef::required(
                ArtifactId::from_static("adapter_report"),
                Path::new("detect_adapters/adapter_report.json").to_path_buf(),
                ArtifactRole::ReportJson,
            ),
        ],
        Path::new("out"),
        BTreeMap::new(),
        PairedMode::SingleEnd,
        QcAggregationScope::GovernedQcArtifacts,
        None,
        None,
    )?;

    assert_eq!(plan.io.inputs.len(), 2);
    assert_eq!(
        plan.effective_params["aggregation_scope"],
        serde_json::json!("governed_qc_artifacts")
    );
    assert!(plan
        .command
        .template
        .iter()
        .any(|part| part == "profile_reads"));
    assert!(plan
        .command
        .template
        .iter()
        .any(|part| part == "detect_adapters"));
    Ok(())
}
