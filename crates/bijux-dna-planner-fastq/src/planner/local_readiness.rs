use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::prelude::{StageId, ToolExecutionSpecV1, ToolId};
use bijux_dna_domain_fastq::stages::ids::STAGE_INDEX_REFERENCE;
use serde::Deserialize;

use crate::selection::{load_fastq_domain_tool_execution_spec, select_index_reference_tools};
use crate::tool_adapters::fastq::index_reference::plan_with_options;
use crate::IndexReferenceStageParams;

const LOCAL_INDEX_REFERENCE_CONFIG_PATH: &str = "configs/bench/local/fastq-index-reference.toml";
const LOCAL_RUNTIME_PROFILE_PATH: &str = "configs/runtime/profiles/local.toml";
const DEFAULT_LOCAL_INDEX_REFERENCE_OUTPUT_DIR: &str = "target/local-ready/fastq.index_reference";

#[derive(Debug, Deserialize)]
struct LocalIndexReferencePlanConfig {
    schema_version: String,
    reference_fasta: PathBuf,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct LocalRuntimeProfile {
    default_threads: u32,
    default_mem_gb: u32,
}

/// # Errors
/// Returns an error if the governed local-ready config or runtime profile cannot be read, the
/// configured reference/tool pair is invalid, or the stage plan cannot be built.
pub fn local_index_reference_plan(
    repo_root: &Path,
) -> Result<bijux_dna_stage_contract::StagePlanV1> {
    let config = load_local_index_reference_plan_config(repo_root)?;
    let local_profile = load_local_runtime_profile(repo_root)?;
    let stage_id = StageId::new(STAGE_INDEX_REFERENCE.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-ready tool_id `{}`: {error}", config.tool_id))?;

    let normalized_tools = select_index_reference_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-ready fastq.index_reference tool selection normalized unexpectedly: {:?}",
            normalized_tools
        ));
    }

    let reference_fasta_abs = repo_root.join(&config.reference_fasta);
    if !reference_fasta_abs.is_file() {
        return Err(anyhow!(
            "local-ready fastq.index_reference reference FASTA is missing: {}",
            reference_fasta_abs.display()
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_local_profile_defaults(&mut tool_spec, &config, &local_profile);
    let out_dir = config
        .output_dir
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_INDEX_REFERENCE_OUTPUT_DIR));

    plan_with_options(
        &tool_spec,
        &config.reference_fasta,
        &out_dir,
        &IndexReferenceStageParams { threads: Some(tool_spec.resources.threads.max(1)) },
    )
}

fn hydrate_local_profile_defaults(
    tool_spec: &mut ToolExecutionSpecV1,
    config: &LocalIndexReferencePlanConfig,
    local_profile: &LocalRuntimeProfile,
) {
    let use_profile_memory_defaults = constraints_are_default(&tool_spec.resources);
    let threads = config.threads.unwrap_or(local_profile.default_threads).max(1);
    if use_profile_memory_defaults {
        tool_spec.resources.mem_gb = local_profile.default_mem_gb.max(1);
        tool_spec.resources.tmp_gb = local_profile.default_mem_gb.max(1);
        tool_spec.resources.threads = threads;
    } else {
        tool_spec.resources.threads = threads;
        tool_spec.resources.mem_gb = tool_spec.resources.mem_gb.max(1);
    }
}

fn constraints_are_default(constraints: &bijux_dna_core::prelude::ToolConstraints) -> bool {
    constraints.runtime == "local"
        && constraints.mem_gb == 1
        && constraints.tmp_gb == 1
        && constraints.threads == 1
}

fn load_local_index_reference_plan_config(
    repo_root: &Path,
) -> Result<LocalIndexReferencePlanConfig> {
    let path = repo_root.join(LOCAL_INDEX_REFERENCE_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalIndexReferencePlanConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_index_reference.v1" {
        return Err(anyhow!(
            "unsupported local-ready fastq.index_reference schema_version `{}`",
            config.schema_version
        ));
    }
    Ok(config)
}

fn load_local_runtime_profile(repo_root: &Path) -> Result<LocalRuntimeProfile> {
    let path = repo_root.join(LOCAL_RUNTIME_PROFILE_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}
