use anyhow::{anyhow, Context, Result};

use crate::commands::cli::Cli;
use bijux_dna_api::v1::api::run::{load_profile, resolve_run_base_dir};

pub(crate) fn load_profile_for_cli(cli: &Cli) -> Result<bijux_dna_api::v1::api::run::Profile> {
    let cwd = std::env::current_dir().context("resolve current directory")?;
    let profile_path =
        bijux_dna_infra::configs_file(&cwd, &format!("runtime/profile.{}.toml", cli.profile));
    let mut profile = load_profile(&profile_path)
        .map_err(|err| anyhow!("failed to load profile {}: {err}", profile_path.display()))?;
    profile.run_base_dir = resolve_run_base_dir(&cwd, &profile.run_base_dir);
    Ok(profile)
}

pub(crate) fn ensure_profile_run_base_dir(
    stage: &bijux_dna_api::v1::api::run::StageId,
    tool: &bijux_dna_api::v1::api::run::ToolId,
    profile: &mut bijux_dna_api::v1::api::run::Profile,
) {
    let run_dir = bijux_dna_api::v1::api::run::run_dir(
        &profile.run_base_dir,
        &bijux_dna_api::v1::api::run::new_run_id(),
        stage,
        tool,
    );
    if run_dir.starts_with(
        profile
            .run_base_dir
            .join(bijux_dna_api::v1::api::run::RUN_LAYOUT_CONTRACT.runs_dir),
    ) {
        let base = profile
            .run_base_dir
            .parent()
            .unwrap_or(&profile.run_base_dir);
        profile.run_base_dir = base.to_path_buf();
    }
}
