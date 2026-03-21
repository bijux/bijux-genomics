use bijux_dna_core::contract::{PipelineEdgeSpec, PipelineNodeSpec, PipelineSpec};

#[test]
fn toolset_selection_uses_execution_modes_for_governed_and_benchmark_paths() -> anyhow::Result<()> {
    let pipeline = PipelineSpec::linear(vec!["fastq.trim_reads".to_string()]);

    let default = bijux_dna_planner_fastq::select_preprocess_toolsets(
        &pipeline,
        bijux_dna_planner_fastq::stage_api::ToolsetExecutionMode::DefaultChoice,
        false,
    )?;
    assert_eq!(default.len(), 1);
    assert_eq!(default[0].stage_id, "fastq.trim_reads");
    assert_eq!(default[0].tool_ids, vec!["fastp".to_string()]);

    let governed = bijux_dna_planner_fastq::select_preprocess_toolsets(
        &pipeline,
        bijux_dna_planner_fastq::stage_api::ToolsetExecutionMode::GovernedExecution,
        false,
    )?;
    assert!(governed[0]
        .tool_ids
        .iter()
        .any(|tool_id| tool_id == "fastp"));
    assert!(governed[0]
        .tool_ids
        .iter()
        .all(|tool_id| tool_id != "seqpurge"));

    let benchmark = bijux_dna_planner_fastq::select_preprocess_toolsets(
        &pipeline,
        bijux_dna_planner_fastq::stage_api::ToolsetExecutionMode::BenchmarkCohort,
        false,
    )?;
    assert!(benchmark[0].tool_ids.is_empty());

    Ok(())
}

#[test]
fn toolset_selection_keeps_declared_bindings_and_declared_only_stages_explicit(
) -> anyhow::Result<()> {
    let trim_pipeline = PipelineSpec::linear(vec!["fastq.trim_reads".to_string()]);
    let all_bindings = bijux_dna_planner_fastq::select_preprocess_toolsets(
        &trim_pipeline,
        bijux_dna_planner_fastq::stage_api::ToolsetExecutionMode::AllBindings,
        false,
    )?;
    assert!(all_bindings[0]
        .tool_ids
        .iter()
        .any(|tool_id| tool_id == "seqpurge"));

    let infer_pipeline = PipelineSpec::linear(vec!["fastq.infer_asvs".to_string()]);
    let declared_only_error = bijux_dna_planner_fastq::select_preprocess_toolsets(
        &infer_pipeline,
        bijux_dna_planner_fastq::stage_api::ToolsetExecutionMode::AllBindings,
        false,
    )
    .expect_err("declared-only stages must still require explicit override");
    assert!(declared_only_error
        .to_string()
        .contains("not active in current scope"));

    let declared_only = bijux_dna_planner_fastq::select_preprocess_toolsets(
        &infer_pipeline,
        bijux_dna_planner_fastq::stage_api::ToolsetExecutionMode::AllBindings,
        true,
    )?;
    assert_eq!(declared_only.len(), 1);
    assert_eq!(declared_only[0].stage_id, "fastq.infer_asvs");
    assert_eq!(declared_only[0].tool_ids, vec!["dada2".to_string()]);

    Ok(())
}

#[test]
fn toolset_selection_skips_planner_owned_select_nodes() -> anyhow::Result<()> {
    let pipeline = PipelineSpec::graph(
        vec![
            PipelineNodeSpec {
                stage_id: "fastq.trim_reads".to_string(),
                stage_instance_id: Some("fastq.trim_reads.cleanup".to_string()),
            },
            PipelineNodeSpec {
                stage_id: "benchmark.select_stage_tool".to_string(),
                stage_instance_id: Some("benchmark.select_stage_tool.trim_reads".to_string()),
            },
            PipelineNodeSpec {
                stage_id: "fastq.filter_reads".to_string(),
                stage_instance_id: Some("fastq.filter_reads.selected".to_string()),
            },
        ],
        vec![
            PipelineEdgeSpec {
                from: "fastq.trim_reads.cleanup".to_string(),
                to: "benchmark.select_stage_tool.trim_reads".to_string(),
                from_output_id: Some("trimmed_reads_r1".to_string()),
                to_input_id: Some("candidate_trimmed_reads_r1".to_string()),
            },
            PipelineEdgeSpec {
                from: "benchmark.select_stage_tool.trim_reads".to_string(),
                to: "fastq.filter_reads.selected".to_string(),
                from_output_id: Some("trimmed_reads_r1".to_string()),
                to_input_id: Some("reads_r1".to_string()),
            },
        ],
    );

    let toolsets = bijux_dna_planner_fastq::select_preprocess_toolsets(
        &pipeline,
        bijux_dna_planner_fastq::stage_api::ToolsetExecutionMode::GovernedExecution,
        false,
    )?;
    assert_eq!(toolsets.len(), 2);
    assert_eq!(toolsets[0].stage_id, "fastq.trim_reads");
    assert_eq!(toolsets[1].stage_id, "fastq.filter_reads");

    Ok(())
}
