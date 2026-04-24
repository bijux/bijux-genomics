use bijux_dna_core::ids::StageId;

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
