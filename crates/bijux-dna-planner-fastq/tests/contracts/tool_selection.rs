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
    let infer_asvs_error = bijux_dna_planner_fastq::select_infer_asvs_tools(&["dada2".to_string()])
        .expect_err("declared-only stage selection must fail before pretending to admit tools");
    assert!(
        infer_asvs_error.to_string().contains("declared-only"),
        "declared-only stage selection must explain the governed runtime boundary",
    );
}

#[test]
fn correct_errors_planning_accepts_closed_backends() {
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

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::correct_errors::plan_correct(
        &tool,
        std::path::Path::new("reads_R1.fastq.gz"),
        Some(std::path::Path::new("reads_R2.fastq.gz")),
        std::path::Path::new("out"),
    )
    .expect("planner must accept correction tools that are closed in domain execution support");
    assert_eq!(plan.tool_id.as_str(), "musket");
    assert_eq!(plan.command.template[0], "sh");
    assert_eq!(plan.command.template[1], "-lc");
    let script = &plan.command.template[2];
    assert!(script.contains("musket -p"));
    assert!(script.contains("correct_report.json"));
    assert!(script.contains("\"tool_id\":\"musket\""));
    assert!(script.contains("\"correction_engine\":\"musket\""));
}

#[test]
fn correct_errors_planning_accepts_single_end_inputs() {
    let tool = ToolExecutionSpecV1 {
        tool_id: ToolId::new("rcorrector"),
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

    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::correct_errors::plan_correct(
        &tool,
        std::path::Path::new("reads_R1.fastq.gz"),
        None,
        std::path::Path::new("out"),
    )
    .expect("single-end correction planning must be admitted");
    assert_eq!(plan.io.inputs.len(), 1);
    assert_eq!(plan.io.outputs.len(), 2);
    assert_eq!(plan.effective_params["paired_mode"], "single_end");
}

#[test]
fn correct_errors_planning_rejects_tools_outside_execution_support() {
    let tool = ToolExecutionSpecV1 {
        tool_id: ToolId::new("seqpurge"),
        tool_version: "99.99.99+fixture".to_string(),
        image: ContainerImageRefV1 {
            image: "bijux/test:latest".to_string(),
            digest: None,
        },
        command: CommandSpecV1 {
            template: vec!["seqpurge".to_string()],
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
            Some(std::path::Path::new("reads_R2.fastq.gz")),
            std::path::Path::new("out"),
        )
        .is_err(),
        "planner must reject correction tools that are not closed in domain execution support",
    );
}

#[test]
fn report_qc_aux_tools_come_from_observer_contributors() {
    let aux_tools = bijux_dna_planner_fastq::stage_api::fastq::report_qc::aux_tool_ids();
    assert!(aux_tools.iter().any(|tool| tool == "fastqc"));
    assert!(aux_tools.iter().any(|tool| tool == "seqkit_stats"));
    assert!(aux_tools.iter().any(|tool| tool == "fastqvalidator"));
    assert!(
        !aux_tools.iter().any(|tool| tool == "multiqc"),
        "report_qc aux tools must describe upstream QC contributors, not the aggregation tool itself",
    );
}
