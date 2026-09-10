//! Governed FASTQ benchmark transform trim polyg tails.
use crate::planner::benchmark::cases::admission::local_smoke_case_output_dir;
use crate::planner::benchmark::cases::hydrate_smoke_threads;
use crate::selection::allowed_tools_for_stage;
use crate::selection::load_fastq_domain_tool_execution_spec;
use crate::tool_adapters::fastq::trim_polyg_tails::plan_trim_polyg_tails_with_options;
use crate::tool_adapters::fastq::trim_polyg_tails::TrimPolygPlanOptions;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_core::prelude::StageId;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_core::prelude::ToolId;
use bijux_dna_domain_fastq::stages::ids::STAGE_TRIM_POLYG_TAILS;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

const LOCAL_TRIM_POLYG_TAILS_CONFIG_PATH: &str = "configs/bench/local/fastq-trim-polyg-tails.toml";

const DEFAULT_LOCAL_TRIM_POLYG_TAILS_OUTPUT_DIR: &str =
    "artifacts/runs/bench/local-smoke/fastq.trim_polyg_tails";

#[derive(Debug, Clone)]
pub struct LocalTrimPolygTailsSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub min_polyg_run: u32,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalTrimPolygTailsSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    supplemental_tool_ids: Vec<String>,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    trim_polyg: Option<bool>,
    #[serde(default)]
    min_polyg_run: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalTrimPolygTailsSmokeCase>,
}

#[derive(Debug, Clone, Deserialize)]
struct LocalTrimPolygTailsSmokeCase {
    sample_id: String,
    r1: PathBuf,
    #[serde(default)]
    r2: Option<PathBuf>,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_trim_polyg_tails_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalTrimPolygTailsSmokeCasePlan>> {
    let config = load_local_trim_polyg_tails_smoke_config(repo_root)?;
    ensure_unique_trim_polyg_tails_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_TRIM_POLYG_TAILS.as_str().to_string());
    let requested_tool_ids = std::iter::once(config.tool_id.clone())
        .chain(config.supplemental_tool_ids.iter().cloned())
        .collect::<Vec<_>>();
    let mut tool_specs = Vec::new();
    for raw_tool_id in requested_tool_ids {
        let tool_id = ToolId::try_from(raw_tool_id.as_str())
            .map_err(|error| anyhow!("invalid local-smoke tool_id `{raw_tool_id}`: {error}"))?;
        if !allowed_tools_for_stage(&stage_id).iter().any(|candidate| candidate == &tool_id) {
            return Err(anyhow!(
                "local-smoke fastq.trim_polyg_tails tool_id `{}` is not admitted for {}",
                tool_id.as_str(),
                stage_id.as_str()
            ));
        }
        if tool_specs.iter().any(|tool_spec: &ToolExecutionSpecV1| tool_spec.tool_id == tool_id) {
            continue;
        }
        let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
        hydrate_smoke_threads(&mut tool_spec, config.threads);
        tool_specs.push(tool_spec);
    }
    let output_root = config
        .output_dir
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_TRIM_POLYG_TAILS_OUTPUT_DIR));
    let trim_polyg = config.trim_polyg.unwrap_or(true);
    let min_polyg_run = config.min_polyg_run.unwrap_or(10).max(1);
    let plan_options = TrimPolygPlanOptions {
        threads: Some(config.threads.unwrap_or(1).max(1)),
        trim_polyg,
        min_polyg_run,
    };

    let cases = config.cases;
    let mut plans = Vec::new();
    for tool_spec in &tool_specs {
        for case in &cases {
            plans.push(build_local_trim_polyg_tails_smoke_case(
                repo_root,
                tool_spec,
                &plan_options,
                &output_root,
                case.clone(),
            )?);
        }
    }
    Ok(plans)
}

fn build_local_trim_polyg_tails_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    options: &TrimPolygPlanOptions,
    output_root: &Path,
    case: LocalTrimPolygTailsSmokeCase,
) -> Result<LocalTrimPolygTailsSmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.trim_polyg_tails r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    if let Some(r2) = case.r2.as_ref() {
        let r2_abs = repo_root.join(r2);
        if !r2_abs.is_file() {
            return Err(anyhow!(
                "local-smoke fastq.trim_polyg_tails r2 fixture is missing: {}",
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
    let plan = plan_trim_polyg_tails_with_options(
        tool_spec,
        &case.r1,
        case.r2.as_deref(),
        &out_dir,
        options,
    )?;

    Ok(LocalTrimPolygTailsSmokeCasePlan {
        sample_id: case.sample_id,
        r1: case.r1,
        r2: case.r2,
        min_polyg_run: options.min_polyg_run,
        plan,
    })
}

fn ensure_unique_trim_polyg_tails_sample_ids(cases: &[LocalTrimPolygTailsSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke fastq.trim_polyg_tails sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.trim_polyg_tails sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn load_local_trim_polyg_tails_smoke_config(
    repo_root: &Path,
) -> Result<LocalTrimPolygTailsSmokeConfig> {
    let path = repo_root.join(LOCAL_TRIM_POLYG_TAILS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalTrimPolygTailsSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_trim_polyg_tails.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.trim_polyg_tails schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.trim_polyg_tails must declare at least one governed case"
        ));
    }
    Ok(config)
}
