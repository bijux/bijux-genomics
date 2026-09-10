//! Owner: acquisition of governed HPC stage-tool resource declarations.
use anyhow::{Context, Result};
use bijux_dna_bench_model::{
    project_hpc_job_resources, HpcJobResourceHints, StageToolResourcesConfig,
};
use std::{fs, path::Path};
/// Canonical authored scheduler resource declarations.
pub const HPC_JOB_RESOURCE_CONFIG_PATH: &str = "configs/bench/local/stage-tool-resources.toml";
/// Acquire and admit one current resource declaration file.
/// # Errors
/// Returns acquisition, decoding, semantic or scheduler-unit projection errors.
pub fn load_hpc_job_resources(root: &Path) -> Result<HpcJobResourceHints> {
    let path = root.join(HPC_JOB_RESOURCE_CONFIG_PATH);
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: StageToolResourcesConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    project_hpc_job_resources(&config)
}
