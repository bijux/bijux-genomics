use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};

const CAMPAIGN_SCHEMA_VERSION: &str = "bijux.hpc.campaign.v1";
const ENV_DEFAULT_PATH: &str = "configs/hpc/.env";
const USER_OVERRIDE_DEFAULT_PATH: &str = "configs/hpc/campaign/user.override.toml";

const BUILTIN_LUNARC_PROFILE: &str = "lunarc";
const BUILTIN_GENERIC_PROFILE: &str = "generic";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CampaignConfig {
    #[serde(default = "default_campaign_schema_version")]
    pub schema_version: String,
    pub campaign: CampaignMeta,
    pub layout: CampaignLayout,
    #[serde(default)]
    pub slurm: CampaignSlurm,
    #[serde(default)]
    pub output_templates: OutputTemplates,
    #[serde(default)]
    pub resources: ResourceTemplates,
    #[serde(default)]
    pub security: CampaignSecurity,
    #[serde(default)]
    pub site_profiles: BTreeMap<String, SiteProfile>,
    #[serde(default)]
    pub jobs: Vec<CampaignJob>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CampaignMeta {
    pub id: String,
    pub domain: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CampaignLayout {
    pub corpora_root: PathBuf,
    pub databases_root: PathBuf,
    pub images_root: PathBuf,
    pub scratch_root: PathBuf,
    pub logs_root: PathBuf,
    pub encrypted_results_root: PathBuf,
    pub encrypted_code_root: PathBuf,
    pub appraiser_imports_root: PathBuf,
    pub baselines_root: PathBuf,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CampaignSlurm {
    pub site_profile: Option<String>,
    pub account: Option<String>,
    pub project: Option<String>,
    pub partition: Option<String>,
    pub qos: Option<String>,
    pub mail_user: Option<String>,
    pub default_resource_template: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CampaignJob {
    pub stage: String,
    pub tool: String,
    pub sample: String,
    #[serde(default)]
    pub array_task: Option<u32>,
    #[serde(default)]
    pub resource_template: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OutputTemplates {
    #[serde(default = "default_log_template")]
    pub log: String,
    #[serde(default = "default_out_template")]
    pub out: String,
    #[serde(default = "default_err_template")]
    pub err: String,
    #[serde(default = "default_results_template")]
    pub results: String,
    #[serde(default = "default_code_template")]
    pub code: String,
}

impl Default for OutputTemplates {
    fn default() -> Self {
        Self {
            log: default_log_template(),
            out: default_out_template(),
            err: default_err_template(),
            results: default_results_template(),
            code: default_code_template(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourceTemplate {
    pub cpus: u32,
    pub mem_gb: u32,
    pub walltime: String,
    pub scratch_gb: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourceTemplates {
    #[serde(default = "default_resource_template_name")]
    pub default: String,
    #[serde(default = "default_resource_template_map")]
    pub templates: BTreeMap<String, ResourceTemplate>,
}

impl Default for ResourceTemplates {
    fn default() -> Self {
        Self {
            default: default_resource_template_name(),
            templates: default_resource_template_map(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CampaignSecurity {
    #[serde(default)]
    pub env_file: Option<PathBuf>,
    #[serde(default)]
    pub redacted_env_keys: Vec<String>,
    #[serde(default)]
    pub encryption_recipients: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SiteProfile {
    pub partition: Option<String>,
    pub qos: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CampaignOverrides {
    #[serde(default)]
    pub layout: Option<LayoutOverrides>,
    #[serde(default)]
    pub slurm: Option<SlurmOverrides>,
    #[serde(default)]
    pub resources: Option<ResourceOverrides>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct LayoutOverrides {
    pub corpora_root: Option<PathBuf>,
    pub databases_root: Option<PathBuf>,
    pub images_root: Option<PathBuf>,
    pub scratch_root: Option<PathBuf>,
    pub logs_root: Option<PathBuf>,
    pub encrypted_results_root: Option<PathBuf>,
    pub encrypted_code_root: Option<PathBuf>,
    pub appraiser_imports_root: Option<PathBuf>,
    pub baselines_root: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SlurmOverrides {
    pub site_profile: Option<String>,
    pub account: Option<String>,
    pub project: Option<String>,
    pub partition: Option<String>,
    pub qos: Option<String>,
    pub default_resource_template: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ResourceOverrides {
    pub default: Option<String>,
    #[serde(default)]
    pub templates: BTreeMap<String, ResourceTemplate>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CampaignPreflightReport {
    pub schema_version: &'static str,
    pub config_path: String,
    pub env_file_path: String,
    pub user_override_path: String,
    pub user_overrides_applied: bool,
    pub checks: Vec<CampaignCheck>,
    pub resolved_slurm: ResolvedSlurm,
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CampaignDryRunReport {
    pub schema_version: &'static str,
    pub config_path: String,
    pub env_file_path: String,
    pub user_override_path: String,
    pub user_overrides_applied: bool,
    pub campaign_id: String,
    pub domain: String,
    pub resolved_slurm: ResolvedSlurm,
    pub planned_jobs: Vec<PlannedJob>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlannedJob {
    pub job_id: String,
    pub stage: String,
    pub tool: String,
    pub sample: String,
    pub resource_template: String,
    pub resources: ResourceTemplate,
    pub outputs: PlannedOutputs,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlannedOutputs {
    pub log: String,
    pub out: String,
    pub err: String,
    pub results: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CampaignCheck {
    pub name: String,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedSlurm {
    pub site_profile: String,
    pub account_redacted: String,
    pub project_redacted: String,
    pub partition: String,
    pub qos: String,
    pub default_resource_template: String,
}

#[derive(Debug, Clone)]
struct ResolutionMetadata {
    env_file_path: PathBuf,
    user_override_path: PathBuf,
    user_overrides_applied: bool,
}

fn default_campaign_schema_version() -> String {
    CAMPAIGN_SCHEMA_VERSION.to_string()
}

fn default_resource_template_name() -> String {
    "standard".to_string()
}

fn default_resource_template_map() -> BTreeMap<String, ResourceTemplate> {
    BTreeMap::from([(
        "standard".to_string(),
        ResourceTemplate {
            cpus: 16,
            mem_gb: 64,
            walltime: "04:00:00".to_string(),
            scratch_gb: 128,
        },
    )])
}

fn default_log_template() -> String {
    "{campaign}/{domain}/{stage}/{tool}/{sample}/{job_id}-{timestamp}.log".to_string()
}

fn default_out_template() -> String {
    "{campaign}/{domain}/{stage}/{tool}/{sample}/{job_id}-{timestamp}.out".to_string()
}

fn default_err_template() -> String {
    "{campaign}/{domain}/{stage}/{tool}/{sample}/{job_id}-{timestamp}.err".to_string()
}

fn default_results_template() -> String {
    "{campaign}/{domain}/{stage}/{tool}/{sample}/{job_id}-{timestamp}.results".to_string()
}

fn default_code_template() -> String {
    "{campaign}/{domain}/{stage}/{tool}/{sample}/{job_id}-{timestamp}.code".to_string()
}

fn trim_to_option(value: Option<String>) -> Option<String> {
    value.map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

fn parse_env_line(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }
    let content =
        if let Some(rest) = trimmed.strip_prefix("export ") { rest.trim() } else { trimmed };
    let (key, value) = content.split_once('=')?;
    let key = key.trim().to_string();
    if key.is_empty() {
        return None;
    }
    let value = value.trim().trim_matches('"').trim_matches('\'').to_string();
    Some((key, value))
}

fn load_env_file(path: &Path) -> Result<BTreeMap<String, String>> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut map = BTreeMap::new();
    for line in raw.lines() {
        if let Some((key, value)) = parse_env_line(line) {
            map.insert(key, value);
        }
    }
    Ok(map)
}

fn resolve_from_env(keys: &[&str], env_map: &BTreeMap<String, String>) -> Option<String> {
    for key in keys {
        if let Ok(value) = std::env::var(key) {
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

fn redacted(value: Option<&str>) -> String {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return "<missing>".to_string();
    };
    if value.len() <= 4 {
        return "****".to_string();
    }
    format!("{}***{}", &value[0..2], &value[value.len() - 2..])
}

fn path_writable(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |delta| delta.as_nanos());
    let probe = path.join(format!(".bijux_probe_{nonce}"));
    std::fs::write(&probe, b"ok").and_then(|_| std::fs::remove_file(&probe)).is_ok()
}

fn env_file_private(path: &Path) -> Option<bool> {
    if !path.exists() {
        return None;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        return std::fs::metadata(path).ok().map(|meta| meta.permissions().mode() & 0o077 == 0);
    }
    #[cfg(not(unix))]
    {
        Some(true)
    }
}

fn is_secret_key_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    let allowlist = ["secret_provider", "secret_ref", "secret_env_key", "encryption_recipients"];
    if allowlist.iter().any(|allowed| lower.ends_with(allowed)) {
        return false;
    }
    let needles = ["password", "token", "secret", "private_key", "apikey", "api_key", "access_key"];
    needles.iter().any(|needle| lower.contains(needle))
}

fn looks_placeholder(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    value.starts_with('<')
        || value.contains("${")
        || lowered.contains("example")
        || lowered.contains("placeholder")
        || lowered == "changeme"
        || lowered == "replace_me"
}

fn validate_confidential_config(raw_toml: &str) -> Result<()> {
    let mut failures = Vec::new();
    let mut section = String::new();
    for (index, line) in raw_toml.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            section =
                trimmed.trim_start_matches('[').trim_end_matches(']').trim().to_ascii_lowercase();
            continue;
        }
        if trimmed.contains("BEGIN PRIVATE KEY") {
            failures.push(format!("line {} embeds private key material", index + 1));
            continue;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            let key = key.trim().to_ascii_lowercase();
            let value = value.trim().trim_matches('"').trim_matches('\'');
            if section == "slurm"
                && (key == "account" || key == "project" || key == "mail_user")
                && !value.is_empty()
                && !looks_placeholder(value)
            {
                failures.push(format!(
                    "line {} stores confidential slurm `{}` in tracked config; use env file",
                    index + 1,
                    key
                ));
            }
            if is_secret_key_name(&key) && !value.is_empty() && !looks_placeholder(value) {
                failures.push(format!(
                    "line {} stores sensitive key `{key}` in tracked config",
                    index + 1
                ));
            }
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(anyhow!("confidential config validation failed:\n{}", failures.join("\n")))
    }
}

fn built_in_site_profile(name: &str) -> Option<SiteProfile> {
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

fn apply_overrides(config: &mut CampaignConfig, overrides: CampaignOverrides) {
    if let Some(layout) = overrides.layout {
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
    if let Some(slurm) = overrides.slurm {
        config.slurm.site_profile =
            trim_to_option(slurm.site_profile).or(config.slurm.site_profile.take());
        config.slurm.account = trim_to_option(slurm.account).or(config.slurm.account.take());
        config.slurm.project = trim_to_option(slurm.project).or(config.slurm.project.take());
        config.slurm.partition = trim_to_option(slurm.partition).or(config.slurm.partition.take());
        config.slurm.qos = trim_to_option(slurm.qos).or(config.slurm.qos.take());
        config.slurm.default_resource_template = trim_to_option(slurm.default_resource_template)
            .or(config.slurm.default_resource_template.take());
    }
    if let Some(resources) = overrides.resources {
        if let Some(default) = trim_to_option(resources.default) {
            config.resources.default = default;
        }
        for (name, template) in resources.templates {
            config.resources.templates.insert(name, template);
        }
    }
}

fn template_tokens(template: &str) -> Result<BTreeSet<String>> {
    let re = Regex::new(r"\{([a-z_]+)\}").context("compile template regex")?;
    let mut out = BTreeSet::new();
    for cap in re.captures_iter(template) {
        if let Some(token) = cap.get(1) {
            out.insert(token.as_str().to_string());
        }
    }
    Ok(out)
}

fn validate_templates(templates: &OutputTemplates) -> Result<()> {
    let allowed = BTreeSet::from([
        "job_id".to_string(),
        "timestamp".to_string(),
        "campaign".to_string(),
        "domain".to_string(),
        "stage".to_string(),
        "tool".to_string(),
        "sample".to_string(),
        "array_task".to_string(),
    ]);
    let required = BTreeSet::from([
        "job_id".to_string(),
        "timestamp".to_string(),
        "campaign".to_string(),
        "domain".to_string(),
        "stage".to_string(),
        "tool".to_string(),
        "sample".to_string(),
    ]);
    for (name, template) in [
        ("log", templates.log.as_str()),
        ("out", templates.out.as_str()),
        ("err", templates.err.as_str()),
        ("results", templates.results.as_str()),
        ("code", templates.code.as_str()),
    ] {
        let tokens = template_tokens(template)?;
        for token in &tokens {
            if !allowed.contains(token.as_str()) {
                return Err(anyhow!(
                    "output template `{name}` references unsupported token `{token}`"
                ));
            }
        }
        for required_token in &required {
            if !tokens.contains(required_token.as_str()) {
                return Err(anyhow!(
                    "output template `{name}` must include required token `{required_token}`"
                ));
            }
        }
    }
    Ok(())
}

fn fill_template(template: &str, values: &BTreeMap<String, String>) -> Result<String> {
    let mut out = template.to_string();
    for token in template_tokens(template)? {
        let value = values
            .get(&token)
            .ok_or_else(|| anyhow!("template token `{token}` has no resolved value"))?;
        out = out.replace(&format!("{{{token}}}"), value);
    }
    Ok(out)
}

fn resolve_site_profile(config: &CampaignConfig) -> SiteProfile {
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

fn merge_site_profile_file(config: &mut CampaignConfig, config_path: &Path) -> Result<()> {
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

fn resolve_slurm(config: &CampaignConfig, env_map: &BTreeMap<String, String>) -> ResolvedSlurm {
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
        .or_else(|| resolve_from_env(&["BIJUX_SLURM_ACCOUNT", "SLURM_ACCOUNT"], env_map));

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
            )
        });

    let partition = config
        .slurm
        .partition
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| site_profile.partition);

    let qos = config
        .slurm
        .qos
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| site_profile.qos);

    let default_resource_template = config
        .slurm
        .default_resource_template
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| config.resources.default.clone());

    ResolvedSlurm {
        site_profile: site_profile_name,
        account_redacted: redacted(account.as_deref()),
        project_redacted: redacted(project.as_deref()),
        partition: partition.unwrap_or_else(|| "<missing>".to_string()),
        qos: qos.unwrap_or_else(|| "<missing>".to_string()),
        default_resource_template,
    }
}

fn now_timestamp_compact() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |delta| delta.as_secs());
    secs.to_string()
}

fn load_campaign_config_raw(config_path: &Path) -> Result<(CampaignConfig, String)> {
    let raw = std::fs::read_to_string(config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    validate_confidential_config(&raw)?;
    let cfg: CampaignConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))?;
    Ok((cfg, raw))
}

fn load_override_file(path: &Path) -> Result<CampaignOverrides> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let parsed = toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    Ok(parsed)
}

fn load_env_map(
    config: &CampaignConfig,
    env_file_override: Option<&Path>,
) -> Result<(BTreeMap<String, String>, PathBuf)> {
    let env_path = env_file_override
        .map(Path::to_path_buf)
        .or_else(|| config.security.env_file.clone())
        .unwrap_or_else(|| PathBuf::from(ENV_DEFAULT_PATH));

    if env_path.exists() {
        Ok((load_env_file(&env_path)?, env_path))
    } else {
        Ok((BTreeMap::new(), env_path))
    }
}

fn resolve_campaign_config(
    config_path: &Path,
    env_file_override: Option<&Path>,
    user_override_path: Option<&Path>,
) -> Result<(CampaignConfig, ResolvedSlurm, ResolutionMetadata)> {
    let (mut config, _) = load_campaign_config_raw(config_path)?;
    merge_site_profile_file(&mut config, config_path)?;

    let override_path = user_override_path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(USER_OVERRIDE_DEFAULT_PATH));
    let mut user_overrides_applied = false;
    if override_path.exists() {
        let overrides = load_override_file(&override_path)?;
        apply_overrides(&mut config, overrides);
        user_overrides_applied = true;
    }

    validate_templates(&config.output_templates)?;
    let (env_map, env_file_path) = load_env_map(&config, env_file_override)?;
    let resolved_slurm = resolve_slurm(&config, &env_map);
    Ok((
        config,
        resolved_slurm,
        ResolutionMetadata {
            env_file_path,
            user_override_path: override_path,
            user_overrides_applied,
        },
    ))
}

pub fn write_campaign_profiles(out_dir: &Path) -> Result<Vec<PathBuf>> {
    bijux_dna_infra::ensure_dir(out_dir)
        .with_context(|| format!("create {}", out_dir.display()))?;

    let lunarc_path = out_dir.join("lunarc-small.toml");
    let generic_path = out_dir.join("generic-small.toml");

    let lunarc = r#"schema_version = "bijux.hpc.campaign.v1"

[campaign]
id = "fastq-hpc-mini"
domain = "fastq"
description = "Small Lunarc profile for stage diagnostics"

[layout]
corpora_root = "/mnt/shared/bijux/corpora"
databases_root = "/mnt/shared/bijux/databases"
images_root = "/mnt/shared/bijux/images"
scratch_root = "/mnt/shared/bijux/scratch"
logs_root = "/mnt/shared/bijux/logs"
encrypted_results_root = "/mnt/shared/bijux/results"
encrypted_code_root = "/mnt/shared/bijux/code"
appraiser_imports_root = "/mnt/shared/bijux/appraiser-imports"
baselines_root = "/mnt/shared/bijux/baselines"

[slurm]
site_profile = "lunarc"
partition = "main"
qos = "normal"
default_resource_template = "standard"

[resources]
default = "standard"

[resources.templates.standard]
cpus = 16
mem_gb = 64
walltime = "04:00:00"
scratch_gb = 128

[[jobs]]
stage = "fastq.validate_reads"
tool = "seqkit_v2"
sample = "sample_0001"
resource_template = "standard"
"#;

    let generic = r#"schema_version = "bijux.hpc.campaign.v1"

[campaign]
id = "vcf-hpc-mini"
domain = "vcf"
description = "Portable Slurm profile"

[layout]
corpora_root = "/shared/bijux/corpora"
databases_root = "/shared/bijux/databases"
images_root = "/shared/bijux/images"
scratch_root = "/shared/bijux/scratch"
logs_root = "/shared/bijux/logs"
encrypted_results_root = "/shared/bijux/results"
encrypted_code_root = "/shared/bijux/code"
appraiser_imports_root = "/shared/bijux/appraiser-imports"
baselines_root = "/shared/bijux/baselines"

[slurm]
site_profile = "generic"
default_resource_template = "standard"

[resources]
default = "standard"

[resources.templates.standard]
cpus = 8
mem_gb = 32
walltime = "02:00:00"
scratch_gb = 64

[[jobs]]
stage = "vcf.validate"
tool = "bcftools_v1_20"
sample = "cohort_01"
resource_template = "standard"
"#;

    bijux_dna_api::v1::api::run::atomic_write_bytes(&lunarc_path, lunarc.as_bytes())
        .with_context(|| format!("write {}", lunarc_path.display()))?;
    bijux_dna_api::v1::api::run::atomic_write_bytes(&generic_path, generic.as_bytes())
        .with_context(|| format!("write {}", generic_path.display()))?;

    Ok(vec![lunarc_path, generic_path])
}

pub fn campaign_preflight(
    config_path: &Path,
    env_file_override: Option<&Path>,
    user_override_path: Option<&Path>,
) -> Result<CampaignPreflightReport> {
    let (config, resolved_slurm, metadata) =
        resolve_campaign_config(config_path, env_file_override, user_override_path)?;

    let mut checks = Vec::new();
    checks.push(CampaignCheck {
        name: "schema_version".to_string(),
        ok: config.schema_version == CAMPAIGN_SCHEMA_VERSION,
        detail: config.schema_version.clone(),
    });

    checks.push(CampaignCheck {
        name: "campaign_id_non_empty".to_string(),
        ok: !config.campaign.id.trim().is_empty(),
        detail: config.campaign.id.clone(),
    });

    checks.push(CampaignCheck {
        name: "jobs_non_empty".to_string(),
        ok: !config.jobs.is_empty(),
        detail: format!("jobs={}", config.jobs.len()),
    });

    checks.push(CampaignCheck {
        name: "slurm_account_resolved".to_string(),
        ok: resolved_slurm.account_redacted != "<missing>",
        detail: resolved_slurm.account_redacted.clone(),
    });
    checks.push(CampaignCheck {
        name: "slurm_project_resolved".to_string(),
        ok: resolved_slurm.project_redacted != "<missing>",
        detail: resolved_slurm.project_redacted.clone(),
    });

    checks.push(CampaignCheck {
        name: "slurm_partition_resolved".to_string(),
        ok: resolved_slurm.partition != "<missing>",
        detail: resolved_slurm.partition.clone(),
    });

    checks.push(CampaignCheck {
        name: "slurm_qos_resolved".to_string(),
        ok: resolved_slurm.qos != "<missing>",
        detail: resolved_slurm.qos.clone(),
    });

    checks.push(CampaignCheck {
        name: "encryption_recipients_present".to_string(),
        ok: !config.security.encryption_recipients.is_empty(),
        detail: format!("count={}", config.security.encryption_recipients.len()),
    });
    checks.push(CampaignCheck {
        name: "env_file_private".to_string(),
        ok: env_file_private(&metadata.env_file_path).unwrap_or(true),
        detail: metadata.env_file_path.display().to_string(),
    });

    for (name, path) in [
        ("corpora_root", &config.layout.corpora_root),
        ("databases_root", &config.layout.databases_root),
        ("images_root", &config.layout.images_root),
        ("scratch_root", &config.layout.scratch_root),
        ("logs_root", &config.layout.logs_root),
        ("encrypted_results_root", &config.layout.encrypted_results_root),
        ("encrypted_code_root", &config.layout.encrypted_code_root),
        ("appraiser_imports_root", &config.layout.appraiser_imports_root),
        ("baselines_root", &config.layout.baselines_root),
    ] {
        checks.push(CampaignCheck {
            name: format!("layout_{name}_absolute"),
            ok: path.is_absolute(),
            detail: path.display().to_string(),
        });
        checks.push(CampaignCheck {
            name: format!("layout_{name}_exists"),
            ok: path.exists(),
            detail: path.display().to_string(),
        });
        checks.push(CampaignCheck {
            name: format!("layout_{name}_writable"),
            ok: path_writable(path),
            detail: path.display().to_string(),
        });
    }

    let default_template_exists =
        config.resources.templates.contains_key(&resolved_slurm.default_resource_template);
    checks.push(CampaignCheck {
        name: "default_resource_template_exists".to_string(),
        ok: default_template_exists,
        detail: resolved_slurm.default_resource_template.clone(),
    });

    for job in &config.jobs {
        let template_name = job
            .resource_template
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(&resolved_slurm.default_resource_template)
            .to_string();
        checks.push(CampaignCheck {
            name: format!("job_template_present:{}:{}", job.stage, job.sample),
            ok: config.resources.templates.contains_key(&template_name),
            detail: template_name,
        });
    }

    let ok = checks.iter().all(|check| check.ok);
    Ok(CampaignPreflightReport {
        schema_version: CAMPAIGN_SCHEMA_VERSION,
        config_path: config_path.display().to_string(),
        env_file_path: metadata.env_file_path.display().to_string(),
        user_override_path: metadata.user_override_path.display().to_string(),
        user_overrides_applied: metadata.user_overrides_applied,
        checks,
        resolved_slurm,
        ok,
    })
}

pub fn campaign_dry_run(
    config_path: &Path,
    env_file_override: Option<&Path>,
    user_override_path: Option<&Path>,
) -> Result<CampaignDryRunReport> {
    let (config, resolved_slurm, metadata) =
        resolve_campaign_config(config_path, env_file_override, user_override_path)?;
    let timestamp = now_timestamp_compact();
    let mut planned_jobs = Vec::new();

    for (index, job) in config.jobs.iter().enumerate() {
        let job_id = format!("dryrun-{:04}", index + 1);
        let template_name = job
            .resource_template
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(&resolved_slurm.default_resource_template)
            .to_string();

        let resources =
            config.resources.templates.get(&template_name).cloned().ok_or_else(|| {
                anyhow!(
                    "job `{}` references unknown resource template `{template_name}`",
                    job.stage
                )
            })?;

        let mut values = BTreeMap::new();
        values.insert("job_id".to_string(), job_id.clone());
        values.insert("timestamp".to_string(), timestamp.clone());
        values.insert("campaign".to_string(), config.campaign.id.clone());
        values.insert("domain".to_string(), config.campaign.domain.clone());
        values.insert("stage".to_string(), job.stage.clone());
        values.insert("tool".to_string(), job.tool.clone());
        values.insert("sample".to_string(), job.sample.clone());
        values.insert("array_task".to_string(), job.array_task.unwrap_or(0).to_string());

        let outputs = PlannedOutputs {
            log: config
                .layout
                .logs_root
                .join(fill_template(&config.output_templates.log, &values)?)
                .display()
                .to_string(),
            out: config
                .layout
                .logs_root
                .join(fill_template(&config.output_templates.out, &values)?)
                .display()
                .to_string(),
            err: config
                .layout
                .logs_root
                .join(fill_template(&config.output_templates.err, &values)?)
                .display()
                .to_string(),
            results: config
                .layout
                .encrypted_results_root
                .join(fill_template(&config.output_templates.results, &values)?)
                .display()
                .to_string(),
            code: config
                .layout
                .encrypted_code_root
                .join(fill_template(&config.output_templates.code, &values)?)
                .display()
                .to_string(),
        };

        planned_jobs.push(PlannedJob {
            job_id,
            stage: job.stage.clone(),
            tool: job.tool.clone(),
            sample: job.sample.clone(),
            resource_template: template_name,
            resources,
            outputs,
        });
    }

    Ok(CampaignDryRunReport {
        schema_version: CAMPAIGN_SCHEMA_VERSION,
        config_path: config_path.display().to_string(),
        env_file_path: metadata.env_file_path.display().to_string(),
        user_override_path: metadata.user_override_path.display().to_string(),
        user_overrides_applied: metadata.user_overrides_applied,
        campaign_id: config.campaign.id,
        domain: config.campaign.domain,
        resolved_slurm,
        planned_jobs,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::{
        campaign_dry_run, campaign_preflight, default_resource_template_map, parse_env_line,
        validate_confidential_config, ENV_DEFAULT_PATH,
    };

    #[test]
    fn parse_env_line_supports_export_prefix() {
        let row = parse_env_line("export BIJUX_SLURM_ACCOUNT=proj-123").expect("parsed");
        assert_eq!(row.0, "BIJUX_SLURM_ACCOUNT");
        assert_eq!(row.1, "proj-123");
    }

    #[test]
    fn confidential_config_rejects_secret_values() {
        let err = validate_confidential_config("token = \"abcd\"").expect_err("must reject secret");
        assert!(err.to_string().contains("sensitive key `token`"));
    }

    #[test]
    fn confidential_config_rejects_tracked_slurm_account_fields() {
        let err = validate_confidential_config(
            r#"
[slurm]
account = "lunarc-account"
project = "lunarc-project"
"#,
        )
        .expect_err("must reject tracked slurm account fields");
        assert!(err.to_string().contains("confidential slurm `account`"));
        assert!(err.to_string().contains("confidential slurm `project`"));
    }

    #[test]
    fn campaign_templates_require_core_tokens() {
        let root = tempfile::tempdir().expect("tempdir");
        let config_path = root.path().join("campaign.toml");
        let config = r#"
[campaign]
id = "mini"
domain = "fastq"

[layout]
corpora_root = "/shared/corpora"
databases_root = "/shared/databases"
images_root = "/shared/images"
scratch_root = "/shared/scratch"
logs_root = "/shared/logs"
encrypted_results_root = "/shared/results"
encrypted_code_root = "/shared/code"
appraiser_imports_root = "/shared/imports"
baselines_root = "/shared/baselines"

[output_templates]
log = "{campaign}/{job_id}.log"
out = "{campaign}/{job_id}.out"
err = "{campaign}/{job_id}.err"
results = "{campaign}/{job_id}.results"
code = "{campaign}/{job_id}.code"

[slurm]
site_profile = "generic"

[security]
encryption_recipients = ["alice"]

[[jobs]]
stage = "fastq.validate_reads"
tool = "seqkit_v2"
sample = "sample-1"
"#;
        std::fs::write(&config_path, config).expect("write config");

        let err =
            campaign_preflight(&config_path, None, None).expect_err("must reject missing tokens");
        assert!(err.to_string().contains("must include required token"));
    }

    #[test]
    fn campaign_dry_run_expands_templates() {
        let root = tempfile::tempdir().expect("tempdir");
        let config_path = root.path().join("campaign.toml");
        let config = r#"
schema_version = "bijux.hpc.campaign.v1"

[campaign]
id = "mini"
domain = "fastq"

[layout]
corpora_root = "/shared/corpora"
databases_root = "/shared/databases"
images_root = "/shared/images"
scratch_root = "/shared/scratch"
logs_root = "/shared/logs"
encrypted_results_root = "/shared/results"
encrypted_code_root = "/shared/code"
appraiser_imports_root = "/shared/imports"
baselines_root = "/shared/baselines"

[slurm]
site_profile = "generic"
default_resource_template = "standard"

[resources]
default = "standard"

[resources.templates.standard]
cpus = 8
mem_gb = 32
walltime = "02:00:00"
scratch_gb = 64

[security]
encryption_recipients = ["alice"]

[[jobs]]
stage = "fastq.validate_reads"
tool = "seqkit_v2"
sample = "sample-1"
resource_template = "standard"
"#;
        std::fs::write(&config_path, config).expect("write config");

        let report = campaign_dry_run(
            &config_path,
            None,
            Some(root.path().join("missing.override").as_path()),
        )
        .expect("dry run");
        assert!(report.env_file_path.ends_with(ENV_DEFAULT_PATH));
        assert!(!report.user_overrides_applied);
        assert_eq!(report.planned_jobs.len(), 1);
        let job = &report.planned_jobs[0];
        assert!(job
            .outputs
            .log
            .contains("/shared/logs/mini/fastq/fastq.validate_reads/seqkit_v2/sample-1/"));
        assert!(job.outputs.results.ends_with(".results"));
        assert!(job.outputs.code.ends_with(".code"));
    }

    #[test]
    fn campaign_preflight_detects_missing_resource_template() {
        let root = tempfile::tempdir().expect("tempdir");
        let config_path = root.path().join("campaign.toml");
        let config = r#"
[campaign]
id = "mini"
domain = "bam"

[layout]
corpora_root = "/shared/corpora"
databases_root = "/shared/databases"
images_root = "/shared/images"
scratch_root = "/shared/scratch"
logs_root = "/shared/logs"
encrypted_results_root = "/shared/results"
encrypted_code_root = "/shared/code"
appraiser_imports_root = "/shared/imports"
baselines_root = "/shared/baselines"

[slurm]
site_profile = "generic"
default_resource_template = "standard"

[security]
encryption_recipients = ["alice"]

[[jobs]]
stage = "bam.sort"
tool = "samtools"
sample = "sample-1"
resource_template = "missing"
"#;
        std::fs::write(&config_path, config).expect("write config");

        let report = campaign_preflight(
            &config_path,
            None,
            Some(root.path().join("missing.override").as_path()),
        )
        .expect("preflight");
        assert!(!report.ok);
        assert!(report
            .checks
            .iter()
            .any(|check| check.name.starts_with("job_template_present") && !check.ok));
        assert!(default_resource_template_map().contains_key("standard"));
    }

    #[test]
    fn campaign_reports_mark_user_overrides_when_file_exists() {
        let root = tempfile::tempdir().expect("tempdir");
        let config_path = root.path().join("campaign.toml");
        let override_path = root.path().join("user.override.toml");
        let config = r#"
[campaign]
id = "mini"
domain = "fastq"

[layout]
corpora_root = "/shared/corpora"
databases_root = "/shared/databases"
images_root = "/shared/images"
scratch_root = "/shared/scratch"
logs_root = "/shared/logs"
encrypted_results_root = "/shared/results"
encrypted_code_root = "/shared/code"
appraiser_imports_root = "/shared/imports"
baselines_root = "/shared/baselines"

[slurm]
site_profile = "generic"

[security]
encryption_recipients = ["alice"]

[[jobs]]
stage = "fastq.validate_reads"
tool = "seqkit_v2"
sample = "sample-1"
"#;
        let override_toml = r#"
[slurm]
partition = "debug"
"#;
        std::fs::write(&config_path, config).expect("write config");
        std::fs::write(&override_path, override_toml).expect("write override");

        let report = campaign_dry_run(&config_path, None, Some(&override_path)).expect("dry run");
        assert!(report.user_overrides_applied);
        assert_eq!(report.user_override_path, override_path.display().to_string());
        assert_eq!(report.resolved_slurm.partition, "debug");
    }

    #[test]
    fn campaign_preflight_passes_when_layout_roots_are_writable() {
        let root = tempfile::tempdir().expect("tempdir");
        let config_path = root.path().join("campaign.toml");
        let env_file_path = root.path().join("campaign.env");
        let corpora_root = root.path().join("corpora");
        let databases_root = root.path().join("databases");
        let images_root = root.path().join("images");
        let scratch_root = root.path().join("scratch");
        let logs_root = root.path().join("logs");
        let encrypted_results_root = root.path().join("results");
        let encrypted_code_root = root.path().join("code");
        let appraiser_imports_root = root.path().join("imports");
        let baselines_root = root.path().join("baselines");
        for dir in [
            &corpora_root,
            &databases_root,
            &images_root,
            &scratch_root,
            &logs_root,
            &encrypted_results_root,
            &encrypted_code_root,
            &appraiser_imports_root,
            &baselines_root,
        ] {
            std::fs::create_dir_all(dir).expect("create dir");
        }

        let config = format!(
            r#"
[campaign]
id = "mini"
domain = "fastq"

[layout]
corpora_root = "{corpora_root}"
databases_root = "{databases_root}"
images_root = "{images_root}"
scratch_root = "{scratch_root}"
logs_root = "{logs_root}"
encrypted_results_root = "{encrypted_results_root}"
encrypted_code_root = "{encrypted_code_root}"
appraiser_imports_root = "{appraiser_imports_root}"
baselines_root = "{baselines_root}"

[slurm]
site_profile = "generic"

[security]
encryption_recipients = ["alice"]

[[jobs]]
stage = "fastq.validate_reads"
tool = "seqkit_v2"
sample = "sample-1"
"#,
            corpora_root = corpora_root.display(),
            databases_root = databases_root.display(),
            images_root = images_root.display(),
            scratch_root = scratch_root.display(),
            logs_root = logs_root.display(),
            encrypted_results_root = encrypted_results_root.display(),
            encrypted_code_root = encrypted_code_root.display(),
            appraiser_imports_root = appraiser_imports_root.display(),
            baselines_root = baselines_root.display()
        );
        std::fs::write(&config_path, config).expect("write config");
        std::fs::write(
            &env_file_path,
            "BIJUX_SLURM_ACCOUNT=account-local\nBIJUX_SLURM_PROJECT=project-local\n",
        )
        .expect("write env");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&env_file_path).expect("env metadata").permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&env_file_path, perms).expect("set env mode");
        }

        let report =
            campaign_preflight(&config_path, Some(&env_file_path), None).expect("preflight");
        assert!(report.ok);
    }

    #[test]
    #[cfg(unix)]
    fn campaign_preflight_flags_world_readable_env_file() {
        use std::os::unix::fs::PermissionsExt;

        let root = tempfile::tempdir().expect("tempdir");
        let config_path = root.path().join("campaign.toml");
        let env_file_path = root.path().join("campaign.env");
        for name in [
            "corpora",
            "databases",
            "images",
            "scratch",
            "logs",
            "results",
            "code",
            "imports",
            "baselines",
        ] {
            std::fs::create_dir_all(root.path().join(name)).expect("create dir");
        }
        let config = format!(
            r#"
[campaign]
id = "mini"
domain = "fastq"

[layout]
corpora_root = "{root}/corpora"
databases_root = "{root}/databases"
images_root = "{root}/images"
scratch_root = "{root}/scratch"
logs_root = "{root}/logs"
encrypted_results_root = "{root}/results"
encrypted_code_root = "{root}/code"
appraiser_imports_root = "{root}/imports"
baselines_root = "{root}/baselines"

[slurm]
site_profile = "generic"

[security]
encryption_recipients = ["alice"]

[[jobs]]
stage = "fastq.validate_reads"
tool = "seqkit_v2"
sample = "sample-1"
"#,
            root = root.path().display()
        );
        std::fs::write(&config_path, config).expect("write config");
        std::fs::write(
            &env_file_path,
            "BIJUX_SLURM_ACCOUNT=account-local\nBIJUX_SLURM_PROJECT=project-local\n",
        )
        .expect("write env");
        let mut perms = std::fs::metadata(&env_file_path).expect("env metadata").permissions();
        perms.set_mode(0o644);
        std::fs::set_permissions(&env_file_path, perms).expect("set env mode");

        let report =
            campaign_preflight(&config_path, Some(&env_file_path), None).expect("preflight");
        assert!(!report.ok);
        assert!(report.checks.iter().any(|check| check.name == "env_file_private" && !check.ok));
    }

    #[test]
    fn campaign_resolves_partition_from_site_profile_file() {
        let root = tempfile::tempdir().expect("tempdir");
        let config_dir = root.path().join("campaign");
        let profiles_dir = config_dir.join("site-profiles");
        std::fs::create_dir_all(&profiles_dir).expect("create profile dir");
        let config_path = config_dir.join("mini.toml");
        let env_file_path = root.path().join("campaign.env");

        std::fs::write(
            profiles_dir.join("generic.toml"),
            "partition = \"debug\"\nqos = \"short\"\n",
        )
        .expect("write profile");
        std::fs::write(
            &env_file_path,
            "BIJUX_SLURM_ACCOUNT=account-local\nBIJUX_SLURM_PROJECT=project-local\n",
        )
        .expect("write env");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&env_file_path).expect("env metadata").permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&env_file_path, perms).expect("set env mode");
        }
        for name in [
            "corpora",
            "databases",
            "images",
            "scratch",
            "logs",
            "results",
            "code",
            "imports",
            "baselines",
        ] {
            std::fs::create_dir_all(root.path().join(name)).expect("create dir");
        }
        let config = format!(
            r#"
[campaign]
id = "mini"
domain = "fastq"

[layout]
corpora_root = "{root}/corpora"
databases_root = "{root}/databases"
images_root = "{root}/images"
scratch_root = "{root}/scratch"
logs_root = "{root}/logs"
encrypted_results_root = "{root}/results"
encrypted_code_root = "{root}/code"
appraiser_imports_root = "{root}/imports"
baselines_root = "{root}/baselines"

[slurm]
site_profile = "generic"

[security]
encryption_recipients = ["alice"]

[[jobs]]
stage = "fastq.validate_reads"
tool = "seqkit_v2"
sample = "sample-1"
"#,
            root = root.path().display()
        );
        std::fs::write(&config_path, config).expect("write config");

        let report = campaign_dry_run(&config_path, Some(&env_file_path), None).expect("dry run");
        assert_eq!(report.resolved_slurm.partition, "debug");
        assert_eq!(report.resolved_slurm.qos, "short");
    }
}
