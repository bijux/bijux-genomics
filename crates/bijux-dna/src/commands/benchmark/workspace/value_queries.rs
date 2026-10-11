use std::path::Path;

use anyhow::Result;
use bijux_dna_api::v1::api::bench::benchmark_workspace_config_value;

use super::load_benchmark_workspace_config;

pub(crate) fn benchmark_workspace_value(
    cwd: &Path,
    explicit_path: Option<&Path>,
    key: &str,
) -> Result<String> {
    let workspace = load_benchmark_workspace_config(cwd, explicit_path)?;
    benchmark_workspace_config_value(&workspace, key)
}
