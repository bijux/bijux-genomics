//! Owner: benchmark campaign repository acquisition.
//! Campaign configuration owned by the repo boundary.
use anyhow::Context;
use anyhow::{anyhow, Result};
use bijux_dna_bench_model::validate_hpc_campaign_config_source as validate_confidential_config;
use bijux_dna_bench_model::{CampaignConfig, CampaignPolicies, SiteProfile};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
const BUILTIN_GENERIC_PROFILE: &str = "generic";

pub(crate) fn merge_site_profile_file(
    config: &mut CampaignConfig,
    config_path: &Path,
) -> Result<()> {
    let profile_name = config
        .slurm
        .site_profile
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(BUILTIN_GENERIC_PROFILE);
    let Some(config_dir) = config_path.parent() else {
        return Ok(());
    };
    let profile_path = config_dir.join("site-profiles").join(format!("{profile_name}.toml"));
    if !profile_path.exists() {
        return Ok(());
    }
    let raw = std::fs::read_to_string(&profile_path)
        .with_context(|| format!("read {}", profile_path.display()))?;
    let profile: SiteProfile =
        toml::from_str(&raw).with_context(|| format!("parse {}", profile_path.display()))?;
    config.site_profiles.insert(profile_name.to_string(), profile);
    Ok(())
}

pub(crate) fn load_campaign_config_raw(config_path: &Path) -> Result<(CampaignConfig, String)> {
    let raw = std::fs::read_to_string(config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    validate_confidential_config(&raw)?;
    let cfg: CampaignConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))?;
    Ok((cfg, raw))
}

pub(crate) fn load_policy_file(path: &Path) -> Result<CampaignPolicies> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let parsed = toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    Ok(parsed)
}

pub(crate) fn workspace_root_for_campaign_stage_catalog(start: &Path) -> Option<PathBuf> {
    let mut cursor =
        if start.is_dir() { start.to_path_buf() } else { start.parent()?.to_path_buf() };
    loop {
        let domain_dir = cursor.join("domain");
        let registry = bijux_dna_infra::configs_file(&cursor, "ci/registry/tool_registry.toml");
        if domain_dir.is_dir() && registry.is_file() {
            return Some(cursor);
        }
        let parent = cursor.parent()?.to_path_buf();
        cursor = parent;
    }
}

pub(crate) fn campaign_stage_catalog(root: &Path) -> Result<BTreeSet<String>> {
    let mut stage_ids = BTreeSet::new();
    for domain in ["fastq", "bam", "vcf"] {
        let stages_dir = root.join("domain").join(domain).join("stages");
        for entry in std::fs::read_dir(&stages_dir)
            .with_context(|| format!("read {}", stages_dir.display()))?
        {
            let path = entry?.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|value| value.to_str()) else {
                continue;
            };
            if stem.starts_with('_') {
                continue;
            }
            stage_ids.insert(format!("{domain}.{stem}"));
        }
    }
    Ok(stage_ids)
}

pub(crate) fn validate_campaign_job_stage_ids(
    config_path: &Path,
    config: &CampaignConfig,
    fallback_root: &Path,
) -> Result<()> {
    let workspace_root = workspace_root_for_campaign_stage_catalog(config_path)
        .or_else(|| workspace_root_for_campaign_stage_catalog(fallback_root));
    let workspace_root = workspace_root.ok_or_else(|| {
        anyhow!(
            "unable to locate workspace root for stage catalog validation from {}",
            config_path.display()
        )
    })?;
    let known_stage_ids = campaign_stage_catalog(&workspace_root)?;
    for job in &config.jobs {
        if !known_stage_ids.contains(&job.stage) {
            return Err(anyhow!(
                "campaign job `{}` references unknown stage `{}`",
                job.name.as_deref().unwrap_or("<unnamed>"),
                job.stage
            ));
        }
    }
    Ok(())
}
