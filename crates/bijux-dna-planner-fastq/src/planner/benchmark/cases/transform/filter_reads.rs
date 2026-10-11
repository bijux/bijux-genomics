//! Governed FASTQ benchmark transform filter reads.
use crate::planner::benchmark::cases::admission::admit_unique_case_samples;
use crate::planner::benchmark::cases::admission::local_smoke_case_output_dir;
use crate::planner::benchmark::cases::hydrate_smoke_threads;
use crate::selection::load_fastq_domain_tool_execution_spec;
use crate::selection::select_filter_tools;
use crate::tool_adapters::fastq::filter_reads::plan_filter;
use crate::tool_adapters::fastq::filter_reads::FilterPlanOptions;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_core::prelude::StageId;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_core::prelude::ToolId;
use bijux_dna_domain_fastq::stages::ids::STAGE_FILTER_READS;
use serde::Deserialize;
use std::path::Path;
use std::path::PathBuf;

const LOCAL_FILTER_READS_CONFIG_PATH: &str = "configs/bench/local/fastq-filter-reads.toml";

const DEFAULT_LOCAL_FILTER_READS_OUTPUT_DIR: &str =
    "artifacts/runs/bench/local-smoke/fastq.filter_reads";

#[derive(Debug, Clone)]
pub struct LocalFilterReadsSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub max_n_count: Option<u32>,
    pub low_complexity_threshold: Option<f64>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalFilterReadsSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    max_n: Option<u32>,
    #[serde(default)]
    max_n_fraction: Option<f64>,
    #[serde(default)]
    max_n_count: Option<u32>,
    #[serde(default)]
    low_complexity_threshold: Option<f64>,
    #[serde(default)]
    entropy_threshold: Option<f64>,
    #[serde(default)]
    polyx_policy: Option<String>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalFilterReadsSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalFilterReadsSmokeCase {
    sample_id: String,
    r1: PathBuf,
    #[serde(default)]
    r2: Option<PathBuf>,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_filter_reads_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalFilterReadsSmokeCasePlan>> {
    let config = load_local_filter_reads_smoke_config(repo_root)?;
    admit_unique_case_samples(
        "fastq.filter_reads",
        config.cases.iter().map(|case| case.sample_id.as_str()),
    )?;

    let stage_id = StageId::new(STAGE_FILTER_READS.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools = select_filter_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.filter_reads tool selection normalized unexpectedly: {normalized_tools:?}"
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_FILTER_READS_OUTPUT_DIR));
    let plan_options = FilterPlanOptions {
        threads: Some(tool_spec.resources.threads.max(1)),
        max_n: config.max_n,
        max_n_fraction: config.max_n_fraction,
        max_n_count: config.max_n_count,
        low_complexity_threshold: config.low_complexity_threshold,
        entropy_threshold: config.entropy_threshold,
        kmer_ref: None,
        redundant_filters: Vec::new(),
        polyx_policy: config.polyx_policy.clone(),
    };

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_filter_reads_smoke_case(
                repo_root,
                &tool_spec,
                &plan_options,
                &output_root,
                case,
            )
        })
        .collect()
}

fn build_local_filter_reads_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    plan_options: &FilterPlanOptions,
    output_root: &Path,
    case: LocalFilterReadsSmokeCase,
) -> Result<LocalFilterReadsSmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.filter_reads r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    if let Some(r2) = case.r2.as_ref() {
        let r2_abs = repo_root.join(r2);
        if !r2_abs.is_file() {
            return Err(anyhow!(
                "local-smoke fastq.filter_reads r2 fixture is missing: {}",
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
    let plan = plan_filter(tool_spec, &case.r1, case.r2.as_deref(), &out_dir, plan_options)?;

    Ok(LocalFilterReadsSmokeCasePlan {
        sample_id: case.sample_id,
        r1: case.r1,
        r2: case.r2,
        max_n_count: plan_options.max_n_count.or(plan_options.max_n),
        low_complexity_threshold: plan_options
            .low_complexity_threshold
            .or(plan_options.entropy_threshold),
        plan,
    })
}

fn load_local_filter_reads_smoke_config(repo_root: &Path) -> Result<LocalFilterReadsSmokeConfig> {
    let path = repo_root.join(LOCAL_FILTER_READS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalFilterReadsSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_filter_reads.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.filter_reads schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.filter_reads must declare at least one governed case"
        ));
    }
    Ok(config)
}
