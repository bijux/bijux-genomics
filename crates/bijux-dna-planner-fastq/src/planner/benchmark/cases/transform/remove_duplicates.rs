//! Governed FASTQ benchmark transform remove duplicates.
use crate::planner::benchmark::cases::admission::admit_unique_case_samples;
use crate::planner::benchmark::cases::admission::local_smoke_case_output_dir;
use crate::planner::benchmark::cases::hydrate_smoke_threads;
use crate::selection::load_fastq_domain_tool_execution_spec;
use crate::selection::select_remove_duplicates_tools;
use crate::tool_adapters::fastq::remove_duplicates::dedup_mode_from_literal;
use crate::tool_adapters::fastq::remove_duplicates::plan_deduplicate_with_options;
use crate::tool_adapters::fastq::remove_duplicates::RemoveDuplicatesPlanOptions;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_core::prelude::StageId;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_core::prelude::ToolId;
use bijux_dna_domain_fastq::stages::ids::STAGE_REMOVE_DUPLICATES;
use serde::Deserialize;
use std::path::Path;
use std::path::PathBuf;

const LOCAL_REMOVE_DUPLICATES_CONFIG_PATH: &str =
    "configs/bench/local/fastq-remove-duplicates.toml";

const DEFAULT_LOCAL_REMOVE_DUPLICATES_OUTPUT_DIR: &str =
    "artifacts/runs/bench/local-smoke/fastq.remove_duplicates";

#[derive(Debug, Clone)]
pub struct LocalRemoveDuplicatesSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub dedup_mode: bijux_dna_domain_fastq::params::remove_duplicates::DedupMode,
    pub keep_order: bool,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalRemoveDuplicatesSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    dedup_mode: Option<String>,
    #[serde(default)]
    keep_order: Option<bool>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalRemoveDuplicatesSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalRemoveDuplicatesSmokeCase {
    sample_id: String,
    r1: PathBuf,
    #[serde(default)]
    r2: Option<PathBuf>,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_remove_duplicates_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalRemoveDuplicatesSmokeCasePlan>> {
    let config = load_local_remove_duplicates_smoke_config(repo_root)?;
    admit_unique_case_samples(
        "fastq.remove_duplicates",
        config.cases.iter().map(|case| case.sample_id.as_str()),
    )?;

    let stage_id = StageId::new(STAGE_REMOVE_DUPLICATES.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools = select_remove_duplicates_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.remove_duplicates tool selection normalized unexpectedly: {normalized_tools:?}"
        ));
    }
    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let dedup_mode = dedup_mode_from_literal(config.dedup_mode.as_deref().unwrap_or("exact"))?;
    let keep_order = config.keep_order.unwrap_or(true);
    let output_root = config
        .output_dir
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_REMOVE_DUPLICATES_OUTPUT_DIR));
    let plan_options = RemoveDuplicatesPlanOptions {
        dedup_mode: dedup_mode.clone(),
        keep_order,
        threads_override: Some(tool_spec.resources.threads.max(1)),
    };

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_remove_duplicates_smoke_case(
                repo_root,
                &tool_spec,
                &plan_options,
                &output_root,
                case,
            )
        })
        .collect()
}

fn build_local_remove_duplicates_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    options: &RemoveDuplicatesPlanOptions,
    output_root: &Path,
    case: LocalRemoveDuplicatesSmokeCase,
) -> Result<LocalRemoveDuplicatesSmokeCasePlan> {
    let paired_mode = case.r2.is_some();
    let tool_id = tool_spec.tool_id.as_str();
    if !crate::stage_api::tool_supports_input_layout(
        &StageId::new(STAGE_REMOVE_DUPLICATES.as_str().to_string()),
        &tool_spec.tool_id,
        paired_mode,
    ) {
        return Err(anyhow!(
            "local-smoke fastq.remove_duplicates tool_id `{}` does not support governed {} smoke inputs",
            tool_id,
            if paired_mode { "paired-end" } else { "single-end" }
        ));
    }

    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.remove_duplicates r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    if let Some(r2) = case.r2.as_ref() {
        let r2_abs = repo_root.join(r2);
        if !r2_abs.is_file() {
            return Err(anyhow!(
                "local-smoke fastq.remove_duplicates r2 fixture is missing: {}",
                r2_abs.display()
            ));
        }
    }

    let out_dir = local_smoke_case_output_dir(
        repo_root,
        output_root,
        &case.sample_id,
        tool_spec.tool_id.as_str(),
    )?;
    let plan =
        plan_deduplicate_with_options(tool_spec, &case.r1, case.r2.as_deref(), &out_dir, options)?;

    Ok(LocalRemoveDuplicatesSmokeCasePlan {
        sample_id: case.sample_id,
        r1: case.r1,
        r2: case.r2,
        dedup_mode: options.dedup_mode.clone(),
        keep_order: options.keep_order,
        plan,
    })
}

fn load_local_remove_duplicates_smoke_config(
    repo_root: &Path,
) -> Result<LocalRemoveDuplicatesSmokeConfig> {
    let path = repo_root.join(LOCAL_REMOVE_DUPLICATES_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalRemoveDuplicatesSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_remove_duplicates.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.remove_duplicates schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.remove_duplicates must declare at least one governed case"
        ));
    }
    Ok(config)
}
