use std::path::Path;

use anyhow::{anyhow, Context, Result};
use bijux_dna_api::v1::api::bench::parse_benchmark_config;

pub(crate) use bijux_dna_api::v1::api::bench::expand_benchmark_config_env_placeholders as expand_env_placeholders;

use super::{
    benchmark_config_path, benchmark_publication_config_path, benchmark_workspace_config_path,
    BenchmarkConfig, BenchmarkPublicationConfig, BenchmarkWorkspaceConfig,
};

pub(crate) fn load_benchmark_config(
    cwd: &Path,
    explicit_path: Option<&Path>,
) -> Result<BenchmarkConfig> {
    let path = benchmark_config_path(cwd, explicit_path);
    if !path.is_file() {
        return Err(anyhow!("missing benchmark config: {}", path.display()));
    }
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("read benchmark config {}", path.display()))?;
    parse_benchmark_config(&raw).map_err(|error| anyhow!("parse {}: {error:#}", path.display()))
}

pub(crate) fn load_optional_benchmark_workspace_config(
    cwd: &Path,
    explicit_path: Option<&Path>,
) -> Result<Option<BenchmarkWorkspaceConfig>> {
    let path = benchmark_config_path(cwd, explicit_path);
    if !path.is_file() {
        return Ok(None);
    }
    let config = load_benchmark_config(cwd, explicit_path)?;
    if config.workspace == BenchmarkWorkspaceConfig::default() {
        return Ok(None);
    }
    Ok(Some(config.workspace))
}

pub(crate) fn load_benchmark_workspace_config(
    cwd: &Path,
    explicit_path: Option<&Path>,
) -> Result<BenchmarkWorkspaceConfig> {
    let path = benchmark_workspace_config_path(cwd, explicit_path);
    load_optional_benchmark_workspace_config(cwd, explicit_path)?
        .ok_or_else(|| anyhow!("missing benchmark workspace config: {}", path.display()))
}

pub(crate) fn load_benchmark_publication_config(
    cwd: &Path,
    explicit_path: Option<&Path>,
) -> Result<BenchmarkPublicationConfig> {
    let path = benchmark_publication_config_path(cwd, explicit_path);
    let config = load_benchmark_config(cwd, explicit_path)?;
    if config.publication == BenchmarkPublicationConfig::default() {
        return Err(anyhow!("missing benchmark publication config: {}", path.display()));
    }
    Ok(config.publication)
}
