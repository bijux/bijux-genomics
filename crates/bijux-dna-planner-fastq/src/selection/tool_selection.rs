use bijux_dna_core::ids::{StageId, ToolId};

#[must_use]
pub fn allowed_tools_for_stage(stage_id: &StageId) -> Vec<ToolId> {
    let mut tools = bijux_dna_domain_fastq::admitted_execution_tools_for_stage(stage_id);
    tools = tools.into_iter().map(|tool| ToolId::new(tool.as_str().to_string())).collect();
    tools.sort_by(|a, b| a.as_str().cmp(b.as_str()));
    tools
}

#[must_use]
pub fn default_tool_for_stage(stage_id: &StageId) -> Option<ToolId> {
    bijux_dna_domain_fastq::default_execution_tool_for_stage(stage_id)
        .map(|tool| ToolId::new(tool.as_str().to_string()))
}
