use bijux_core::ids::StageId;
use bijux_domain_fastq::tool_registry::{
    canonical_tools_for_stage, default_tool_for_stage as domain_default_tool_for_stage,
};

#[must_use]
pub fn allowed_tools_for_stage(stage_id: &StageId) -> Vec<String> {
    canonical_tools_for_stage(stage_id)
        .iter()
        .map(|tool| (*tool).to_string())
        .collect()
}

#[must_use]
pub fn default_tool_for_stage(stage_id: &StageId) -> Option<String> {
    domain_default_tool_for_stage(stage_id).map(|tool| tool.to_string())
}
