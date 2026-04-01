use bijux_dna_core::ids::StageId;

use crate::EffectiveDefaults;

pub fn merge_effective_defaults(
    profile: &EffectiveDefaults,
    config: Option<&EffectiveDefaults>,
    cli: Option<&EffectiveDefaults>,
    api: Option<&EffectiveDefaults>,
) -> anyhow::Result<EffectiveDefaults> {
    let mut merged = profile.clone();
    if let Some(config) = config {
        apply_overrides(&mut merged, profile, config, "config override")?;
    }
    if let Some(cli) = cli {
        apply_overrides(&mut merged, profile, cli, "cli override")?;
    }
    if let Some(api) = api {
        apply_overrides(&mut merged, profile, api, "api override")?;
    }
    Ok(merged)
}

fn apply_overrides(
    merged: &mut EffectiveDefaults,
    profile: &EffectiveDefaults,
    overrides: &EffectiveDefaults,
    rationale: &str,
) -> anyhow::Result<()> {
    for (stage, tool) in &overrides.tools {
        ensure_stage_known(profile, stage, "tool override")?;
        merged.tools.insert(stage.clone(), tool.clone());
        merged
            .rationales
            .insert(stage.clone(), rationale.to_string());
    }
    for (stage, params) in &overrides.params {
        ensure_stage_known(profile, stage, "params override")?;
        merged.params.insert(stage.clone(), params.clone());
        merged
            .rationales
            .insert(stage.clone(), rationale.to_string());
    }
    Ok(())
}

fn ensure_stage_known(
    profile: &EffectiveDefaults,
    stage: &StageId,
    context: &str,
) -> anyhow::Result<()> {
    if profile.tools.contains_key(stage) || profile.params.contains_key(stage) {
        return Ok(());
    }
    Err(anyhow::anyhow!(
        "{} references unknown stage {}",
        context,
        stage.as_str()
    ))
}
