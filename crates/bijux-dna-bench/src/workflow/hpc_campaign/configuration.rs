//! Owner: benchmark local campaign planning.
//! Campaign configuration owned by the workflow boundary.
use crate::repo::hpc_campaign::configuration::{
    load_campaign_config_raw, load_policy_file, merge_site_profile_file,
    validate_campaign_job_stage_ids,
};
use crate::repo::hpc_campaign::environment::load_env_map;
use crate::workflow::hpc_campaign::policies::{apply_policies, resolve_slurm};
use crate::workflow::hpc_campaign::templates::validate_templates;
use crate::workflow::hpc_campaign::CampaignContext;
use anyhow::Result;
use bijux_dna_bench_model::{CampaignConfig, ResolvedSlurm};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub(crate) struct ResolutionMetadata {
    pub(crate) env_file_path: PathBuf,
    pub(crate) user_policy_path: PathBuf,
    pub(crate) user_policies_applied: bool,
}

pub(crate) fn resolve_campaign_config(
    context: &CampaignContext,
    config_path: &Path,
    env_file_policy: Option<&Path>,
    user_policy_path: Option<&Path>,
) -> Result<(CampaignConfig, ResolvedSlurm, ResolutionMetadata)> {
    let source_path = context.workspace_root.join(config_path);
    let (mut config, _) = load_campaign_config_raw(&source_path)?;
    merge_site_profile_file(&mut config, &source_path)?;
    validate_campaign_job_stage_ids(&source_path, &config, &context.workspace_root)?;

    let policy_path = if let Some(path) = user_policy_path {
        path.to_path_buf()
    } else {
        context.campaign_root.join("user.policy.toml")
    };
    let mut user_policies_applied = false;
    let policy_source = context.workspace_root.join(&policy_path);
    if policy_source.exists() {
        let policies = load_policy_file(&policy_source)?;
        apply_policies(&mut config, policies);
        user_policies_applied = true;
    }

    validate_templates(&config.output_templates)?;
    let (env_map, env_file_path) = load_env_map(&config, env_file_policy, &context.workspace_root)?;
    let resolved_slurm = resolve_slurm(&config, &env_map, &context.process_environment);
    Ok((
        config,
        resolved_slurm,
        ResolutionMetadata { env_file_path, user_policy_path: policy_path, user_policies_applied },
    ))
}
