//! Owner: benchmark local campaign planning.
//! Campaign policies owned by the workflow boundary.
use bijux_dna_bench_model::{CampaignConfig, CampaignPolicies, ResolvedSlurm, SiteProfile};
use std::collections::BTreeMap;
const BUILTIN_LUNARC_PROFILE: &str = "lunarc";
const BUILTIN_GENERIC_PROFILE: &str = "generic";

pub(crate) fn trim_to_option(value: Option<String>) -> Option<String> {
    value.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

pub(crate) fn resolve_from_env(
    keys: &[&str],
    env_map: &BTreeMap<String, String>,
    process_environment: &BTreeMap<String, String>,
) -> Option<String> {
    for key in keys {
        if let Some(value) = process_environment.get(*key) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    for key in keys {
        if let Some(value) = env_map.get(*key).map(|v| v.trim()).filter(|v| !v.is_empty()) {
            return Some(value.to_string());
        }
    }
    None
}

pub(crate) fn redacted(value: Option<&str>) -> String {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return "<missing>".to_string();
    };
    if value.chars().count() <= 4 {
        return "****".to_string();
    }
    let mut characters = value.chars();
    let prefix = characters.by_ref().take(2).collect::<String>();
    let suffix = characters.rev().take(2).collect::<Vec<_>>().into_iter().rev().collect::<String>();
    format!("{prefix}***{suffix}")
}

pub(crate) fn built_in_site_profile(name: &str) -> Option<SiteProfile> {
    match name {
        BUILTIN_LUNARC_PROFILE => Some(SiteProfile {
            partition: Some("main".to_string()),
            qos: Some("normal".to_string()),
        }),
        BUILTIN_GENERIC_PROFILE => Some(SiteProfile {
            partition: Some("compute".to_string()),
            qos: Some("normal".to_string()),
        }),
        _ => None,
    }
}

pub(crate) fn apply_policies(config: &mut CampaignConfig, policies: CampaignPolicies) {
    if let Some(layout) = policies.layout {
        if let Some(value) = layout.corpora_root {
            config.layout.corpora_root = value;
        }
        if let Some(value) = layout.databases_root {
            config.layout.databases_root = value;
        }
        if let Some(value) = layout.images_root {
            config.layout.images_root = value;
        }
        if let Some(value) = layout.scratch_root {
            config.layout.scratch_root = value;
        }
        if let Some(value) = layout.logs_root {
            config.layout.logs_root = value;
        }
        if let Some(value) = layout.encrypted_results_root {
            config.layout.encrypted_results_root = value;
        }
        if let Some(value) = layout.encrypted_code_root {
            config.layout.encrypted_code_root = value;
        }
        if let Some(value) = layout.appraiser_imports_root {
            config.layout.appraiser_imports_root = value;
        }
        if let Some(value) = layout.baselines_root {
            config.layout.baselines_root = value;
        }
    }
    if let Some(slurm) = policies.slurm {
        config.slurm.site_profile =
            trim_to_option(slurm.site_profile).or(config.slurm.site_profile.take());
        config.slurm.account = trim_to_option(slurm.account).or(config.slurm.account.take());
        config.slurm.project = trim_to_option(slurm.project).or(config.slurm.project.take());
        config.slurm.partition = trim_to_option(slurm.partition).or(config.slurm.partition.take());
        config.slurm.qos = trim_to_option(slurm.qos).or(config.slurm.qos.take());
        config.slurm.default_resource_template = trim_to_option(slurm.default_resource_template)
            .or(config.slurm.default_resource_template.take());
        if let Some(value) = slurm.retry_attempts {
            config.slurm.retry_attempts = Some(value);
        }
        if let Some(value) = slurm.retry_backoff_seconds {
            config.slurm.retry_backoff_seconds = Some(value);
        }
        if let Some(values) = slurm.retry_on_exit_codes {
            config.slurm.retry_on_exit_codes = values;
        }
    }
    if let Some(resources) = policies.resources {
        if let Some(default) = trim_to_option(resources.default) {
            config.resources.default = default;
        }
        for (name, template) in resources.templates {
            config.resources.templates.insert(name, template);
        }
    }
}

pub(crate) fn resolve_site_profile(config: &CampaignConfig) -> SiteProfile {
    let profile_name = config
        .slurm
        .site_profile
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(BUILTIN_GENERIC_PROFILE);

    config
        .site_profiles
        .get(profile_name)
        .cloned()
        .or_else(|| built_in_site_profile(profile_name))
        .unwrap_or_default()
}

pub(crate) fn resolve_slurm(
    config: &CampaignConfig,
    env_map: &BTreeMap<String, String>,
    process_environment: &BTreeMap<String, String>,
) -> ResolvedSlurm {
    let site_profile_name = config
        .slurm
        .site_profile
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(BUILTIN_GENERIC_PROFILE)
        .to_string();
    let site_profile = resolve_site_profile(config);

    let account = config
        .slurm
        .account
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            resolve_from_env(
                &["BIJUX_SLURM_ACCOUNT", "SLURM_ACCOUNT"],
                env_map,
                process_environment,
            )
        });

    let project = config
        .slurm
        .project
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            resolve_from_env(
                &["BIJUX_SLURM_PROJECT", "BIJUX_HPC_PROJECT", "SLURM_PROJECT"],
                env_map,
                process_environment,
            )
        });

    let partition = config
        .slurm
        .partition
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or(site_profile.partition);

    let qos = config
        .slurm
        .qos
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or(site_profile.qos);

    let default_resource_template = config
        .slurm
        .default_resource_template
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| config.resources.default.clone());
    let retry_attempts = config.slurm.retry_attempts.unwrap_or(1).max(1);
    let retry_backoff_seconds = config.slurm.retry_backoff_seconds.unwrap_or(30).max(1);
    let retry_on_exit_codes = if config.slurm.retry_on_exit_codes.is_empty() {
        vec![1]
    } else {
        config.slurm.retry_on_exit_codes.clone()
    };

    ResolvedSlurm {
        site_profile: site_profile_name,
        account_redacted: redacted(account.as_deref()),
        project_redacted: redacted(project.as_deref()),
        partition: partition.unwrap_or_else(|| "<missing>".to_string()),
        qos: qos.unwrap_or_else(|| "<missing>".to_string()),
        default_resource_template,
        retry_attempts,
        retry_backoff_seconds,
        retry_on_exit_codes,
    }
}
