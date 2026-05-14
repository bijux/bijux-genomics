use bijux_dna_core::ids::validate_tool_id;
use bijux_dna_core::ids::StageId;
use bijux_dna_core::ids::ToolId;

use crate::{DefaultParams, EffectiveDefaults};

pub(super) fn ensure_stage_known(
    profile: &EffectiveDefaults,
    stage: &StageId,
    context: &str,
) -> anyhow::Result<()> {
    if profile.tools.contains_key(stage) || profile.params.contains_key(stage) {
        return Ok(());
    }
    let stage_id = stage.as_str();
    Err(anyhow::anyhow!("{context} references unknown stage {stage_id}"))
}

pub(super) fn ensure_tool_id_valid(tool: &ToolId, context: &str) -> anyhow::Result<()> {
    validate_tool_id(tool)
        .map_err(|err| anyhow::anyhow!("{context} references invalid tool id {tool}: {err}"))
}

pub(super) fn ensure_params_match_stage(
    stage: &StageId,
    params: &DefaultParams,
    context: &str,
) -> anyhow::Result<()> {
    let stage_id = stage.as_str();
    let valid = match params {
        DefaultParams::Bam(_) => stage_id.starts_with("bam."),
        DefaultParams::Vcf(_) => stage_id.starts_with("vcf."),
        DefaultParams::Empty(_) => stage_id.starts_with("core.") || stage_id.starts_with("cross."),
        _ => stage_id.starts_with("fastq."),
    };
    if valid {
        return Ok(());
    }
    Err(anyhow::anyhow!("{context} uses params incompatible with stage {stage_id}"))
}
