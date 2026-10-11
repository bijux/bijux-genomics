//! Owner: local campaign copy-back planning and publication workflow.
use super::{campaign_dry_run, CampaignContext};
use anyhow::Result;
use bijux_dna_bench_model::{plan_campaign_copy_back, CopyBackManifestReport};
use std::path::Path;

/// Publish copy instructions for an admitted local campaign without transferring artifacts.
///
/// # Errors
/// Returns campaign acquisition, planning or manifest publication errors.
pub fn write_campaign_copy_back_manifest(
    context: &CampaignContext,
    config: &Path,
    env_file: Option<&Path>,
    user_policies: Option<&Path>,
    output_path: Option<&Path>,
) -> Result<CopyBackManifestReport> {
    let campaign = campaign_dry_run(context, config, env_file, user_policies)?;
    let manifest = plan_campaign_copy_back(&campaign, output_path);
    crate::repo::hpc_campaign::copy_back::publish_copy_back_manifest(&manifest)?;
    Ok(manifest)
}

#[cfg(test)]
#[path = "../../../tests/contracts/campaign/copy_back.rs"]
mod tests;
