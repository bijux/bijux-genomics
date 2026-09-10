//! Governed FASTQ benchmark transform trim reads.
use crate::planner::benchmark::cases::admission::local_smoke_case_output_dir;
use crate::planner::benchmark::cases::hydrate_smoke_threads;
use crate::selection::allowed_tools_for_stage;
use crate::selection::load_fastq_domain_tool_execution_spec;
use crate::selection::select_trim_tools;
use crate::tool_adapters::fastq::trim_reads::plan_with_options as plan_trim_reads_with_options;
use crate::tool_adapters::fastq::trim_reads::TrimPlanOptions;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_core::prelude::StageId;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_core::prelude::ToolId;
use bijux_dna_domain_fastq::banks::DEFAULT_ADAPTER_PRESET;
use bijux_dna_domain_fastq::stages::ids::STAGE_TRIM_READS;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

const LOCAL_TRIM_READS_CONFIG_PATH: &str = "configs/bench/local/fastq-trim-reads.toml";

const DEFAULT_LOCAL_TRIM_READS_OUTPUT_DIR: &str =
    "artifacts/runs/bench/local-smoke/fastq.trim_reads";

#[derive(Debug, Clone)]
pub struct LocalTrimReadsSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub min_length: u32,
    pub quality_cutoff: Option<u32>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalTrimReadsSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    supplemental_tool_ids: Vec<String>,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    min_length: Option<u32>,
    #[serde(default)]
    quality_cutoff: Option<u32>,
    #[serde(default)]
    n_policy: Option<String>,
    #[serde(default)]
    adapter_policy: Option<String>,
    #[serde(default)]
    adapter_preset: Option<String>,
    #[serde(default)]
    polyx_policy: Option<String>,
    #[serde(default)]
    contaminant_policy: Option<String>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalTrimReadsSmokeCase>,
}

#[derive(Debug, Clone, Deserialize)]
struct LocalTrimReadsSmokeCase {
    sample_id: String,
    r1: PathBuf,
    #[serde(default)]
    r2: Option<PathBuf>,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_trim_reads_smoke_plans(repo_root: &Path) -> Result<Vec<LocalTrimReadsSmokeCasePlan>> {
    let config = load_local_trim_reads_smoke_config(repo_root)?;
    ensure_unique_trim_reads_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_TRIM_READS.as_str().to_string());
    let requested_tool_ids = std::iter::once(config.tool_id.clone())
        .chain(config.supplemental_tool_ids.iter().cloned())
        .collect::<Vec<_>>();
    let normalized_tools = select_trim_tools(&requested_tool_ids, false)?;
    if normalized_tools.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.trim_reads must retain at least one admitted trim tool"
        ));
    }

    let adapter_policy = config.adapter_policy.clone().unwrap_or_else(|| "none".to_string());
    let adapter_bank = if matches!(adapter_policy.as_str(), "bank" | "ancient_strict") {
        Some(load_local_trim_reads_adapter_bank_context(
            repo_root,
            config.adapter_preset.as_deref().unwrap_or(DEFAULT_ADAPTER_PRESET),
        )?)
    } else {
        None
    };

    let mut tool_specs = Vec::new();
    for normalized_tool_id in normalized_tools {
        let tool_id = ToolId::try_from(normalized_tool_id.as_str()).map_err(|error| {
            anyhow!("invalid normalized local-smoke tool_id `{normalized_tool_id}`: {error}")
        })?;
        let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
        hydrate_smoke_threads(&mut tool_spec, config.threads);
        tool_specs.push(tool_spec);
    }
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_TRIM_READS_OUTPUT_DIR));
    let plan_options = TrimPlanOptions {
        threads: Some(config.threads.unwrap_or(1).max(1)),
        min_length: Some(config.min_length.unwrap_or(30).max(1)),
        quality_cutoff: config.quality_cutoff,
        n_policy: config.n_policy,
        adapter_policy: Some(adapter_policy),
        polyx_policy: config.polyx_policy,
        contaminant_policy: config.contaminant_policy,
    };

    let cases = config.cases;
    let mut plans = Vec::new();
    for tool_spec in &tool_specs {
        let mut tool_options = plan_options.clone();
        let tool_adapter_bank = if tool_spec.tool_id.as_str() == "bbduk"
            && matches!(tool_options.adapter_policy.as_deref(), Some("bank" | "ancient_strict"))
        {
            tool_options.adapter_policy = Some("none".to_string());
            tool_options.quality_cutoff = None;
            None
        } else {
            adapter_bank.as_ref()
        };
        for case in &cases {
            if tool_spec.tool_id.as_str() == "bbduk" && case.r2.is_some() {
                continue;
            }
            plans.push(build_local_trim_reads_smoke_case(
                repo_root,
                tool_spec,
                tool_adapter_bank,
                &tool_options,
                &output_root,
                case.clone(),
            )?);
        }
    }
    Ok(plans)
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or one of the admitted tool plans cannot be built for output-contract proof coverage.
pub fn local_trim_reads_output_contract_plans(
    repo_root: &Path,
) -> Result<Vec<LocalTrimReadsSmokeCasePlan>> {
    let config = load_local_trim_reads_smoke_config(repo_root)?;
    ensure_unique_trim_reads_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_TRIM_READS.as_str().to_string());
    let mut tool_ids = allowed_tools_for_stage(&stage_id);
    tool_ids.sort_by(|left, right| left.as_str().cmp(right.as_str()));
    let adapter_policy = config.adapter_policy.clone().unwrap_or_else(|| "none".to_string());
    let adapter_bank = if matches!(adapter_policy.as_str(), "bank" | "ancient_strict") {
        Some(load_local_trim_reads_adapter_bank_context(
            repo_root,
            config.adapter_preset.as_deref().unwrap_or(DEFAULT_ADAPTER_PRESET),
        )?)
    } else {
        None
    };
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_TRIM_READS_OUTPUT_DIR));
    let cases = config.cases;
    let base_options = TrimPlanOptions {
        threads: Some(config.threads.unwrap_or(1).max(1)),
        min_length: Some(config.min_length.unwrap_or(30).max(1)),
        quality_cutoff: config.quality_cutoff,
        n_policy: config.n_policy,
        adapter_policy: Some(adapter_policy),
        polyx_policy: config.polyx_policy,
        contaminant_policy: config.contaminant_policy,
    };

    let mut plans = Vec::new();
    for tool_id in tool_ids {
        let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
        hydrate_smoke_threads(&mut tool_spec, config.threads);
        let mut tool_options = base_options.clone();
        let tool_adapter_bank = if tool_spec.tool_id.as_str() == "bbduk"
            && matches!(tool_options.adapter_policy.as_deref(), Some("bank" | "ancient_strict"))
        {
            tool_options.adapter_policy = Some("none".to_string());
            tool_options.quality_cutoff = None;
            None
        } else if matches!(
            tool_spec.tool_id.as_str(),
            "trimmomatic" | "prinseq" | "seqkit" | "seqpurge"
        ) && matches!(
            tool_options.adapter_policy.as_deref(),
            Some("bank" | "ancient_strict")
        ) {
            tool_options.adapter_policy = Some("none".to_string());
            None
        } else {
            adapter_bank.as_ref()
        };
        match tool_spec.tool_id.as_str() {
            "seqkit" | "seqpurge" | "seqprep" => {
                tool_options.quality_cutoff = None;
            }
            "fastx_clipper" | "leehom" => {
                tool_options.min_length = None;
                tool_options.quality_cutoff = None;
            }
            _ => {}
        }
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
            plans.push(build_local_trim_reads_smoke_case(
                repo_root,
                &tool_spec,
                tool_adapter_bank,
                &tool_options,
                &output_root,
                case.clone(),
            )?);
        }
        if matched_cases == 0 {
            return Err(anyhow!(
                "local-smoke fastq.trim_reads tool_id `{}` has no governed smoke case for its admitted input layouts",
                tool_spec.tool_id.as_str()
            ));
        }
    }
    Ok(plans)
}

fn build_local_trim_reads_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    adapter_bank: Option<&serde_json::Value>,
    options: &TrimPlanOptions,
    output_root: &Path,
    case: LocalTrimReadsSmokeCase,
) -> Result<LocalTrimReadsSmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.trim_reads r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    if let Some(r2) = case.r2.as_ref() {
        let r2_abs = repo_root.join(r2);
        if !r2_abs.is_file() {
            return Err(anyhow!(
                "local-smoke fastq.trim_reads r2 fixture is missing: {}",
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
    let plan = plan_trim_reads_with_options(
        tool_spec,
        &case.r1,
        case.r2.as_deref(),
        &out_dir,
        adapter_bank,
        None,
        None,
        options,
    )?;

    Ok(LocalTrimReadsSmokeCasePlan {
        sample_id: case.sample_id,
        r1: case.r1,
        r2: case.r2,
        min_length: options.min_length.unwrap_or(30),
        quality_cutoff: options.quality_cutoff,
        plan,
    })
}

fn ensure_unique_trim_reads_sample_ids(cases: &[LocalTrimReadsSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke fastq.trim_reads sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.trim_reads sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn load_local_trim_reads_smoke_config(repo_root: &Path) -> Result<LocalTrimReadsSmokeConfig> {
    let path = repo_root.join(LOCAL_TRIM_READS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalTrimReadsSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_trim_reads.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.trim_reads schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.trim_reads must declare at least one governed case"
        ));
    }
    Ok(config)
}

fn load_local_trim_reads_adapter_bank_context(
    repo_root: &Path,
    preset_name: &str,
) -> Result<serde_json::Value> {
    bijux_dna_domain_fastq::banks::adapter_bank_context_at(
        repo_root,
        Some(preset_name),
        None,
        None,
        &[],
        &[],
    )?
    .context("adapter bank selection did not produce provenance")
}
