use crate::internal::fastq::stages::preprocess::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PreprocessSelectionMode {
    RunAllGovernedTools,
    DefaultChoice,
    AutoSelect,
}

pub(super) fn preprocess_selection_mode(
    args: &bijux_dna_planner_fastq::stage_api::args::BenchFastqPreprocessArgs,
) -> PreprocessSelectionMode {
    if args.run_all_governed_tools {
        PreprocessSelectionMode::RunAllGovernedTools
    } else if args.auto {
        PreprocessSelectionMode::AutoSelect
    } else {
        PreprocessSelectionMode::DefaultChoice
    }
}

pub(super) fn report_qc_aux_tool_ids(
    pipeline: &bijux_dna_core::contract::PipelineSpec,
    selected_stage_tools: &[StageToolSelection],
) -> Vec<String> {
    let contributor_node_ids = pipeline
        .ordered_nodes()
        .into_iter()
        .filter(|node| node.stage_id == STAGE_REPORT_QC.as_str())
        .flat_map(|report_node| {
            let report_node_id = bijux_dna_core::contract::PipelineSpec::stage_node_id(
                &report_node.stage_id,
                report_node.stage_instance_id.as_deref(),
            );
            pipeline
                .edges
                .iter()
                .filter(move |edge| edge.to == report_node_id)
                .map(|edge| edge.from.clone())
                .collect::<Vec<_>>()
        })
        .collect::<std::collections::BTreeSet<_>>();
    let mut tool_ids = selected_stage_tools
        .iter()
        .filter_map(|selection| {
            let node_id = bijux_dna_core::contract::PipelineSpec::stage_node_id(
                &selection.stage_id,
                selection.stage_instance_id.as_deref(),
            );
            contributor_node_ids.contains(node_id.as_str()).then(|| selection.tool_id.clone())
        })
        .collect::<Vec<_>>();
    tool_ids.sort();
    tool_ids.dedup();
    tool_ids
}

pub(super) fn planner_selection_surfaces(
    selected_stage_tools: &[StageToolSelection],
    tool_specs: &[bijux_dna_core::prelude::ToolExecutionSpecV1],
    planner_stage_toolsets: Vec<bijux_dna_planner_fastq::FastqStageToolsetBinding>,
) -> Vec<bijux_dna_planner_fastq::FastqStageToolsetBinding> {
    if !planner_stage_toolsets.is_empty() {
        return planner_stage_toolsets;
    }
    assert_eq!(
        selected_stage_tools.len(),
        tool_specs.len(),
        "selected preprocess stage tools and tool specs must stay aligned"
    );

    selected_stage_tools
        .iter()
        .zip(tool_specs.iter())
        .map(|(selection, tool)| bijux_dna_planner_fastq::FastqStageToolsetBinding {
            stage_id: selection.stage_id.clone(),
            stage_instance_id: selection.stage_instance_id.clone(),
            tools: vec![tool.clone()],
            reason: Some(selection.reason.clone()),
            params: None,
        })
        .collect::<Vec<_>>()
}
