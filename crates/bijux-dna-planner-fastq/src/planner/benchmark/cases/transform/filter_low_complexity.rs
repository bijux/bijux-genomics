//! Governed FASTQ benchmark transform filter low complexity.
use crate::planner::benchmark::cases::admission::admit_unique_case_samples;
use crate::planner::benchmark::cases::admission::local_smoke_case_output_dir;
use crate::planner::benchmark::cases::hydrate_smoke_threads;
use crate::selection::load_fastq_domain_tool_execution_spec;
use crate::selection::select_filter_low_complexity_tools;
use crate::tool_adapters::fastq::filter_low_complexity::plan_low_complexity;
use crate::tool_adapters::fastq::filter_low_complexity::LowComplexityPlanOptions;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_core::prelude::StageId;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_core::prelude::ToolId;
use bijux_dna_domain_fastq::stages::ids::STAGE_FILTER_LOW_COMPLEXITY;
use serde::Deserialize;
use std::path::Path;
use std::path::PathBuf;

const LOCAL_FILTER_LOW_COMPLEXITY_CONFIG_PATH: &str =
    "configs/bench/local/fastq-filter-low-complexity.toml";

const DEFAULT_LOCAL_FILTER_LOW_COMPLEXITY_OUTPUT_DIR: &str =
    "artifacts/runs/bench/local-smoke/fastq.filter_low_complexity";

#[derive(Debug, Clone)]
pub struct LocalFilterLowComplexitySmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub entropy_threshold: f64,
    pub polyx_threshold: Option<u32>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalFilterLowComplexitySmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    entropy_threshold: Option<f64>,
    #[serde(default)]
    polyx_threshold: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalFilterLowComplexitySmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalFilterLowComplexitySmokeCase {
    sample_id: String,
    r1: PathBuf,
    #[serde(default)]
    r2: Option<PathBuf>,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_filter_low_complexity_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalFilterLowComplexitySmokeCasePlan>> {
    let config = load_local_filter_low_complexity_smoke_config(repo_root)?;
    admit_unique_case_samples(
        "fastq.filter_low_complexity",
        config.cases.iter().map(|case| case.sample_id.as_str()),
    )?;

    let stage_id = StageId::new(STAGE_FILTER_LOW_COMPLEXITY.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools =
        select_filter_low_complexity_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.filter_low_complexity tool selection normalized unexpectedly: {normalized_tools:?}"
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root = config
        .output_dir
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_FILTER_LOW_COMPLEXITY_OUTPUT_DIR));
    let plan_options = LowComplexityPlanOptions {
        entropy_threshold: Some(config.entropy_threshold.unwrap_or(0.5)),
        polyx_threshold: config.polyx_threshold,
    };

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_filter_low_complexity_smoke_case(
                repo_root,
                &tool_spec,
                &plan_options,
                &output_root,
                case,
            )
        })
        .collect()
}

fn build_local_filter_low_complexity_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    plan_options: &LowComplexityPlanOptions,
    output_root: &Path,
    case: LocalFilterLowComplexitySmokeCase,
) -> Result<LocalFilterLowComplexitySmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.filter_low_complexity r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    if let Some(r2) = case.r2.as_ref() {
        let r2_abs = repo_root.join(r2);
        if !r2_abs.is_file() {
            return Err(anyhow!(
                "local-smoke fastq.filter_low_complexity r2 fixture is missing: {}",
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
        plan_low_complexity(tool_spec, &case.r1, case.r2.as_deref(), &out_dir, plan_options)?;

    Ok(LocalFilterLowComplexitySmokeCasePlan {
        sample_id: case.sample_id,
        r1: case.r1,
        r2: case.r2,
        entropy_threshold: plan_options.entropy_threshold.unwrap_or(0.5),
        polyx_threshold: plan_options.polyx_threshold,
        plan,
    })
}

fn load_local_filter_low_complexity_smoke_config(
    repo_root: &Path,
) -> Result<LocalFilterLowComplexitySmokeConfig> {
    let path = repo_root.join(LOCAL_FILTER_LOW_COMPLEXITY_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalFilterLowComplexitySmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_filter_low_complexity.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.filter_low_complexity schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.filter_low_complexity must declare at least one governed case"
        ));
    }
    Ok(config)
}
