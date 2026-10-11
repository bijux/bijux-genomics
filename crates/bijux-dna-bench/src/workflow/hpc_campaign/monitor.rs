//! Owner: benchmark local campaign monitoring workflow.
use super::{campaign_dry_run, CampaignContext};
use anyhow::Result;
use bijux_dna_bench_model::{summarize_campaign_monitor, SlurmMonitorReport};
use std::path::Path;
/// Observe declared campaign outputs without scheduler effects.
///
/// # Errors
/// Returns configuration or submission observation admission errors.
pub fn monitor_campaign(
    context: &CampaignContext,
    config: &Path,
    env_file: Option<&Path>,
    user_policies: Option<&Path>,
    submission_manifest: Option<&Path>,
) -> Result<SlurmMonitorReport> {
    let report = campaign_dry_run(context, config, env_file, user_policies)?;
    let entries = crate::repo::hpc_campaign::monitor::observe_campaign_jobs(
        &context.workspace_root,
        &report,
        submission_manifest,
    )?;
    Ok(summarize_campaign_monitor(report.campaign_id, report.domain, entries))
}
