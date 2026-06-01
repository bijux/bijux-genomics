use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::prelude::{StageId, ToolExecutionSpecV1, ToolId};
use bijux_dna_domain_fastq::stages::ids::{STAGE_DEPLETE_HOST, STAGE_INDEX_REFERENCE};
use bijux_dna_domain_fastq::STAGE_DEPLETE_RRNA;
use serde::Deserialize;

use crate::selection::{
    load_fastq_domain_tool_execution_spec, select_deplete_host_tools, select_deplete_rrna_tools,
    select_deplete_reference_contaminants_tools, select_index_reference_tools,
};
use crate::tool_adapters::fastq::deplete_host::plan_host_depletion_with_options;
use crate::tool_adapters::fastq::deplete_reference_contaminants::plan_contaminant_screen_with_options;
use crate::tool_adapters::fastq::deplete_rrna::plan_rrna_with_options;
use crate::tool_adapters::fastq::index_reference::plan_with_options;
use crate::{DepleteHostStageParams, DepleteRrnaStageParams, IndexReferenceStageParams};

const LOCAL_DEPLETE_REFERENCE_CONTAMINANTS_CONFIG_PATH: &str =
    "configs/bench/local/fastq-deplete-reference-contaminants.toml";
const LOCAL_INDEX_REFERENCE_CONFIG_PATH: &str = "configs/bench/local/fastq-index-reference.toml";
const LOCAL_DEPLETE_HOST_CONFIG_PATH: &str = "configs/bench/local/fastq-deplete-host.toml";
const LOCAL_DEPLETE_RRNA_CONFIG_PATH: &str = "configs/bench/local/fastq-deplete-rrna.toml";
const LOCAL_RUNTIME_PROFILE_PATH: &str = "configs/runtime/profiles/local.toml";
const DEFAULT_LOCAL_DEPLETE_REFERENCE_CONTAMINANTS_OUTPUT_DIR: &str =
    "target/local-ready/fastq.deplete_reference_contaminants";
const DEFAULT_LOCAL_INDEX_REFERENCE_OUTPUT_DIR: &str = "target/local-ready/fastq.index_reference";
const DEFAULT_LOCAL_DEPLETE_HOST_OUTPUT_DIR: &str = "target/local-ready/fastq.deplete_host";
const DEFAULT_LOCAL_DEPLETE_RRNA_OUTPUT_DIR: &str = "target/local-ready/fastq.deplete_rrna";

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
struct LocalDepleteRrnaPlanConfig {
    schema_version: String,
    input_r1: PathBuf,
    rrna_db: PathBuf,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct LocalDepleteHostPlanConfig {
    schema_version: String,
    input_r1: PathBuf,
    reference_index: PathBuf,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct LocalDepleteReferenceContaminantsPlanConfig {
    schema_version: String,
    input_r1: PathBuf,
    reference_index: PathBuf,
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
    hydrate_local_profile_defaults(&mut tool_spec, config.threads, &local_profile);
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

/// # Errors
/// Returns an error if the governed local-ready config or runtime profile cannot be read, the
/// configured reads/index/tool triple is invalid, the Bowtie2 index prefix is incomplete, or the
/// stage plan cannot be built.
pub fn local_deplete_host_plan(repo_root: &Path) -> Result<bijux_dna_stage_contract::StagePlanV1> {
    let config = load_local_deplete_host_plan_config(repo_root)?;
    let local_profile = load_local_runtime_profile(repo_root)?;
    let stage_id = StageId::new(STAGE_DEPLETE_HOST.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-ready tool_id `{}`: {error}", config.tool_id))?;

    let normalized_tools = select_deplete_host_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-ready fastq.deplete_host tool selection normalized unexpectedly: {:?}",
            normalized_tools
        ));
    }

    let input_r1_abs = repo_root.join(&config.input_r1);
    if !input_r1_abs.is_file() {
        return Err(anyhow!(
            "local-ready fastq.deplete_host input FASTQ is missing: {}",
            input_r1_abs.display()
        ));
    }

    let reference_index_abs = repo_root.join(&config.reference_index);
    ensure_bowtie2_index_prefix_exists(&reference_index_abs, "local-ready fastq.deplete_host")?;

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_local_profile_defaults(&mut tool_spec, config.threads, &local_profile);
    let out_dir = config
        .output_dir
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_DEPLETE_HOST_OUTPUT_DIR));

    plan_host_depletion_with_options(
        &tool_spec,
        &config.input_r1,
        None,
        &config.reference_index,
        &out_dir,
        &DepleteHostStageParams {
            host_identity_threshold: DepleteHostStageParams::baseline().host_identity_threshold,
            retain_unmapped_only: DepleteHostStageParams::baseline().retain_unmapped_only,
            threads: Some(tool_spec.resources.threads.max(1)),
        },
    )
}

/// # Errors
/// Returns an error if the governed local-ready config or runtime profile cannot be read, the
/// configured reads/index/tool triple is invalid, the Bowtie2 index prefix is incomplete, or the
/// stage plan cannot be built.
pub fn local_deplete_reference_contaminants_plan(
    repo_root: &Path,
) -> Result<bijux_dna_stage_contract::StagePlanV1> {
    let config = load_local_deplete_reference_contaminants_plan_config(repo_root)?;
    let local_profile = load_local_runtime_profile(repo_root)?;
    let stage_id = StageId::new(
        bijux_dna_domain_fastq::stages::ids::STAGE_DEPLETE_REFERENCE_CONTAMINANTS
            .as_str()
            .to_string(),
    );
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-ready tool_id `{}`: {error}", config.tool_id))?;

    let normalized_tools =
        select_deplete_reference_contaminants_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-ready fastq.deplete_reference_contaminants tool selection normalized unexpectedly: {:?}",
            normalized_tools
        ));
    }

    let input_r1_abs = repo_root.join(&config.input_r1);
    if !input_r1_abs.is_file() {
        return Err(anyhow!(
            "local-ready fastq.deplete_reference_contaminants input FASTQ is missing: {}",
            input_r1_abs.display()
        ));
    }

    let reference_index_abs = repo_root.join(&config.reference_index);
    ensure_bowtie2_index_prefix_exists(
        &reference_index_abs,
        "local-ready fastq.deplete_reference_contaminants",
    )?;

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_local_profile_defaults(&mut tool_spec, config.threads, &local_profile);
    let out_dir = config.output_dir.unwrap_or_else(|| {
        PathBuf::from(DEFAULT_LOCAL_DEPLETE_REFERENCE_CONTAMINANTS_OUTPUT_DIR)
    });

    plan_contaminant_screen_with_options(
        &tool_spec,
        &config.input_r1,
        None,
        &config.reference_index,
        &out_dir,
        &crate::DepleteReferenceContaminantsStageParams {
            decoy_mode: crate::DepleteReferenceContaminantsStageParams::baseline().decoy_mode,
            threads: Some(tool_spec.resources.threads.max(1)),
        },
    )
}

/// # Errors
/// Returns an error if the governed local-ready config or runtime profile cannot be read, the
/// configured reads/database/tool triple is invalid, or the stage plan cannot be built.
pub fn local_deplete_rrna_plan(repo_root: &Path) -> Result<bijux_dna_stage_contract::StagePlanV1> {
    let config = load_local_deplete_rrna_plan_config(repo_root)?;
    let local_profile = load_local_runtime_profile(repo_root)?;
    let stage_id = StageId::new(STAGE_DEPLETE_RRNA.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-ready tool_id `{}`: {error}", config.tool_id))?;

    let normalized_tools = select_deplete_rrna_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-ready fastq.deplete_rrna tool selection normalized unexpectedly: {:?}",
            normalized_tools
        ));
    }

    let input_r1_abs = repo_root.join(&config.input_r1);
    if !input_r1_abs.is_file() {
        return Err(anyhow!(
            "local-ready fastq.deplete_rrna input FASTQ is missing: {}",
            input_r1_abs.display()
        ));
    }

    let rrna_db_abs = repo_root.join(&config.rrna_db);
    if !rrna_db_abs.is_file() {
        return Err(anyhow!(
            "local-ready fastq.deplete_rrna rRNA reference is missing: {}",
            rrna_db_abs.display()
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_local_profile_defaults(&mut tool_spec, config.threads, &local_profile);
    let out_dir = config
        .output_dir
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_DEPLETE_RRNA_OUTPUT_DIR));

    plan_rrna_with_options(
        &tool_spec,
        &config.input_r1,
        None,
        &out_dir,
        &DepleteRrnaStageParams {
            rrna_db: config.rrna_db.display().to_string(),
            min_identity: DepleteRrnaStageParams::baseline().min_identity,
            threads: Some(tool_spec.resources.threads.max(1)),
        },
    )
}

fn hydrate_local_profile_defaults(
    tool_spec: &mut ToolExecutionSpecV1,
    configured_threads: Option<u32>,
    local_profile: &LocalRuntimeProfile,
) {
    let use_profile_memory_defaults = constraints_are_default(&tool_spec.resources);
    let threads = configured_threads.unwrap_or(local_profile.default_threads).max(1);
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

fn load_local_deplete_host_plan_config(repo_root: &Path) -> Result<LocalDepleteHostPlanConfig> {
    let path = repo_root.join(LOCAL_DEPLETE_HOST_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalDepleteHostPlanConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_deplete_host.v1" {
        return Err(anyhow!(
            "unsupported local-ready fastq.deplete_host schema_version `{}`",
            config.schema_version
        ));
    }
    Ok(config)
}

fn load_local_deplete_reference_contaminants_plan_config(
    repo_root: &Path,
) -> Result<LocalDepleteReferenceContaminantsPlanConfig> {
    let path = repo_root.join(LOCAL_DEPLETE_REFERENCE_CONTAMINANTS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalDepleteReferenceContaminantsPlanConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version
        != "bijux.bench.fastq.local_deplete_reference_contaminants.v1"
    {
        return Err(anyhow!(
            "unsupported local-ready fastq.deplete_reference_contaminants schema_version `{}`",
            config.schema_version
        ));
    }
    Ok(config)
}

fn load_local_deplete_rrna_plan_config(repo_root: &Path) -> Result<LocalDepleteRrnaPlanConfig> {
    let path = repo_root.join(LOCAL_DEPLETE_RRNA_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalDepleteRrnaPlanConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_deplete_rrna.v1" {
        return Err(anyhow!(
            "unsupported local-ready fastq.deplete_rrna schema_version `{}`",
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

fn ensure_bowtie2_index_prefix_exists(prefix: &Path, label: &str) -> Result<()> {
    let file_name = prefix
        .file_name()
        .ok_or_else(|| anyhow!("{label} reference index prefix has no file name"))?
        .to_string_lossy()
        .into_owned();
    let required_suffixes = [
        ".1.bt2",
        ".2.bt2",
        ".3.bt2",
        ".4.bt2",
        ".rev.1.bt2",
        ".rev.2.bt2",
    ];
    let missing = required_suffixes
        .into_iter()
        .map(|suffix| prefix.with_file_name(format!("{file_name}{suffix}")))
        .find(|path| !path.is_file());
    if let Some(path) = missing {
        return Err(anyhow!(
            "{label} reference index prefix is incomplete, missing {}",
            path.display()
        ));
    }
    Ok(())
}
