//! Governed FASTQ benchmark transform correct errors.
use crate::planner::benchmark::cases::admission::local_smoke_case_output_dir;
use crate::planner::benchmark::cases::hydrate_smoke_threads;
use crate::selection::load_fastq_domain_tool_execution_spec;
use crate::selection::select_correct_tools;
use crate::tool_adapters::fastq::correct_errors::plan_correct_with_options;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_core::prelude::StageId;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_core::prelude::ToolId;
use bijux_dna_domain_fastq::params::correct::QualityEncoding;
use bijux_dna_domain_fastq::stages::ids::STAGE_CORRECT_ERRORS;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

const LOCAL_CORRECT_ERRORS_CONFIG_PATH: &str = "configs/bench/local/fastq-correct-errors.toml";

const DEFAULT_LOCAL_CORRECT_ERRORS_OUTPUT_DIR: &str =
    "artifacts/runs/bench/local-smoke/fastq.correct_errors";

#[derive(Debug, Clone)]
pub struct LocalCorrectErrorsSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub quality_encoding: QualityEncoding,
    pub conservative_mode: bool,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalCorrectErrorsSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    quality_encoding: Option<String>,
    #[serde(default)]
    conservative_mode: Option<bool>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalCorrectErrorsSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalCorrectErrorsSmokeCase {
    sample_id: String,
    r1: PathBuf,
    #[serde(default)]
    r2: Option<PathBuf>,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_correct_errors_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalCorrectErrorsSmokeCasePlan>> {
    let config = load_local_correct_errors_smoke_config(repo_root)?;
    ensure_unique_correct_errors_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_CORRECT_ERRORS.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools = select_correct_tools(std::slice::from_ref(&config.tool_id), false)?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.correct_errors tool selection normalized unexpectedly: {normalized_tools:?}"
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let quality_encoding =
        resolve_local_correct_errors_quality_encoding(config.quality_encoding.as_deref())?;
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_CORRECT_ERRORS_OUTPUT_DIR));
    let mut plan_options = crate::CorrectErrorsStageParams::baseline();
    plan_options.threads = Some(tool_spec.resources.threads.max(1));
    plan_options.quality_encoding = quality_encoding.clone();
    plan_options.conservative_mode = config.conservative_mode.unwrap_or(false);

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_correct_errors_smoke_case(
                repo_root,
                &tool_spec,
                &plan_options,
                &output_root,
                case,
            )
        })
        .collect()
}

fn build_local_correct_errors_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    options: &crate::CorrectErrorsStageParams,
    output_root: &Path,
    case: LocalCorrectErrorsSmokeCase,
) -> Result<LocalCorrectErrorsSmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.correct_errors r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    if let Some(r2) = case.r2.as_ref() {
        let r2_abs = repo_root.join(r2);
        if !r2_abs.is_file() {
            return Err(anyhow!(
                "local-smoke fastq.correct_errors r2 fixture is missing: {}",
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
        plan_correct_with_options(tool_spec, &case.r1, case.r2.as_deref(), &out_dir, options)?;

    Ok(LocalCorrectErrorsSmokeCasePlan {
        sample_id: case.sample_id,
        r1: case.r1,
        r2: case.r2,
        quality_encoding: options.quality_encoding.clone(),
        conservative_mode: options.conservative_mode,
        plan,
    })
}

fn ensure_unique_correct_errors_sample_ids(cases: &[LocalCorrectErrorsSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke fastq.correct_errors sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.correct_errors sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn resolve_local_correct_errors_quality_encoding(value: Option<&str>) -> Result<QualityEncoding> {
    match value.unwrap_or("phred33") {
        "phred33" => Ok(QualityEncoding::Phred33),
        "phred64" => Ok(QualityEncoding::Phred64),
        other => {
            Err(anyhow!("unsupported local-smoke fastq.correct_errors quality_encoding `{other}`"))
        }
    }
}

fn load_local_correct_errors_smoke_config(
    repo_root: &Path,
) -> Result<LocalCorrectErrorsSmokeConfig> {
    let path = repo_root.join(LOCAL_CORRECT_ERRORS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalCorrectErrorsSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_correct_errors.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.correct_errors schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.correct_errors must declare at least one governed case"
        ));
    }
    Ok(config)
}
