use bijux_dna_core::ids::validate_tool_id;
use bijux_dna_core::ids::StageId;
use bijux_dna_core::ids::ToolId;

use crate::EffectiveDefaults;

pub(super) fn ensure_stage_known(
    profile: &EffectiveDefaults,
    stage: &StageId,
    context: &str,
) -> anyhow::Result<()> {
    if profile.tools.contains_key(stage) || profile.params.contains_key(stage) {
        return Ok(());
    }
    Err(anyhow::anyhow!("{} references unknown stage {}", context, stage.as_str()))
}

pub(super) fn ensure_tool_id_valid(tool: &ToolId, context: &str) -> anyhow::Result<()> {
    validate_tool_id(tool)
        .map_err(|err| anyhow::anyhow!("{} references invalid tool id {}: {err}", context, tool))
}
