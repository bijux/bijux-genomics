//! Governed FASTQ benchmark pre detect adapters.
use crate::planner::benchmark::cases::admission::local_smoke_case_output_dir;
use crate::planner::benchmark::cases::hydrate_smoke_threads;
use crate::selection::load_fastq_domain_tool_execution_spec;
use crate::selection::select_detect_adapters_tools;
use crate::tool_adapters::fastq::detect_adapters::plan_with_options as plan_detect_adapters;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_core::prelude::StageId;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_core::prelude::ToolId;
use bijux_dna_domain_fastq::stages::ids::STAGE_DETECT_ADAPTERS;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

const LOCAL_DETECT_ADAPTERS_CONFIG_PATH: &str = "configs/bench/local/fastq-detect-adapters.toml";

const DEFAULT_LOCAL_DETECT_ADAPTERS_OUTPUT_DIR: &str =
    "artifacts/runs/bench/local-smoke/fastq.detect_adapters";

#[derive(Debug, Clone)]
pub struct LocalDetectAdaptersSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalDetectAdaptersSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalDetectAdaptersSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalDetectAdaptersSmokeCase {
    sample_id: String,
    r1: PathBuf,
    #[serde(default)]
    r2: Option<PathBuf>,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_detect_adapters_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalDetectAdaptersSmokeCasePlan>> {
    let config = load_local_detect_adapters_smoke_config(repo_root)?;
    ensure_unique_detect_adapters_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_DETECT_ADAPTERS.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools = select_detect_adapters_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.detect_adapters tool selection normalized unexpectedly: {normalized_tools:?}"
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root = config
        .output_dir
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_DETECT_ADAPTERS_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_detect_adapters_smoke_case(repo_root, &tool_spec, &output_root, case)
        })
        .collect()
}

fn build_local_detect_adapters_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalDetectAdaptersSmokeCase,
) -> Result<LocalDetectAdaptersSmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.detect_adapters r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    if let Some(r2) = case.r2.as_ref() {
        let r2_abs = repo_root.join(r2);
        if !r2_abs.is_file() {
            return Err(anyhow!(
                "local-smoke fastq.detect_adapters r2 fixture is missing: {}",
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
    let options =
        crate::DetectAdaptersStageParams { threads: Some(tool_spec.resources.threads.max(1)) };
    let plan = plan_detect_adapters(tool_spec, &case.r1, case.r2.as_deref(), &out_dir, &options)?;

    Ok(LocalDetectAdaptersSmokeCasePlan {
        sample_id: case.sample_id,
        r1: case.r1,
        r2: case.r2,
        plan,
    })
}

fn ensure_unique_detect_adapters_sample_ids(cases: &[LocalDetectAdaptersSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke fastq.detect_adapters sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.detect_adapters sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn load_local_detect_adapters_smoke_config(
    repo_root: &Path,
) -> Result<LocalDetectAdaptersSmokeConfig> {
    let path = repo_root.join(LOCAL_DETECT_ADAPTERS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalDetectAdaptersSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_detect_adapters.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.detect_adapters schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.detect_adapters must declare at least one governed case"
        ));
    }
    Ok(config)
}
