use anyhow::{anyhow, Result};
use bijux_dna_core::ids::{StageId, ToolId};

pub(crate) fn enforce_stage_tool(stage_id: &str, tool_id: &ToolId) -> Result<()> {
    let stage_id = StageId::new(stage_id.to_string());
    let allowed_tools = crate::selection::allowed_tools_for_stage(&stage_id);
    if allowed_tools.is_empty() {
        return Err(anyhow!(
            "{} has no admitted execution tools in the FASTQ domain registry",
            stage_id.as_str()
        ));
    }
    if allowed_tools.iter().any(|allowed| allowed == tool_id) {
        return Ok(());
    }
    Err(anyhow!(
        "{} is not admitted for {}; allowed tools: {}",
        tool_id.as_str(),
        stage_id.as_str(),
        allowed_tools.iter().map(|tool| tool.as_str()).collect::<Vec<_>>().join(", ")
    ))
}
