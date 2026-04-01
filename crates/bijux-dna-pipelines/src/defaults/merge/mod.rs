use crate::EffectiveDefaults;

mod override_application;
mod validation;

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
