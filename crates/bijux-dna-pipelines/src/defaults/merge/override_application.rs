use crate::EffectiveDefaults;

pub(super) fn apply(
    merged: &mut EffectiveDefaults,
    profile: &EffectiveDefaults,
    overrides: &EffectiveDefaults,
    rationale: &str,
) -> anyhow::Result<()> {
    for (stage, tool) in &overrides.tools {
        super::validation::ensure_stage_known(profile, stage, "tool override")?;
        merged.tools.insert(stage.clone(), tool.clone());
        merged.rationales.insert(stage.clone(), rationale.to_string());
    }
    for (stage, params) in &overrides.params {
        super::validation::ensure_stage_known(profile, stage, "params override")?;
        merged.params.insert(stage.clone(), params.clone());
        merged.rationales.insert(stage.clone(), rationale.to_string());
    }
    Ok(())
}
