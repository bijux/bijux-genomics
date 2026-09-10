//! Governed FASTQ benchmark qc profile reads.
use crate::planner::benchmark::cases::admission::local_smoke_case_output_dir;
use crate::planner::benchmark::cases::hydrate_smoke_threads;
use crate::selection::allowed_tools_for_stage;
use crate::selection::load_fastq_domain_tool_execution_spec;
use crate::selection::select_stats_tools;
use crate::tool_adapters::fastq::profile_reads::plan_stats_with_threads;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_core::prelude::StageId;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_core::prelude::ToolId;
use bijux_dna_domain_fastq::stages::ids::STAGE_PROFILE_READS;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

const LOCAL_PROFILE_READS_CONFIG_PATH: &str = "configs/bench/local/fastq-profile-reads.toml";

const DEFAULT_LOCAL_PROFILE_READS_OUTPUT_DIR: &str =
    "artifacts/runs/bench/local-smoke/fastq.profile_reads";

#[derive(Debug, Clone)]
pub struct LocalProfileReadsSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalProfileReadsSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalProfileReadsSmokeCase>,
}

#[derive(Debug, Clone, Deserialize)]
struct LocalProfileReadsSmokeCase {
    sample_id: String,
    r1: PathBuf,
    #[serde(default)]
    r2: Option<PathBuf>,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_profile_reads_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalProfileReadsSmokeCasePlan>> {
    let config = load_local_profile_reads_smoke_config(repo_root)?;
    ensure_unique_profile_reads_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_PROFILE_READS.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools = select_stats_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.profile_reads tool selection normalized unexpectedly: {normalized_tools:?}"
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_PROFILE_READS_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| build_local_profile_reads_smoke_case(repo_root, &tool_spec, &output_root, case))
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or one of the admitted tool plans cannot be built for output-contract proof coverage.
pub fn local_profile_reads_output_contract_plans(
    repo_root: &Path,
) -> Result<Vec<LocalProfileReadsSmokeCasePlan>> {
    let config = load_local_profile_reads_smoke_config(repo_root)?;
    ensure_unique_profile_reads_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_PROFILE_READS.as_str().to_string());
    let mut tool_ids = allowed_tools_for_stage(&stage_id);
    tool_ids.sort_by(|left, right| left.as_str().cmp(right.as_str()));
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_PROFILE_READS_OUTPUT_DIR));
    let cases = config.cases;

    let mut plans = Vec::new();
    for tool_id in tool_ids {
        let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
        hydrate_smoke_threads(&mut tool_spec, config.threads);
        let mut matched_cases = 0usize;
        for case in &cases {
            if !bijux_dna_domain_fastq::tool_supports_input_layout(
                &stage_id,
                &tool_spec.tool_id,
                case.r2.is_some(),
            ) {
                continue;
            }
            matched_cases += 1;
            plans.push(build_local_profile_reads_smoke_case(
                repo_root,
                &tool_spec,
                &output_root,
                case.clone(),
            )?);
        }
        if matched_cases == 0 {
            return Err(anyhow!(
                "local-smoke fastq.profile_reads tool_id `{}` has no governed smoke case for its admitted input layouts",
                tool_spec.tool_id.as_str()
            ));
        }
    }
    Ok(plans)
}

fn build_local_profile_reads_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalProfileReadsSmokeCase,
) -> Result<LocalProfileReadsSmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.profile_reads r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    if let Some(r2) = case.r2.as_ref() {
        let r2_abs = repo_root.join(r2);
        if !r2_abs.is_file() {
            return Err(anyhow!(
                "local-smoke fastq.profile_reads r2 fixture is missing: {}",
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
    let plan = plan_stats_with_threads(
        tool_spec,
        &case.r1,
        case.r2.as_deref(),
        &out_dir,
        Some(tool_spec.resources.threads.max(1)),
    )?;

    Ok(LocalProfileReadsSmokeCasePlan { sample_id: case.sample_id, r1: case.r1, r2: case.r2, plan })
}

fn ensure_unique_profile_reads_sample_ids(cases: &[LocalProfileReadsSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke fastq.profile_reads sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.profile_reads sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn load_local_profile_reads_smoke_config(repo_root: &Path) -> Result<LocalProfileReadsSmokeConfig> {
    let path = repo_root.join(LOCAL_PROFILE_READS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalProfileReadsSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_profile_reads.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.profile_reads schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.profile_reads must declare at least one governed case"
        ));
    }
    Ok(config)
}
