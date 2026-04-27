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
    assert!(governed_tools.iter().any(|tool_id| tool_id.as_str() == "fastp"));
    assert!(!governed_tools.iter().any(|tool_id| tool_id.as_str() == "seqpurge"));

    let benchmark_tools = bijux_dna_planner_fastq::stage_api::toolset_for_stage(
        &trim_stage,
        bijux_dna_planner_fastq::stage_api::ToolsetExecutionMode::BenchmarkCohort,
    );
    assert!(!benchmark_tools.is_empty());
    assert!(benchmark_tools.iter().all(|tool_id| governed_tools.contains(tool_id)));

    let all_bindings = bijux_dna_planner_fastq::stage_api::toolset_for_stage(
        &trim_stage,
        bijux_dna_planner_fastq::stage_api::ToolsetExecutionMode::AllBindings,
    );
    assert!(governed_tools.iter().all(|tool_id| all_bindings.contains(tool_id)));
    assert!(all_bindings.iter().any(|tool_id| tool_id.as_str() == "seqpurge"));
}

#[test]
fn benchmark_toolsets_can_be_requested_per_fairness_scenario() {
    let dedup_stage = StageId::from_static("fastq.remove_duplicates");

    let default_benchmark_tools = bijux_dna_planner_fastq::stage_api::toolset_for_stage(
        &dedup_stage,
        bijux_dna_planner_fastq::stage_api::ToolsetExecutionMode::BenchmarkCohort,
    );
    assert_eq!(
        default_benchmark_tools.iter().map(bijux_dna_core::ids::ToolId::as_str).collect::<Vec<_>>(),
        vec!["clumpify", "fastuniq"]
    );

    let dedup_tools = bijux_dna_planner_fastq::stage_api::toolset_for_stage_benchmark_scenario(
        &dedup_stage,
        "dedup_fairness",
    );
    assert_eq!(
        dedup_tools.iter().map(bijux_dna_core::ids::ToolId::as_str).collect::<Vec<_>>(),
        vec!["clumpify", "fastuniq"]
    );

    let unknown_tools = bijux_dna_planner_fastq::stage_api::toolset_for_stage_benchmark_scenario(
        &dedup_stage,
        "unknown_fairness",
    );
    assert!(unknown_tools.is_empty());
}

#[test]
fn toolset_modes_publish_governed_infer_asvs_runtime_tools() {
    let infer_stage = StageId::from_static("fastq.infer_asvs");

    let governed_tools = bijux_dna_planner_fastq::stage_api::toolset_for_stage(
        &infer_stage,
        bijux_dna_planner_fastq::stage_api::ToolsetExecutionMode::GovernedExecution,
    );
    assert_eq!(
        governed_tools.iter().map(bijux_dna_core::ids::ToolId::as_str).collect::<Vec<_>>(),
        vec!["dada2"]
    );

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
