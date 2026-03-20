use bijux_dna_core::ids::StageId;

#[test]
fn toolset_modes_separate_default_governed_benchmark_and_all_bindings() {
    let trim_stage = StageId::from_static("fastq.trim_reads");

    let default_tools = bijux_dna_planner_fastq::stage_api::toolset_for_stage(
        &trim_stage,
        bijux_dna_planner_fastq::stage_api::ToolsetExecutionMode::DefaultChoice,
    );
    assert_eq!(default_tools.len(), 1);
    assert_eq!(default_tools[0].as_str(), "fastp");

    let governed_tools = bijux_dna_planner_fastq::stage_api::toolset_for_stage(
        &trim_stage,
        bijux_dna_planner_fastq::stage_api::ToolsetExecutionMode::GovernedExecution,
    );
    assert!(governed_tools
        .iter()
        .any(|tool_id| tool_id.as_str() == "fastp"));
    assert!(governed_tools
        .iter()
        .all(|tool_id| tool_id.as_str() != "seqpurge"));

    let benchmark_tools = bijux_dna_planner_fastq::stage_api::toolset_for_stage(
        &trim_stage,
        bijux_dna_planner_fastq::stage_api::ToolsetExecutionMode::BenchmarkCohort,
    );
    assert!(benchmark_tools.is_empty());

    let all_bindings = bijux_dna_planner_fastq::stage_api::toolset_for_stage(
        &trim_stage,
        bijux_dna_planner_fastq::stage_api::ToolsetExecutionMode::AllBindings,
    );
    assert!(all_bindings
        .iter()
        .any(|tool_id| tool_id.as_str() == "seqpurge"));
}

#[test]
fn toolset_modes_keep_declared_only_stages_honest() {
    let infer_stage = StageId::from_static("fastq.infer_asvs");

    let governed_tools = bijux_dna_planner_fastq::stage_api::toolset_for_stage(
        &infer_stage,
        bijux_dna_planner_fastq::stage_api::ToolsetExecutionMode::GovernedExecution,
    );
    assert!(governed_tools.is_empty());

    let benchmark_tools = bijux_dna_planner_fastq::stage_api::toolset_for_stage(
        &infer_stage,
        bijux_dna_planner_fastq::stage_api::ToolsetExecutionMode::BenchmarkCohort,
    );
    assert!(benchmark_tools.is_empty());

    let all_bindings = bijux_dna_planner_fastq::stage_api::toolset_for_stage(
        &infer_stage,
        bijux_dna_planner_fastq::stage_api::ToolsetExecutionMode::AllBindings,
    );
    assert_eq!(all_bindings.len(), 1);
    assert_eq!(all_bindings[0].as_str(), "dada2");
}
