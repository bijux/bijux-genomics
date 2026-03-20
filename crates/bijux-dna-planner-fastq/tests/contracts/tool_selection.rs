use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1,
};

#[test]
fn planner_stage_selection_comes_from_domain_execution_support() {
    let trim_stage = StageId::from_static("fastq.trim_reads");
    let trim_tools = bijux_dna_planner_fastq::stage_api::allowed_tools_for_stage(&trim_stage);
    assert!(
        trim_tools.iter().any(|tool| tool.as_str() == "prinseq"),
        "planner trim tool selection must come from the domain execution support manifest",
    );
    assert!(
        !trim_tools.iter().any(|tool| tool.as_str() == "seqpurge"),
        "planner must not admit tools that are absent from the domain execution support manifest",
    );

    let infer_asvs_stage = StageId::from_static("fastq.infer_asvs");
    assert!(
        bijux_dna_planner_fastq::stage_api::allowed_tools_for_stage(&infer_asvs_stage).is_empty(),
        "declared-only stages must not admit execution tools",
    );
    assert!(
        bijux_dna_planner_fastq::stage_api::default_tool_for_stage(&infer_asvs_stage).is_none(),
        "declared-only stages must not expose default execution tools",
    );
}

#[test]
fn correct_errors_planning_rejects_tools_outside_execution_support() {
    let tool = ToolExecutionSpecV1 {
        tool_id: ToolId::new("musket"),
        tool_version: "99.99.99+fixture".to_string(),
        image: ContainerImageRefV1 {
            image: "bijux/test:latest".to_string(),
            digest: None,
        },
        command: CommandSpecV1 {
            template: vec!["musket".to_string()],
        },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
    };

    assert!(
        bijux_dna_planner_fastq::tool_adapters::fastq::correct_errors::plan_correct(
            &tool,
            std::path::Path::new("reads_R1.fastq.gz"),
            std::path::Path::new("reads_R2.fastq.gz"),
            std::path::Path::new("out"),
        )
        .is_err(),
        "planner must reject correction tools that are not closed in domain execution support",
    );
}
