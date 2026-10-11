//! Owner: benchmark campaign repository acquisition.
//! Campaign environment owned by the repo boundary.
use anyhow::Context;
use anyhow::Result;
use bijux_dna_bench_model::CampaignConfig;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
const ENV_DEFAULT_PATH: &str = "configs/hpc/.env";

pub(crate) fn parse_env_line(line: &str) -> Option<(String, String)> {
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

pub(crate) fn load_env_file(path: &Path) -> Result<BTreeMap<String, String>> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut map = BTreeMap::new();
    for line in raw.lines() {
        if let Some((key, value)) = parse_env_line(line) {
            map.insert(key, value);
        }
    }
    Ok(map)
}

pub(crate) fn path_writable(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |delta| delta.as_nanos());
    let probe = path.join(format!(".bijux_probe_{nonce}"));
    if bijux_dna_infra::write_bytes(&probe, b"ok").is_err() {
        return false;
    }
    bijux_dna_infra::remove_file(&probe).is_ok()
}

pub(crate) fn env_file_private(path: &Path) -> Option<bool> {
    if !path.exists() {
        return None;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path).ok().map(|meta| meta.permissions().mode() & 0o077 == 0)
    }
    #[cfg(not(unix))]
    {
        Some(true)
    }
}

pub(crate) fn command_available_in_path(command: &str, paths: Option<&OsStr>) -> bool {
    let Some(paths) = paths else {
        return false;
    };
    for dir in std::env::split_paths(paths) {
        let candidate = dir.join(command);
        if candidate.is_file() {
            return true;
        }
    }
    false
}

pub(crate) fn load_env_map(
    config: &CampaignConfig,
    env_file_policy: Option<&Path>,
    workspace_root: &Path,
) -> Result<(BTreeMap<String, String>, PathBuf)> {
    let env_path = env_file_policy
        .map(Path::to_path_buf)
        .or_else(|| config.security.env_file.clone())
        .unwrap_or_else(|| PathBuf::from(ENV_DEFAULT_PATH));

    let source_path = workspace_root.join(&env_path);
    if source_path.exists() {
        Ok((load_env_file(&source_path)?, env_path))
    } else {
        Ok((BTreeMap::new(), env_path))
    }
}

pub(crate) fn observe_campaign_preflight(
    config: &bijux_dna_bench_model::CampaignConfig,
    env_file_path: &Path,
    executable_search_path: Option<&OsStr>,
    workspace_root: &Path,
) -> bijux_dna_bench_model::CampaignPreflightObservations {
    use bijux_dna_bench_model::{CampaignLayoutObservation, CampaignPreflightObservations};
    let commands = ["age", "apptainer", "sbatch", "squeue", "sacct"]
        .into_iter()
        .map(|command| {
            (command.to_string(), command_available_in_path(command, executable_search_path))
        })
        .collect();
    let layout = [
        ("corpora_root", &config.layout.corpora_root),
        ("databases_root", &config.layout.databases_root),
        ("images_root", &config.layout.images_root),
        ("scratch_root", &config.layout.scratch_root),
        ("logs_root", &config.layout.logs_root),
        ("encrypted_results_root", &config.layout.encrypted_results_root),
        ("encrypted_code_root", &config.layout.encrypted_code_root),
        ("appraiser_imports_root", &config.layout.appraiser_imports_root),
        ("baselines_root", &config.layout.baselines_root),
    ]
    .into_iter()
    .map(|(name, path)| CampaignLayoutObservation {
        name: name.to_string(),
        path: path.to_path_buf(),
        exists: workspace_root.join(path).exists(),
        writable: path_writable(&workspace_root.join(path)),
    })
    .collect();
    CampaignPreflightObservations {
        env_file_private: env_file_private(&workspace_root.join(env_file_path)),
        commands,
        layout,
    }
}
