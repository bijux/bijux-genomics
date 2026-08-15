use std::path::Path;

use anyhow::Result;
use bijux_dna_api::v1::api::bench::publish_benchmark_workspace_layout_status;

use super::load_benchmark_config;

pub(crate) fn write_workspace_layout_status(
    cwd: &Path,
    explicit_config: Option<&Path>,
    docs_root: &Path,
) -> Result<()> {
    let config = load_benchmark_config(cwd, explicit_config)?;
    publish_benchmark_workspace_layout_status(&config, docs_root)
}
