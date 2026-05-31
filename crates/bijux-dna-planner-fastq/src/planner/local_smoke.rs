use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::prelude::{StageId, ToolExecutionSpecV1, ToolId};
use bijux_dna_domain_fastq::stages::ids::STAGE_DETECT_ADAPTERS;
use bijux_dna_domain_fastq::params::validate::{PairSyncPolicy, ValidationMode};
use bijux_dna_domain_fastq::stages::ids::STAGE_PROFILE_READ_LENGTHS;
use bijux_dna_domain_fastq::stages::ids::STAGE_VALIDATE_READS;
use serde::Deserialize;

use crate::selection::{
    load_fastq_domain_tool_execution_spec, select_detect_adapters_tools,
    select_profile_read_lengths_tools, select_validate_tools,
};
use crate::tool_adapters::fastq::detect_adapters::plan_with_options as plan_detect_adapters;
use crate::tool_adapters::fastq::profile_read_lengths::plan_with_options as plan_profile_read_lengths;
use crate::tool_adapters::fastq::validate_reads::{
    default_plan_options_for_layout, plan_with_options, validation_mode_from_literal,
};

const LOCAL_DETECT_ADAPTERS_CONFIG_PATH: &str = "configs/bench/local/fastq-detect-adapters.toml";
const DEFAULT_LOCAL_DETECT_ADAPTERS_OUTPUT_DIR: &str = "target/local-smoke/fastq.detect_adapters";
const LOCAL_PROFILE_READ_LENGTHS_CONFIG_PATH: &str =
    "configs/bench/local/fastq-profile-read-lengths.toml";
const DEFAULT_LOCAL_PROFILE_READ_LENGTHS_OUTPUT_DIR: &str =
    "target/local-smoke/fastq.profile_read_lengths";
const LOCAL_VALIDATE_READS_CONFIG_PATH: &str = "configs/bench/local/fastq-validate-reads.toml";
const DEFAULT_LOCAL_VALIDATE_READS_OUTPUT_DIR: &str = "target/local-smoke/fastq.validate_reads";

#[derive(Debug, Clone)]
pub struct LocalProfileReadLengthsSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub histogram_bins: u32,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalDetectAdaptersSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalValidateReadsSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub validation_mode: ValidationMode,
    pub pair_sync_policy: PairSyncPolicy,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalValidateReadsSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    validation_mode: Option<String>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalValidateReadsSmokeCase>,
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

#[derive(Debug, Deserialize)]
struct LocalProfileReadLengthsSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    histogram_bins: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalProfileReadLengthsSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalProfileReadLengthsSmokeCase {
    sample_id: String,
    r1: PathBuf,
    #[serde(default)]
    r2: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct LocalValidateReadsSmokeCase {
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
            "local-smoke fastq.detect_adapters tool selection normalized unexpectedly: {:?}",
            normalized_tools
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_DETECT_ADAPTERS_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| build_local_detect_adapters_smoke_case(repo_root, &tool_spec, &output_root, case))
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

    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let mut options = crate::DetectAdaptersStageParams::default();
    options.threads = Some(tool_spec.resources.threads.max(1));
    let plan = plan_detect_adapters(tool_spec, &case.r1, case.r2.as_deref(), &out_dir, &options)?;

    Ok(LocalDetectAdaptersSmokeCasePlan { sample_id: case.sample_id, r1: case.r1, r2: case.r2, plan })
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_validate_reads_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalValidateReadsSmokeCasePlan>> {
    let config = load_local_validate_reads_smoke_config(repo_root)?;
    ensure_unique_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_VALIDATE_READS.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools = select_validate_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.validate_reads tool selection normalized unexpectedly: {:?}",
            normalized_tools
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let validation_mode =
        validation_mode_from_literal(config.validation_mode.as_deref().unwrap_or("strict"))?;
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_VALIDATE_READS_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_validate_reads_smoke_case(
                repo_root,
                &tool_spec,
                &validation_mode,
                &output_root,
                case,
            )
        })
        .collect()
}

fn build_local_validate_reads_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    validation_mode: &ValidationMode,
    output_root: &Path,
    case: LocalValidateReadsSmokeCase,
) -> Result<LocalValidateReadsSmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.validate_reads r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    if let Some(r2) = case.r2.as_ref() {
        let r2_abs = repo_root.join(r2);
        if !r2_abs.is_file() {
            return Err(anyhow!(
                "local-smoke fastq.validate_reads r2 fixture is missing: {}",
                r2_abs.display()
            ));
        }
    }

    let mut options = default_plan_options_for_layout(case.r2.as_deref());
    options.threads = Some(tool_spec.resources.threads.max(1));
    options.validation_mode = validation_mode.clone();
    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan = plan_with_options(tool_spec, &case.r1, case.r2.as_deref(), &out_dir, &options)?;

    Ok(LocalValidateReadsSmokeCasePlan {
        sample_id: case.sample_id,
        r1: case.r1,
        r2: case.r2,
        validation_mode: options.validation_mode,
        pair_sync_policy: options.pair_sync_policy,
        plan,
    })
}

fn hydrate_smoke_threads(tool_spec: &mut ToolExecutionSpecV1, threads: Option<u32>) {
    if let Some(threads) = threads {
        tool_spec.resources.threads = threads.max(1);
    } else {
        tool_spec.resources.threads = tool_spec.resources.threads.max(1);
    }
}

fn ensure_unique_sample_ids(cases: &[LocalValidateReadsSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke fastq.validate_reads sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.validate_reads sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn load_local_validate_reads_smoke_config(
    repo_root: &Path,
) -> Result<LocalValidateReadsSmokeConfig> {
    let path = repo_root.join(LOCAL_VALIDATE_READS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalValidateReadsSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_validate_reads.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.validate_reads schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.validate_reads must declare at least one governed case"
        ));
    }
    Ok(config)
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_profile_read_lengths_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalProfileReadLengthsSmokeCasePlan>> {
    let config = load_local_profile_read_lengths_smoke_config(repo_root)?;
    ensure_unique_profile_read_lengths_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_PROFILE_READ_LENGTHS.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str()).map_err(|error| {
        anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id)
    })?;
    let normalized_tools =
        select_profile_read_lengths_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.profile_read_lengths tool selection normalized unexpectedly: {:?}",
            normalized_tools
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let histogram_bins = config.histogram_bins.unwrap_or(100).max(1);
    let output_root = config.output_dir.unwrap_or_else(|| {
        PathBuf::from(DEFAULT_LOCAL_PROFILE_READ_LENGTHS_OUTPUT_DIR)
    });

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_profile_read_lengths_smoke_case(
                repo_root,
                &tool_spec,
                histogram_bins,
                &output_root,
                case,
            )
        })
        .collect()
}

fn build_local_profile_read_lengths_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    histogram_bins: u32,
    output_root: &Path,
    case: LocalProfileReadLengthsSmokeCase,
) -> Result<LocalProfileReadLengthsSmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.profile_read_lengths r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    if let Some(r2) = case.r2.as_ref() {
        let r2_abs = repo_root.join(r2);
        if !r2_abs.is_file() {
            return Err(anyhow!(
                "local-smoke fastq.profile_read_lengths r2 fixture is missing: {}",
                r2_abs.display()
            ));
        }
    }

    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan = plan_profile_read_lengths(
        tool_spec,
        &case.r1,
        case.r2.as_deref(),
        &out_dir,
        Some(tool_spec.resources.threads.max(1)),
        Some(histogram_bins),
    )?;

    Ok(LocalProfileReadLengthsSmokeCasePlan {
        sample_id: case.sample_id,
        r1: case.r1,
        r2: case.r2,
        histogram_bins,
        plan,
    })
}

fn ensure_unique_profile_read_lengths_sample_ids(
    cases: &[LocalProfileReadLengthsSmokeCase],
) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "local-smoke fastq.profile_read_lengths sample_id must not be empty"
            ));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.profile_read_lengths sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_detect_adapters_sample_ids(cases: &[LocalDetectAdaptersSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "local-smoke fastq.detect_adapters sample_id must not be empty"
            ));
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

fn load_local_profile_read_lengths_smoke_config(
    repo_root: &Path,
) -> Result<LocalProfileReadLengthsSmokeConfig> {
    let path = repo_root.join(LOCAL_PROFILE_READ_LENGTHS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalProfileReadLengthsSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_profile_read_lengths.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.profile_read_lengths schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.profile_read_lengths must declare at least one governed case"
        ));
    }
    Ok(config)
}
