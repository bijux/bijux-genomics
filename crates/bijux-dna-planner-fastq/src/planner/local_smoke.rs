use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::prelude::{StageId, ToolExecutionSpecV1, ToolId};
use bijux_dna_domain_fastq::stages::ids::STAGE_DETECT_ADAPTERS;
use bijux_dna_domain_fastq::stages::ids::STAGE_DETECT_DUPLICATES_PREMERGE;
use bijux_dna_domain_fastq::stages::ids::STAGE_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN;
use bijux_dna_domain_fastq::stages::ids::STAGE_NORMALIZE_PRIMERS;
use bijux_dna_domain_fastq::params::validate::{PairSyncPolicy, ValidationMode};
use bijux_dna_domain_fastq::stages::ids::STAGE_PROFILE_READ_LENGTHS;
use bijux_dna_domain_fastq::stages::ids::STAGE_TRIM_TERMINAL_DAMAGE;
use bijux_dna_domain_fastq::stages::ids::STAGE_VALIDATE_READS;
use serde::Deserialize;

use crate::selection::{
    allowed_tools_for_stage, load_fastq_domain_tool_execution_spec, select_detect_adapters_tools,
    select_normalize_primers_tools, select_profile_read_lengths_tools, select_validate_tools,
};
use crate::tool_adapters::fastq::detect_adapters::plan_with_options as plan_detect_adapters;
use crate::tool_adapters::fastq::detect_duplicates_premerge::plan as plan_detect_duplicates_premerge;
use crate::tool_adapters::fastq::estimate_library_complexity_prealign::plan as plan_estimate_library_complexity_prealign;
use crate::tool_adapters::fastq::normalize_primers::{
    plan_with_options as plan_normalize_primers, NormalizePrimersPlanOptions,
};
use crate::tool_adapters::fastq::profile_read_lengths::plan_with_options as plan_profile_read_lengths;
use crate::tool_adapters::fastq::trim_terminal_damage::{
    plan_trim_terminal_damage_with_options, TrimTerminalDamagePlanOptions,
};
use crate::tool_adapters::fastq::validate_reads::{
    default_plan_options_for_layout, plan_with_options, validation_mode_from_literal,
};

const LOCAL_DETECT_ADAPTERS_CONFIG_PATH: &str = "configs/bench/local/fastq-detect-adapters.toml";
const DEFAULT_LOCAL_DETECT_ADAPTERS_OUTPUT_DIR: &str = "target/local-smoke/fastq.detect_adapters";
const LOCAL_DETECT_DUPLICATES_PREMERGE_CONFIG_PATH: &str =
    "configs/bench/local/fastq-detect-duplicates-premerge.toml";
const DEFAULT_LOCAL_DETECT_DUPLICATES_PREMERGE_OUTPUT_DIR: &str =
    "target/local-smoke/fastq.detect_duplicates_premerge";
const LOCAL_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN_CONFIG_PATH: &str =
    "configs/bench/local/fastq-estimate-library-complexity-prealign.toml";
const DEFAULT_LOCAL_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN_OUTPUT_DIR: &str =
    "target/local-smoke/fastq.estimate_library_complexity_prealign";
const LOCAL_NORMALIZE_PRIMERS_CONFIG_PATH: &str =
    "configs/bench/local/fastq-normalize-primers.toml";
const DEFAULT_LOCAL_NORMALIZE_PRIMERS_OUTPUT_DIR: &str =
    "target/local-smoke/fastq.normalize_primers";
const LOCAL_PROFILE_READ_LENGTHS_CONFIG_PATH: &str =
    "configs/bench/local/fastq-profile-read-lengths.toml";
const DEFAULT_LOCAL_PROFILE_READ_LENGTHS_OUTPUT_DIR: &str =
    "target/local-smoke/fastq.profile_read_lengths";
const LOCAL_TRIM_TERMINAL_DAMAGE_CONFIG_PATH: &str =
    "configs/bench/local/fastq-trim-terminal-damage.toml";
const DEFAULT_LOCAL_TRIM_TERMINAL_DAMAGE_OUTPUT_DIR: &str =
    "target/local-smoke/fastq.trim_terminal_damage";
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
pub struct LocalDetectDuplicatesPremergeSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalEstimateLibraryComplexityPrealignSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub kmer_size: u32,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalNormalizePrimersSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone)]
pub struct LocalTrimTerminalDamageSmokeCasePlan {
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
struct LocalDetectDuplicatesPremergeSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalDetectDuplicatesPremergeSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalEstimateLibraryComplexityPrealignSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    kmer_size: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalEstimateLibraryComplexityPrealignSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalNormalizePrimersSmokeConfig {
    schema_version: String,
    tool_id: String,
    primer_set_id: String,
    marker_id: String,
    primer_fasta: PathBuf,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    orientation_policy: Option<String>,
    #[serde(default)]
    max_mismatch_rate: Option<f64>,
    #[serde(default)]
    min_overlap_bp: Option<u32>,
    #[serde(default)]
    strict_5p_anchor: Option<bool>,
    #[serde(default)]
    allow_iupac_codes: Option<bool>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalNormalizePrimersSmokeCase>,
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

#[derive(Debug, Deserialize)]
struct LocalDetectAdaptersSmokeCase {
    sample_id: String,
    r1: PathBuf,
    #[serde(default)]
    r2: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct LocalDetectDuplicatesPremergeSmokeCase {
    sample_id: String,
    r1: PathBuf,
    #[serde(default)]
    r2: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct LocalEstimateLibraryComplexityPrealignSmokeCase {
    sample_id: String,
    r1: PathBuf,
    #[serde(default)]
    r2: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct LocalNormalizePrimersSmokeCase {
    sample_id: String,
    r1: PathBuf,
    #[serde(default)]
    r2: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct LocalTrimTerminalDamageSmokeCase {
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

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_detect_duplicates_premerge_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalDetectDuplicatesPremergeSmokeCasePlan>> {
    let config = load_local_detect_duplicates_premerge_smoke_config(repo_root)?;
    ensure_unique_detect_duplicates_premerge_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_DETECT_DUPLICATES_PREMERGE.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if tool_id.as_str() != "bijux_dna" {
        return Err(anyhow!(
            "local-smoke fastq.detect_duplicates_premerge currently requires governed native tool_id `bijux_dna`, got `{}`",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root = config.output_dir.unwrap_or_else(|| {
        PathBuf::from(DEFAULT_LOCAL_DETECT_DUPLICATES_PREMERGE_OUTPUT_DIR)
    });

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_detect_duplicates_premerge_smoke_case(
                repo_root,
                &tool_spec,
                &output_root,
                case,
            )
        })
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_estimate_library_complexity_prealign_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalEstimateLibraryComplexityPrealignSmokeCasePlan>> {
    let config = load_local_estimate_library_complexity_prealign_smoke_config(repo_root)?;
    ensure_unique_estimate_library_complexity_prealign_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if tool_id.as_str() != "bijux_dna" {
        return Err(anyhow!(
            "local-smoke fastq.estimate_library_complexity_prealign currently requires governed native tool_id `bijux_dna`, got `{}`",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let kmer_size = config.kmer_size.unwrap_or(31).max(1);
    let output_root = config.output_dir.unwrap_or_else(|| {
        PathBuf::from(DEFAULT_LOCAL_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN_OUTPUT_DIR)
    });

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_estimate_library_complexity_prealign_smoke_case(
                repo_root,
                &tool_spec,
                kmer_size,
                &output_root,
                case,
            )
        })
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_normalize_primers_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalNormalizePrimersSmokeCasePlan>> {
    let config = load_local_normalize_primers_smoke_config(repo_root)?;
    ensure_unique_normalize_primers_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_NORMALIZE_PRIMERS.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools = select_normalize_primers_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.normalize_primers tool selection normalized unexpectedly: {:?}",
            normalized_tools
        ));
    }

    let primer_fasta = repo_root.join(&config.primer_fasta);
    if !primer_fasta.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.normalize_primers primer_fasta is missing: {}",
            primer_fasta.display()
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_NORMALIZE_PRIMERS_OUTPUT_DIR));
    let plan_options = NormalizePrimersPlanOptions {
        primer_set_id: config.primer_set_id,
        marker_id: Some(config.marker_id),
        primer_fasta: Some(config.primer_fasta),
        orientation_policy: config
            .orientation_policy
            .unwrap_or_else(|| "normalize_to_forward_primer".to_string()),
        max_mismatch_rate: config.max_mismatch_rate.unwrap_or(0.10),
        min_overlap_bp: config.min_overlap_bp.unwrap_or(10).max(1),
        strict_5p_anchor: config.strict_5p_anchor.unwrap_or(true),
        allow_iupac_codes: config.allow_iupac_codes.unwrap_or(true),
    };

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_normalize_primers_smoke_case(
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
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_trim_terminal_damage_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalTrimTerminalDamageSmokeCasePlan>> {
    let config = load_local_trim_terminal_damage_smoke_config(repo_root)?;
    ensure_unique_trim_terminal_damage_sample_ids(&config.cases)?;

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
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_TRIM_TERMINAL_DAMAGE_OUTPUT_DIR));
    let damage_mode = config.damage_mode.parse().map_err(|error: String| {
        anyhow!("invalid local-smoke fastq.trim_terminal_damage damage_mode `{}`: {error}", config.damage_mode)
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

fn build_local_detect_duplicates_premerge_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalDetectDuplicatesPremergeSmokeCase,
) -> Result<LocalDetectDuplicatesPremergeSmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.detect_duplicates_premerge r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    if let Some(r2) = case.r2.as_ref() {
        let r2_abs = repo_root.join(r2);
        if !r2_abs.is_file() {
            return Err(anyhow!(
                "local-smoke fastq.detect_duplicates_premerge r2 fixture is missing: {}",
                r2_abs.display()
            ));
        }
    }

    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan =
        plan_detect_duplicates_premerge(tool_spec, &case.r1, case.r2.as_deref(), &out_dir)?;

    Ok(LocalDetectDuplicatesPremergeSmokeCasePlan {
        sample_id: case.sample_id,
        r1: case.r1,
        r2: case.r2,
        plan,
    })
}

fn build_local_estimate_library_complexity_prealign_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    kmer_size: u32,
    output_root: &Path,
    case: LocalEstimateLibraryComplexityPrealignSmokeCase,
) -> Result<LocalEstimateLibraryComplexityPrealignSmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.estimate_library_complexity_prealign r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    if let Some(r2) = case.r2.as_ref() {
        let r2_abs = repo_root.join(r2);
        if !r2_abs.is_file() {
            return Err(anyhow!(
                "local-smoke fastq.estimate_library_complexity_prealign r2 fixture is missing: {}",
                r2_abs.display()
            ));
        }
    }

    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan = plan_estimate_library_complexity_prealign(
        tool_spec,
        &case.r1,
        case.r2.as_deref(),
        &out_dir,
        Some(kmer_size),
    )?;

    Ok(LocalEstimateLibraryComplexityPrealignSmokeCasePlan {
        sample_id: case.sample_id,
        r1: case.r1,
        r2: case.r2,
        kmer_size,
        plan,
    })
}

fn build_local_normalize_primers_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    options: &NormalizePrimersPlanOptions,
    output_root: &Path,
    case: LocalNormalizePrimersSmokeCase,
) -> Result<LocalNormalizePrimersSmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.normalize_primers r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    if let Some(r2) = case.r2.as_ref() {
        let r2_abs = repo_root.join(r2);
        if !r2_abs.is_file() {
            return Err(anyhow!(
                "local-smoke fastq.normalize_primers r2 fixture is missing: {}",
                r2_abs.display()
            ));
        }
    }

    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan = plan_normalize_primers(
        tool_spec,
        &case.r1,
        case.r2.as_deref(),
        &out_dir,
        options,
    )?;

    Ok(LocalNormalizePrimersSmokeCasePlan {
        sample_id: case.sample_id,
        r1: case.r1,
        r2: case.r2,
        plan,
    })
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

    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
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

fn ensure_unique_estimate_library_complexity_prealign_sample_ids(
    cases: &[LocalEstimateLibraryComplexityPrealignSmokeCase],
) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "local-smoke fastq.estimate_library_complexity_prealign sample_id must not be empty"
            ));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.estimate_library_complexity_prealign sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_normalize_primers_sample_ids(
    cases: &[LocalNormalizePrimersSmokeCase],
) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "local-smoke fastq.normalize_primers sample_id must not be empty"
            ));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.normalize_primers sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_trim_terminal_damage_sample_ids(
    cases: &[LocalTrimTerminalDamageSmokeCase],
) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "local-smoke fastq.trim_terminal_damage sample_id must not be empty"
            ));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.trim_terminal_damage sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_detect_duplicates_premerge_sample_ids(
    cases: &[LocalDetectDuplicatesPremergeSmokeCase],
) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "local-smoke fastq.detect_duplicates_premerge sample_id must not be empty"
            ));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.detect_duplicates_premerge sample_id `{}`",
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

fn load_local_estimate_library_complexity_prealign_smoke_config(
    repo_root: &Path,
) -> Result<LocalEstimateLibraryComplexityPrealignSmokeConfig> {
    let path = repo_root.join(LOCAL_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalEstimateLibraryComplexityPrealignSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version
        != "bijux.bench.fastq.local_estimate_library_complexity_prealign.v1"
    {
        return Err(anyhow!(
            "unsupported local-smoke fastq.estimate_library_complexity_prealign schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.estimate_library_complexity_prealign must declare at least one governed case"
        ));
    }
    Ok(config)
}

fn load_local_normalize_primers_smoke_config(
    repo_root: &Path,
) -> Result<LocalNormalizePrimersSmokeConfig> {
    let path = repo_root.join(LOCAL_NORMALIZE_PRIMERS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalNormalizePrimersSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_normalize_primers.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.normalize_primers schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.normalize_primers must declare at least one governed case"
        ));
    }
    Ok(config)
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

fn load_local_detect_duplicates_premerge_smoke_config(
    repo_root: &Path,
) -> Result<LocalDetectDuplicatesPremergeSmokeConfig> {
    let path = repo_root.join(LOCAL_DETECT_DUPLICATES_PREMERGE_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalDetectDuplicatesPremergeSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_detect_duplicates_premerge.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.detect_duplicates_premerge schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.detect_duplicates_premerge must declare at least one governed case"
        ));
    }
    Ok(config)
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
