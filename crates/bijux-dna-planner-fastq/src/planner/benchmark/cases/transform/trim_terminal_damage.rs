//! Governed FASTQ benchmark transform trim terminal damage.
use crate::planner::benchmark::cases::admission::admit_unique_case_samples;
use crate::planner::benchmark::cases::admission::local_smoke_case_output_dir;
use crate::planner::benchmark::cases::hydrate_smoke_threads;
use crate::selection::allowed_tools_for_stage;
use crate::selection::load_fastq_domain_tool_execution_spec;
use crate::tool_adapters::fastq::trim_terminal_damage::plan_trim_terminal_damage_with_options;
use crate::tool_adapters::fastq::trim_terminal_damage::TrimTerminalDamagePlanOptions;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_core::prelude::StageId;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_core::prelude::ToolId;
use bijux_dna_domain_fastq::stages::ids::STAGE_TRIM_TERMINAL_DAMAGE;
use serde::Deserialize;
use std::path::Path;
use std::path::PathBuf;

const LOCAL_TRIM_TERMINAL_DAMAGE_CONFIG_PATH: &str =
    "configs/bench/local/fastq-trim-terminal-damage.toml";

const DEFAULT_LOCAL_TRIM_TERMINAL_DAMAGE_OUTPUT_DIR: &str =
    "artifacts/runs/bench/local-smoke/fastq.trim_terminal_damage";

#[derive(Debug, Clone)]
pub struct LocalTrimTerminalDamageSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalTrimTerminalDamageSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    damage_mode: String,
    #[serde(default)]
    execution_policy: Option<String>,
    trim_5p_bases: u32,
    trim_3p_bases: u32,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalTrimTerminalDamageSmokeCase>,
}

#[derive(Debug, Clone, Deserialize)]
struct LocalTrimTerminalDamageSmokeCase {
    sample_id: String,
    r1: PathBuf,
    #[serde(default)]
    r2: Option<PathBuf>,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_trim_terminal_damage_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalTrimTerminalDamageSmokeCasePlan>> {
    let config = load_local_trim_terminal_damage_smoke_config(repo_root)?;
    admit_unique_case_samples(
        "fastq.trim_terminal_damage",
        config.cases.iter().map(|case| case.sample_id.as_str()),
    )?;

    let stage_id = StageId::new(STAGE_TRIM_TERMINAL_DAMAGE.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(&stage_id).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke fastq.trim_terminal_damage tool_id `{}` is not admitted for {}",
            tool_id.as_str(),
            stage_id.as_str()
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root = config
        .output_dir
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_TRIM_TERMINAL_DAMAGE_OUTPUT_DIR));
    let damage_mode = config.damage_mode.parse().map_err(|error: String| {
        anyhow!(
            "invalid local-smoke fastq.trim_terminal_damage damage_mode `{}`: {error}",
            config.damage_mode
        )
    })?;
    let execution_policy =
        bijux_dna_domain_fastq::params::trim::parse_terminal_damage_execution_policy(
            config.execution_policy.as_deref().unwrap_or("policy_derived"),
        )
        .ok_or_else(|| {
            anyhow!(
                "invalid local-smoke fastq.trim_terminal_damage execution_policy `{:?}`",
                config.execution_policy
            )
        })?;
    let plan_options = TrimTerminalDamagePlanOptions {
        threads: Some(tool_spec.resources.threads.max(1)),
        damage_mode,
        execution_policy,
        trim_5p_bases: config.trim_5p_bases,
        trim_3p_bases: config.trim_3p_bases,
    };

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_trim_terminal_damage_smoke_case(
                repo_root,
                &tool_spec,
                &plan_options,
                &output_root,
                case,
            )
        })
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or one of the admitted tool plans cannot be built for output-contract proof coverage.
pub fn local_trim_terminal_damage_output_contract_plans(
    repo_root: &Path,
) -> Result<Vec<LocalTrimTerminalDamageSmokeCasePlan>> {
    let config = load_local_trim_terminal_damage_smoke_config(repo_root)?;
    admit_unique_case_samples(
        "fastq.trim_terminal_damage",
        config.cases.iter().map(|case| case.sample_id.as_str()),
    )?;

    let stage_id = StageId::new(STAGE_TRIM_TERMINAL_DAMAGE.as_str().to_string());
    let mut tool_ids = allowed_tools_for_stage(&stage_id);
    tool_ids.sort_by(|left, right| left.as_str().cmp(right.as_str()));
    let output_root = config
        .output_dir
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_TRIM_TERMINAL_DAMAGE_OUTPUT_DIR));
    let damage_mode = config.damage_mode.parse().map_err(|error: String| {
        anyhow!(
            "invalid local-smoke fastq.trim_terminal_damage damage_mode `{}`: {error}",
            config.damage_mode
        )
    })?;
    let execution_policy =
        bijux_dna_domain_fastq::params::trim::parse_terminal_damage_execution_policy(
            config.execution_policy.as_deref().unwrap_or("policy_derived"),
        )
        .ok_or_else(|| {
            anyhow!(
                "invalid local-smoke fastq.trim_terminal_damage execution_policy `{:?}`",
                config.execution_policy
            )
        })?;
    let cases = config.cases;

    let mut plans = Vec::new();
    for tool_id in tool_ids {
        let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
        hydrate_smoke_threads(&mut tool_spec, config.threads);
        let plan_options = TrimTerminalDamagePlanOptions {
            threads: Some(tool_spec.resources.threads.max(1)),
            damage_mode,
            execution_policy,
            trim_5p_bases: config.trim_5p_bases,
            trim_3p_bases: config.trim_3p_bases,
        };
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
            plans.push(build_local_trim_terminal_damage_smoke_case(
                repo_root,
                &tool_spec,
                &plan_options,
                &output_root,
                case.clone(),
            )?);
        }
        if matched_cases == 0 {
            return Err(anyhow!(
                "local-smoke fastq.trim_terminal_damage tool_id `{}` has no governed smoke case for its admitted input layouts",
                tool_spec.tool_id.as_str()
            ));
        }
    }
    Ok(plans)
}

fn build_local_trim_terminal_damage_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    options: &TrimTerminalDamagePlanOptions,
    output_root: &Path,
    case: LocalTrimTerminalDamageSmokeCase,
) -> Result<LocalTrimTerminalDamageSmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.trim_terminal_damage r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    if let Some(r2) = case.r2.as_ref() {
        let r2_abs = repo_root.join(r2);
        if !r2_abs.is_file() {
            return Err(anyhow!(
                "local-smoke fastq.trim_terminal_damage r2 fixture is missing: {}",
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
    let plan = plan_trim_terminal_damage_with_options(
        tool_spec,
        &case.r1,
        case.r2.as_deref(),
        &out_dir,
        options,
    )?;

    Ok(LocalTrimTerminalDamageSmokeCasePlan {
        sample_id: case.sample_id,
        r1: case.r1,
        r2: case.r2,
        plan,
    })
}

fn load_local_trim_terminal_damage_smoke_config(
    repo_root: &Path,
) -> Result<LocalTrimTerminalDamageSmokeConfig> {
    let path = repo_root.join(LOCAL_TRIM_TERMINAL_DAMAGE_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalTrimTerminalDamageSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_trim_terminal_damage.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.trim_terminal_damage schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.trim_terminal_damage must declare at least one governed case"
        ));
    }
    Ok(config)
}
