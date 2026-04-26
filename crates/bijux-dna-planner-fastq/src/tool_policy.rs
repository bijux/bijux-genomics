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

pub(crate) fn enforce_input_layout(
    stage_id: &str,
    tool_id: &ToolId,
    paired_end: bool,
) -> Result<()> {
    if matches!(stage_id, "fastq.index_reference" | "fastq.normalize_abundance") {
        return Ok(());
    }
    let stage_id = StageId::new(stage_id.to_string());
    if crate::stage_api::tool_supports_input_layout(&stage_id, tool_id, paired_end) {
        return Ok(());
    }
    let layout = if paired_end { "paired-end" } else { "single-end" };
    Err(anyhow!("{} does not support {layout} inputs for {}", tool_id.as_str(), stage_id.as_str()))
}
