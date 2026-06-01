use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::prelude::{ArtifactId, ArtifactRole, StageId, ToolExecutionSpecV1, ToolId};
use bijux_dna_domain_bam::params::{
    AlignEffectiveParams, ContaminationEffectiveParams, ContaminationScope, ReadGroupSpec,
};
#[cfg(feature = "bam_downstream")]
use bijux_dna_domain_bam::params::HaplogroupEffectiveParams;
use bijux_dna_domain_bam::{bam_alignment_strategy_for_tool, BamStage};
use serde::Deserialize;

use crate::selection::{allowed_tools_for_stage, load_bam_domain_tool_execution_spec};

const LOCAL_ALIGN_CONFIG_PATH: &str = "configs/bench/local/bam-align.toml";
const LOCAL_CONTAMINATION_CONFIG_PATH: &str = "configs/bench/local/bam-contamination.toml";
#[cfg(feature = "bam_downstream")]
const LOCAL_HAPLOGROUPS_CONFIG_PATH: &str = "configs/bench/local/bam-haplogroups.toml";
const LOCAL_RUNTIME_PROFILE_PATH: &str = "configs/runtime/profiles/local.toml";
const DEFAULT_LOCAL_ALIGN_OUTPUT_DIR: &str = "target/local-ready/bam.align";
const DEFAULT_LOCAL_CONTAMINATION_OUTPUT_DIR: &str = "target/local-ready/bam.contamination";
#[cfg(feature = "bam_downstream")]
const DEFAULT_LOCAL_HAPLOGROUPS_OUTPUT_DIR: &str = "target/local-ready/bam.haplogroups";

#[derive(Debug, Deserialize)]
struct LocalAlignPlanConfig {
    schema_version: String,
    input_r1: PathBuf,
    #[serde(default)]
    input_r2: Option<PathBuf>,
    reference_fasta: PathBuf,
    reference_index: PathBuf,
    tool_id: String,
    sample_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct LocalContaminationPlanConfig {
    schema_version: String,
    bam: PathBuf,
    bai: PathBuf,
    reference_fasta: PathBuf,
    reference_panels: Vec<PathBuf>,
    tool_id: String,
    sample_id: String,
    scope: ContaminationScope,
    #[serde(default)]
    prior: Option<f64>,
    #[serde(default)]
    sex_specific: bool,
    #[serde(default)]
    assumptions: Option<String>,
    #[serde(default)]
    chromosome_system: Option<String>,
    #[serde(default)]
    minimum_mean_coverage: Option<f64>,
    #[serde(default = "default_emit_confidence_caveats")]
    emit_confidence_caveats: bool,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
}

#[cfg(feature = "bam_downstream")]
#[derive(Debug, Deserialize)]
struct LocalHaplogroupsPlanConfig {
    schema_version: String,
    bam: PathBuf,
    bai: PathBuf,
    reference_fasta: PathBuf,
    reference_panel_id: String,
    reference_panel: PathBuf,
    tool_id: String,
    sample_id: String,
    reference_build: String,
    population_scope: String,
    #[serde(default)]
    min_coverage: Option<f64>,
    #[serde(default = "default_refuse_without_population_context")]
    refuse_without_population_context: bool,
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

fn default_emit_confidence_caveats() -> bool {
    true
}

#[cfg(feature = "bam_downstream")]
fn default_refuse_without_population_context() -> bool {
    true
}

/// # Errors
/// Returns an error if the governed local-ready config or runtime profile cannot be read, the
/// configured reads/reference/tool tuple is invalid, or the align plan cannot be built.
pub fn local_align_plan(repo_root: &Path) -> Result<bijux_dna_stage_contract::StagePlanV1> {
    let config = load_local_align_plan_config(repo_root)?;
    if config.schema_version != "bijux.bench.bam.local_align.v1" {
        return Err(anyhow!(
            "unexpected local-ready bam.align schema_version `{}`",
            config.schema_version
        ));
    }

    let local_profile = load_local_runtime_profile(repo_root)?;
    let stage = BamStage::Align;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-ready tool_id `{}`: {error}", config.tool_id))?;

    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-ready bam.align tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let input_r1_abs = repo_root.join(&config.input_r1);
    ensure_required_file(&input_r1_abs, "local-ready bam.align input_r1")?;
    if let Some(input_r2) = config.input_r2.as_ref() {
        ensure_required_file(&repo_root.join(input_r2), "local-ready bam.align input_r2")?;
    }

    let reference_fasta_abs = repo_root.join(&config.reference_fasta);
    ensure_required_file(&reference_fasta_abs, "local-ready bam.align reference FASTA")?;
    let reference_fai = PathBuf::from(format!("{}.fai", config.reference_fasta.display()));
    let reference_dict = config.reference_fasta.with_extension("dict");
    ensure_required_file(
        &repo_root.join(&reference_fai),
        "local-ready bam.align reference FASTA index",
    )?;
    ensure_required_file(
        &repo_root.join(&reference_dict),
        "local-ready bam.align reference sequence dictionary",
    )?;
    ensure_align_reference_index_exists(
        &repo_root.join(&config.reference_index),
        tool_id.as_str(),
    )?;

    let mut tool_spec = load_bam_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_local_profile_defaults(&mut tool_spec, config.threads, &local_profile);
    let strategy =
        bam_alignment_strategy_for_tool(tool_id.as_str(), Some("default")).ok_or_else(|| {
            anyhow!("local-ready bam.align tool `{}` has no governed alignment strategy", tool_id)
        })?;
    let out_dir =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_ALIGN_OUTPUT_DIR));
    let params = AlignEffectiveParams {
        aligner: tool_id.as_str().to_string(),
        strategy_id: strategy.strategy_id,
        preset: strategy.default_preset,
        mode: strategy.mode,
        threads: tool_spec.resources.threads.max(1),
        reference: config.reference_index.display().to_string(),
        reference_digest: "unknown".to_string(),
        rg_policy: bijux_dna_domain_bam::types::ReadGroupPolicy::Regenerate,
        read_group: ReadGroupSpec::with_defaults(&config.sample_id),
        sensitivity_profile: Some("default".to_string()),
        seed_length: None,
        build_indices: false,
        emit_stats: true,
    };

    let mut plan = crate::tool_adapters::bam::align::plan(
        &tool_spec,
        &config.input_r1,
        config.input_r2.as_deref(),
        &reference_fasta_abs,
        &config.sample_id,
        &params,
        &out_dir,
    )?;
    normalize_plan_path(&mut plan, &reference_fasta_abs, &config.reference_fasta);

    push_required_input(
        &mut plan,
        ArtifactId::from_static("reference_index"),
        &config.reference_index,
        ArtifactRole::Index,
    );
    push_required_input(
        &mut plan,
        ArtifactId::from_static("reference_fai"),
        &reference_fai,
        ArtifactRole::Index,
    );
    push_required_input(
        &mut plan,
        ArtifactId::from_static("reference_dict"),
        &reference_dict,
        ArtifactRole::Index,
    );

    let params = plan
        .params
        .as_object_mut()
        .ok_or_else(|| anyhow!("bam.align local-ready plan params must be a JSON object"))?;
    params.insert("reference_index".to_string(), serde_json::json!(config.reference_index));
    params.insert("reference_fai".to_string(), serde_json::json!(reference_fai));
    params.insert("reference_dict".to_string(), serde_json::json!(reference_dict));
    params.insert("tool".to_string(), serde_json::json!(tool_id.as_str()));

    Ok(plan)
}

/// # Errors
/// Returns an error if the governed local-ready config or runtime profile cannot be read, the
/// configured BAM/reference/panel/tool tuple is invalid, or the contamination plan cannot be
/// built.
pub fn local_contamination_plan(repo_root: &Path) -> Result<bijux_dna_stage_contract::StagePlanV1> {
    let config = load_local_contamination_plan_config(repo_root)?;
    if config.schema_version != "bijux.bench.bam.local_contamination.v1" {
        return Err(anyhow!(
            "unexpected local-ready bam.contamination schema_version `{}`",
            config.schema_version
        ));
    }
    if config.reference_panels.is_empty() {
        return Err(anyhow!(
            "local-ready bam.contamination requires at least one governed reference panel"
        ));
    }

    let local_profile = load_local_runtime_profile(repo_root)?;
    let stage = BamStage::Contamination;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-ready tool_id `{}`: {error}", config.tool_id))?;

    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-ready bam.contamination tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let bam_abs = repo_root.join(&config.bam);
    ensure_required_file(&bam_abs, "local-ready bam.contamination bam")?;
    let bai_abs = repo_root.join(&config.bai);
    ensure_required_file(&bai_abs, "local-ready bam.contamination bai")?;
    let reference_abs = repo_root.join(&config.reference_fasta);
    ensure_required_file(&reference_abs, "local-ready bam.contamination reference FASTA")?;
    for panel in &config.reference_panels {
        ensure_required_file(
            &repo_root.join(panel),
            "local-ready bam.contamination reference panel",
        )?;
    }

    let mut tool_spec = load_bam_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_local_profile_defaults(&mut tool_spec, config.threads, &local_profile);
    let out_dir =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_CONTAMINATION_OUTPUT_DIR));
    let params = ContaminationEffectiveParams {
        reference_panels: config
            .reference_panels
            .iter()
            .map(|path| path.display().to_string())
            .collect(),
        scope: config.scope,
        prior: config.prior,
        sex_specific: config.sex_specific,
        assumptions: config.assumptions,
        required_reference_digest: Some(bijux_dna_infra::hash_file_sha256(&reference_abs)?),
        chromosome_system: config.chromosome_system,
        minimum_mean_coverage: config.minimum_mean_coverage,
        emit_confidence_caveats: config.emit_confidence_caveats,
    };
    let params_json = serde_json::to_value(&params)
        .map_err(|error| anyhow!("local-ready bam.contamination params must serialize: {error}"))?;
    crate::tool_policy::enforce(stage, tool_id.as_str(), Some(&params_json), Some(&reference_abs))?;

    let mut plan = crate::tool_adapters::stages_adna::contamination::plan(
        &tool_spec,
        &config.bam,
        Some(config.bai.as_path()),
        Some(config.reference_fasta.as_path()),
        &out_dir,
        &params,
    )?;
    let params = plan.params.as_object_mut().ok_or_else(|| {
        anyhow!("bam.contamination local-ready plan params must be a JSON object")
    })?;
    params.insert("sample_id".to_string(), serde_json::json!(config.sample_id));
    params.insert("tool".to_string(), serde_json::json!(tool_id.as_str()));

    Ok(plan)
}

/// # Errors
/// Returns an error if the governed local-ready config or runtime profile cannot be read, the
/// configured BAM/reference/panel/tool tuple is invalid, or the haplogroups plan cannot be built.
#[cfg(feature = "bam_downstream")]
pub fn local_haplogroups_plan(repo_root: &Path) -> Result<bijux_dna_stage_contract::StagePlanV1> {
    let config = load_local_haplogroups_plan_config(repo_root)?;
    if config.schema_version != "bijux.bench.bam.local_haplogroups.v1" {
        return Err(anyhow!(
            "unexpected local-ready bam.haplogroups schema_version `{}`",
            config.schema_version
        ));
    }

    let local_profile = load_local_runtime_profile(repo_root)?;
    let stage = BamStage::Haplogroups;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-ready tool_id `{}`: {error}", config.tool_id))?;

    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-ready bam.haplogroups tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let bam_abs = repo_root.join(&config.bam);
    ensure_required_file(&bam_abs, "local-ready bam.haplogroups bam")?;
    let bai_abs = repo_root.join(&config.bai);
    ensure_required_file(&bai_abs, "local-ready bam.haplogroups bai")?;
    let reference_abs = repo_root.join(&config.reference_fasta);
    ensure_required_file(&reference_abs, "local-ready bam.haplogroups reference FASTA")?;
    let reference_panel_abs = repo_root.join(&config.reference_panel);
    ensure_required_file(
        &reference_panel_abs,
        "local-ready bam.haplogroups reference panel",
    )?;

    let mut tool_spec = load_bam_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_local_profile_defaults(&mut tool_spec, config.threads, &local_profile);
    let out_dir =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_HAPLOGROUPS_OUTPUT_DIR));
    let params = HaplogroupEffectiveParams {
        reference_panel: config.reference_panel.display().to_string(),
        reference_build: config.reference_build,
        min_coverage: config.min_coverage,
        population_scope: Some(config.population_scope),
        refuse_without_population_context: config.refuse_without_population_context,
    };
    let params_json = serde_json::to_value(&params)
        .map_err(|error| anyhow!("local-ready bam.haplogroups params must serialize: {error}"))?;
    crate::tool_policy::enforce(stage, tool_id.as_str(), Some(&params_json), Some(&reference_abs))?;

    let mut plan = crate::tool_adapters::stages_downstream::haplogroups::plan(
        &tool_spec,
        &config.bam,
        Some(config.bai.as_path()),
        &out_dir,
        &params,
    )?;
    push_required_input(
        &mut plan,
        ArtifactId::from_static("reference"),
        &config.reference_fasta,
        ArtifactRole::Reference,
    );
    push_required_input(
        &mut plan,
        ArtifactId::from_static("reference_panel"),
        &config.reference_panel,
        ArtifactRole::Reference,
    );

    let params = plan.params.as_object_mut().ok_or_else(|| {
        anyhow!("bam.haplogroups local-ready plan params must be a JSON object")
    })?;
    params.insert(
        "reference_panel_id".to_string(),
        serde_json::json!(config.reference_panel_id),
    );
    params.insert(
        "reference_fasta".to_string(),
        serde_json::json!(config.reference_fasta),
    );
    params.insert(
        "coverage_gate".to_string(),
        serde_json::json!({ "min_coverage": config.min_coverage }),
    );
    params.insert("sample_id".to_string(), serde_json::json!(config.sample_id));
    params.insert("tool".to_string(), serde_json::json!(tool_id.as_str()));

    Ok(plan)
}

fn push_required_input(
    plan: &mut bijux_dna_stage_contract::StagePlanV1,
    artifact_id: ArtifactId,
    path: &Path,
    role: ArtifactRole,
) {
    if plan.io.inputs.iter().any(|artifact| artifact.name == artifact_id) {
        return;
    }
    plan.io.inputs.push(bijux_dna_stage_contract::ArtifactRef::required(
        artifact_id,
        path.to_path_buf(),
        role,
    ));
}

fn normalize_plan_path(plan: &mut bijux_dna_stage_contract::StagePlanV1, from: &Path, to: &Path) {
    for artifact in &mut plan.io.inputs {
        if artifact.path == from {
            artifact.path = to.to_path_buf();
        }
    }

    if let Some(params) = plan.params.as_object_mut() {
        if params.get("reference") == Some(&serde_json::json!(from)) {
            params.insert("reference".to_string(), serde_json::json!(to));
        }
    }

    let from = from.display().to_string();
    let to = to.display().to_string();
    for entry in &mut plan.command.template {
        if entry.contains(&from) {
            *entry = entry.replace(&from, &to);
        }
    }
}

fn hydrate_local_profile_defaults(
    tool_spec: &mut ToolExecutionSpecV1,
    configured_threads: Option<u32>,
    local_profile: &LocalRuntimeProfile,
) {
    let threads = configured_threads.unwrap_or(local_profile.default_threads).max(1);
    tool_spec.resources.threads = threads;
    tool_spec.resources.mem_gb = local_profile.default_mem_gb.max(1);
    tool_spec.resources.tmp_gb = local_profile.default_mem_gb.max(1);
}

fn load_local_align_plan_config(repo_root: &Path) -> Result<LocalAlignPlanConfig> {
    let path = repo_root.join(LOCAL_ALIGN_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn load_local_contamination_plan_config(repo_root: &Path) -> Result<LocalContaminationPlanConfig> {
    let path = repo_root.join(LOCAL_CONTAMINATION_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

#[cfg(feature = "bam_downstream")]
fn load_local_haplogroups_plan_config(repo_root: &Path) -> Result<LocalHaplogroupsPlanConfig> {
    let path = repo_root.join(LOCAL_HAPLOGROUPS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn load_local_runtime_profile(repo_root: &Path) -> Result<LocalRuntimeProfile> {
    let path = repo_root.join(LOCAL_RUNTIME_PROFILE_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn ensure_align_reference_index_exists(prefix: &Path, tool_id: &str) -> Result<()> {
    match tool_id {
        "bowtie2" => ensure_bowtie2_index_prefix_exists(prefix),
        "bwa" => ensure_bwa_index_exists(prefix),
        _ => Err(anyhow!(
            "local-ready bam.align does not support reference-index validation for tool `{tool_id}`"
        )),
    }
}

fn ensure_bowtie2_index_prefix_exists(prefix: &Path) -> Result<()> {
    let file_name = prefix
        .file_name()
        .ok_or_else(|| anyhow!("local-ready bam.align Bowtie2 index prefix has no file name"))?
        .to_string_lossy()
        .into_owned();
    let required_suffixes = [".1.bt2", ".2.bt2", ".3.bt2", ".4.bt2", ".rev.1.bt2", ".rev.2.bt2"];
    let missing = required_suffixes
        .into_iter()
        .map(|suffix| prefix.with_file_name(format!("{file_name}{suffix}")))
        .find(|path| !path.is_file());
    if let Some(path) = missing {
        return Err(anyhow!(
            "local-ready bam.align Bowtie2 index prefix is incomplete, missing {}",
            path.display()
        ));
    }
    Ok(())
}

fn ensure_bwa_index_exists(reference_fasta: &Path) -> Result<()> {
    let required_suffixes = [".amb", ".ann", ".bwt", ".pac", ".sa"];
    let missing = required_suffixes
        .into_iter()
        .map(|suffix| PathBuf::from(format!("{}{}", reference_fasta.display(), suffix)))
        .find(|path| !path.is_file());
    if let Some(path) = missing {
        return Err(anyhow!(
            "local-ready bam.align BWA index set is incomplete, missing {}",
            path.display()
        ));
    }
    Ok(())
}

fn ensure_required_file(path: &Path, label: &str) -> Result<()> {
    if path.is_file() {
        Ok(())
    } else {
        Err(anyhow!("{label} is missing: {}", path.display()))
    }
}
