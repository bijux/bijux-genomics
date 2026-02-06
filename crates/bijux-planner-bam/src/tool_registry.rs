use bijux_domain_bam::{stage_spec, BamStage};

#[must_use]
pub fn allowed_tools_for_stage(stage: BamStage) -> Vec<String> {
    stage_spec(stage)
        .allowed_tools
        .iter()
        .map(|tool| (*tool).to_string())
        .collect()
}

#[must_use]
#[allow(dead_code)]
pub fn default_tool_for_stage(stage: BamStage) -> String {
    stage_spec(stage).default_tool.to_string()
}
