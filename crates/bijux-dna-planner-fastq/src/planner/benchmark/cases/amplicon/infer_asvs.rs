//! Governed FASTQ benchmark amplicon infer asvs.
use crate::planner::benchmark::cases::admission::local_smoke_case_output_dir;
use crate::planner::benchmark::cases::hydrate_smoke_threads;
use crate::selection::load_fastq_domain_tool_execution_spec;
use crate::selection::select_infer_asvs_tools;
use crate::tool_adapters::fastq::infer_asvs::plan_with_options as plan_infer_asvs_with_options;
use crate::tool_adapters::fastq::infer_asvs::InferAsvsPlanOptions;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_core::prelude::StageId;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_core::prelude::ToolId;
use bijux_dna_domain_fastq::stages::ids::STAGE_INFER_ASVS;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

const LOCAL_INFER_ASVS_CONFIG_PATH: &str = "configs/bench/local/fastq-infer-asvs.toml";

const DEFAULT_LOCAL_INFER_ASVS_OUTPUT_DIR: &str =
    "artifacts/runs/bench/local-smoke/fastq.infer_asvs";

#[derive(Debug, Clone)]
pub struct LocalInferAsvsSmokeCasePlan {
    pub sample_id: String,
    pub reads: PathBuf,
    pub denoising_method: String,
    pub pooling_mode: String,
    pub chimera_policy: String,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalInferAsvsSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    denoising_method: Option<String>,
    #[serde(default)]
    pooling_mode: Option<String>,
    #[serde(default)]
    chimera_policy: Option<String>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalInferAsvsSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalInferAsvsSmokeCase {
    sample_id: String,
    reads: PathBuf,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_infer_asvs_smoke_plans(repo_root: &Path) -> Result<Vec<LocalInferAsvsSmokeCasePlan>> {
    let config = load_local_infer_asvs_smoke_config(repo_root)?;
    ensure_unique_infer_asvs_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_INFER_ASVS.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools = select_infer_asvs_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.infer_asvs tool selection normalized unexpectedly: {normalized_tools:?}"
        ));
    }
    if !crate::stage_api::tool_supports_input_layout(&stage_id, &tool_id, false) {
        return Err(anyhow!(
            "local-smoke fastq.infer_asvs tool_id `{}` does not support governed single-end smoke inputs",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let mut plan_options = InferAsvsPlanOptions::baseline();
    plan_options.threads = Some(tool_spec.resources.threads.max(1));
    if let Some(denoising_method) = config.denoising_method {
        plan_options.denoising_method = denoising_method;
    }
    if let Some(pooling_mode) = config.pooling_mode {
        plan_options.pooling_mode = pooling_mode;
    }
    if let Some(chimera_policy) = config.chimera_policy {
        plan_options.chimera_policy = chimera_policy;
    }
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_INFER_ASVS_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_infer_asvs_smoke_case(
                repo_root,
                &tool_spec,
                &plan_options,
                &output_root,
                case,
            )
        })
        .collect()
}

fn build_local_infer_asvs_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    options: &InferAsvsPlanOptions,
    output_root: &Path,
    case: LocalInferAsvsSmokeCase,
) -> Result<LocalInferAsvsSmokeCasePlan> {
    let reads_abs = repo_root.join(&case.reads);
    if !reads_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.infer_asvs reads fixture is missing: {}",
            reads_abs.display()
        ));
    }

    let out_dir = local_smoke_case_output_dir(
        repo_root,
        output_root,
        &case.sample_id,
        tool_spec.tool_id.as_str(),
    )?;
    let plan = plan_infer_asvs_with_options(tool_spec, &case.reads, None, &out_dir, options)?;

    Ok(LocalInferAsvsSmokeCasePlan {
        sample_id: case.sample_id,
        reads: case.reads,
        denoising_method: options.denoising_method.clone(),
        pooling_mode: options.pooling_mode.clone(),
        chimera_policy: options.chimera_policy.clone(),
        plan,
    })
}

fn ensure_unique_infer_asvs_sample_ids(cases: &[LocalInferAsvsSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke fastq.infer_asvs sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.infer_asvs sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn load_local_infer_asvs_smoke_config(repo_root: &Path) -> Result<LocalInferAsvsSmokeConfig> {
    let path = repo_root.join(LOCAL_INFER_ASVS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalInferAsvsSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_infer_asvs.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.infer_asvs schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.infer_asvs must declare at least one governed case"
        ));
    }
    Ok(config)
}
