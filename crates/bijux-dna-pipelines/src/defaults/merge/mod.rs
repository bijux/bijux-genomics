use crate::EffectiveDefaults;
use bijux_dna_core::ids::StageId;

mod override_application;

pub fn merge_effective_defaults(
    profile: &EffectiveDefaults,
    config: Option<&EffectiveDefaults>,
    cli: Option<&EffectiveDefaults>,
    api: Option<&EffectiveDefaults>,
) -> anyhow::Result<EffectiveDefaults> {
    let mut merged = profile.clone();
    if let Some(config) = config {
        override_application::apply(&mut merged, profile, config, "config override")?;
    }
    if let Some(cli) = cli {
        override_application::apply(&mut merged, profile, cli, "cli override")?;
    }
    if let Some(api) = api {
        override_application::apply(&mut merged, profile, api, "api override")?;
    }
    Ok(merged)
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
