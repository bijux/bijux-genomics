use std::path::Path;

use anyhow::{anyhow, Result};

use super::load_benchmark_workspace_config;

pub(crate) fn benchmark_workspace_value(
    cwd: &Path,
    explicit_path: Option<&Path>,
    key: &str,
) -> Result<String> {
    let workspace = load_benchmark_workspace_config(cwd, explicit_path)?;
    match key {
        "local.results_root" => workspace
            .local
            .as_ref()
            .and_then(|row| row.results_root.clone()),
        "local.cache_mirror_root" => workspace
            .local
            .as_ref()
            .and_then(|row| row.cache_mirror_root.clone()),
        "local.extra_data_root" => workspace
            .local
            .as_ref()
            .and_then(|row| row.extra_data_root.clone()),
        "local.reference_root" => workspace
            .local
            .as_ref()
            .and_then(|row| row.reference_root.clone()),
        "remote.ssh_host" => workspace
            .remote
            .as_ref()
            .and_then(|row| row.ssh_host.clone()),
        "remote.repo_root" => workspace
            .remote
            .as_ref()
            .and_then(|row| row.repo_root.clone()),
        "remote.cache_root" => workspace
            .remote
            .as_ref()
            .and_then(|row| row.cache_root.clone()),
        "remote.corpus_root" => workspace
            .remote
            .as_ref()
            .and_then(|row| row.corpus_root.clone()),
        "remote.results_root" => workspace
            .remote
            .as_ref()
            .and_then(|row| row.results_root.clone()),
        "remote.extra_data_root" => workspace
            .remote
            .as_ref()
            .and_then(|row| row.extra_data_root.clone()),
        "remote.containers_root" => workspace
            .remote
            .as_ref()
            .and_then(|row| row.containers_root.clone()),
        "remote.reference_root" => workspace
            .remote
            .as_ref()
            .and_then(|row| row.reference_root.clone()),
        "sync.defaults.pull_base" => workspace
            .sync
            .as_ref()
            .and_then(|row| row.defaults.as_ref())
            .and_then(|row| row.pull_base.clone()),
        "sync.defaults.pull_mode" => workspace
            .sync
            .as_ref()
            .and_then(|row| row.defaults.as_ref())
            .and_then(|row| row.pull_mode.clone()),
        "sync.defaults.include_profile" => workspace
            .sync
            .as_ref()
            .and_then(|row| row.defaults.as_ref())
            .and_then(|row| row.include_profile.clone()),
        "sync.defaults.exclude_profile" => workspace
            .sync
            .as_ref()
            .and_then(|row| row.defaults.as_ref())
            .and_then(|row| row.exclude_profile.clone()),
        "sync.defaults.clean_context" => workspace
            .sync
            .as_ref()
            .and_then(|row| row.defaults.as_ref())
            .and_then(|row| row.clean_context.map(|value| value.to_string())),
        "sync.defaults.allow_dirty" => workspace
            .sync
            .as_ref()
            .and_then(|row| row.defaults.as_ref())
            .and_then(|row| row.allow_dirty.map(|value| value.to_string())),
        "sync.defaults.include_containers_manifest" => workspace
            .sync
            .as_ref()
            .and_then(|row| row.defaults.as_ref())
            .and_then(|row| {
                row.include_containers_manifest
                    .map(|value| value.to_string())
            }),
        "sync.defaults.data_manifest_glob" => workspace
            .sync
            .as_ref()
            .and_then(|row| row.defaults.as_ref())
            .and_then(|row| row.data_manifest_glob.clone()),
        other => return Err(anyhow!("unsupported benchmark workspace key `{other}`")),
    }
    .ok_or_else(|| anyhow!("missing benchmark workspace value for `{key}`"))
}
