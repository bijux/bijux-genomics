//! HPC campaign request adaptation. Configuration and planning belong to benchmark owners.
use crate::commands::benchmark::path_resolution::BenchmarkPathResolver;
use anyhow::{Context, Result};
use bijux_dna_api::v1::api::bench::CampaignContext;
pub use bijux_dna_api::v1::api::bench::{
    is_supported_encryption_backend, CampaignCheck, CampaignConfig, CampaignDryRunReport,
    CampaignJob, CampaignLayout, CampaignMeta, CampaignPolicies, CampaignPreflightReport,
    CampaignSecurity, CampaignSlurm, CorpusSizeScalePolicy, LayoutPolicies, OutputTemplates,
    PlannedJob, PlannedLayout, PlannedOutputs, PlannedSecurity, ResolvedSlurm, ResourcePolicies,
    ResourceTemplate, ResourceTemplates, SiteProfile, SlurmPolicies, CAMPAIGN_SCHEMA_VERSION,
};
use std::path::{Path, PathBuf};
const DEFAULT_CAMPAIGN_PROFILE_OUT_DIR: &str = "benchmarks/configs/hpc/campaign";
pub(super) fn campaign_context() -> Result<CampaignContext> {
    let workspace_root = std::env::current_dir().context("resolve current directory")?;
    Ok(CampaignContext {
        campaign_root: BenchmarkPathResolver::new(&workspace_root, None)
            .benchmark_hpc_campaign_root(),
        workspace_root,
        process_environment: [
            "BIJUX_SLURM_ACCOUNT",
            "SLURM_ACCOUNT",
            "BIJUX_SLURM_PROJECT",
            "BIJUX_HPC_PROJECT",
            "SLURM_PROJECT",
        ]
        .into_iter()
        .filter_map(|key| std::env::var(key).ok().map(|value| (key.to_owned(), value)))
        .collect(),
        executable_search_path: std::env::var_os("PATH"),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |duration| duration.as_secs())
            .to_string(),
    })
}
pub fn campaign_dry_run(
    config_path: &Path,
    env_file_policy: Option<&Path>,
    user_policy_path: Option<&Path>,
) -> Result<CampaignDryRunReport> {
    bijux_dna_api::v1::api::bench::campaign_dry_run(
        &campaign_context()?,
        config_path,
        env_file_policy,
        user_policy_path,
    )
}
pub fn campaign_preflight(
    config_path: &Path,
    env_file_policy: Option<&Path>,
    user_policy_path: Option<&Path>,
) -> Result<CampaignPreflightReport> {
    bijux_dna_api::v1::api::bench::campaign_preflight(
        &campaign_context()?,
        config_path,
        env_file_policy,
        user_policy_path,
    )
}
pub fn write_campaign_profiles(out_dir: &Path) -> Result<Vec<PathBuf>> {
    let context = campaign_context()?;
    let output = if out_dir == Path::new(DEFAULT_CAMPAIGN_PROFILE_OUT_DIR) {
        context.campaign_root
    } else if out_dir.is_absolute() {
        out_dir.to_path_buf()
    } else {
        context.workspace_root.join(out_dir)
    };
    bijux_dna_api::v1::api::bench::write_campaign_profiles(&output)
}
