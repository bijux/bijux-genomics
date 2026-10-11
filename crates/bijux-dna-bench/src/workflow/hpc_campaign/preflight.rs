//! Owner: benchmark local campaign planning.
//! Acquire campaign inputs and delegate all preflight decisions to the model policy.
use super::{configuration::resolve_campaign_config, CampaignContext};
use crate::repo::hpc_campaign::environment::observe_campaign_preflight;
use anyhow::Result;
use bijux_dna_bench_model::{
    evaluate_campaign_preflight, CampaignPreflightInputs, CampaignPreflightReport,
};
use std::path::Path;

pub fn campaign_preflight(
    context: &CampaignContext,
    config_path: &Path,
    env_file_policy: Option<&Path>,
    user_policy_path: Option<&Path>,
) -> Result<CampaignPreflightReport> {
    let (config, resolved_slurm, metadata) =
        resolve_campaign_config(context, config_path, env_file_policy, user_policy_path)?;
    let observations = observe_campaign_preflight(
        &config,
        &metadata.env_file_path,
        context.executable_search_path.as_deref(),
        &context.workspace_root,
    );
    Ok(evaluate_campaign_preflight(CampaignPreflightInputs {
        config: &config,
        config_path,
        env_file_path: &metadata.env_file_path,
        user_policy_path: &metadata.user_policy_path,
        user_policies_applied: metadata.user_policies_applied,
        resolved_slurm,
        observations,
    })?)
}
