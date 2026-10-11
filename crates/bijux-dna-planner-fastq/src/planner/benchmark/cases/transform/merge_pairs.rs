//! Governed FASTQ benchmark transform merge pairs.
use crate::planner::benchmark::cases::admission::admit_unique_case_samples;
use crate::planner::benchmark::cases::admission::local_smoke_case_output_dir;
use crate::planner::benchmark::cases::hydrate_smoke_threads;
use crate::selection::allowed_tools_for_stage;
use crate::selection::load_fastq_domain_tool_execution_spec;
use crate::selection::select_merge_tools;
use crate::tool_adapters::fastq::merge_pairs::plan_merge_with_options;
use crate::tool_adapters::fastq::merge_pairs::MergePlanOptions;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_core::prelude::StageId;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_core::prelude::ToolId;
use bijux_dna_domain_fastq::stages::ids::STAGE_MERGE_PAIRS;
use serde::Deserialize;
use std::path::Path;
use std::path::PathBuf;

const LOCAL_MERGE_PAIRS_CONFIG_PATH: &str = "configs/bench/local/fastq-merge-pairs.toml";

const DEFAULT_LOCAL_MERGE_PAIRS_OUTPUT_DIR: &str =
    "artifacts/runs/bench/local-smoke/fastq.merge_pairs";

#[derive(Debug, Clone)]
pub struct LocalMergePairsSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: PathBuf,
    pub merge_overlap: u32,
    pub min_length: u32,
    pub unmerged_read_policy: bijux_dna_domain_fastq::params::merge::UnmergedReadPolicy,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalMergePairsSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    merge_overlap: Option<u32>,
    #[serde(default)]
    min_length: Option<u32>,
    #[serde(default)]
    unmerged_read_policy: Option<String>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalMergePairsSmokeCase>,
}

#[derive(Debug, Clone, Deserialize)]
struct LocalMergePairsSmokeCase {
    sample_id: String,
    r1: PathBuf,
    r2: PathBuf,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_merge_pairs_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalMergePairsSmokeCasePlan>> {
    build_local_merge_pairs_plans(repo_root, false)
}

/// Build one governed output-contract plan per supported `fastq.merge_pairs` tool and case.
///
/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or any supported tool cannot plan a compatible governed smoke case.
pub fn local_merge_pairs_output_contract_plans(
    repo_root: &Path,
) -> Result<Vec<LocalMergePairsSmokeCasePlan>> {
    build_local_merge_pairs_plans(repo_root, true)
}

fn build_local_merge_pairs_plans(
    repo_root: &Path,
    include_all_supported_tools: bool,
) -> Result<Vec<LocalMergePairsSmokeCasePlan>> {
    let config = load_local_merge_pairs_smoke_config(repo_root)?;
    admit_unique_case_samples(
        "fastq.merge_pairs",
        config.cases.iter().map(|case| case.sample_id.as_str()),
    )?;

    let stage_id = StageId::new(STAGE_MERGE_PAIRS.as_str().to_string());
    let mut tool_ids = if include_all_supported_tools {
        allowed_tools_for_stage(&stage_id)
    } else {
        let tool_id = ToolId::try_from(config.tool_id.as_str()).map_err(|error| {
            anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id)
        })?;
        let normalized_tools = select_merge_tools(std::slice::from_ref(&config.tool_id))?;
        if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
            return Err(anyhow!(
                "local-smoke fastq.merge_pairs tool selection normalized unexpectedly: {normalized_tools:?}"
            ));
        }
        vec![tool_id]
    };
    tool_ids.sort_by(|left, right| left.as_str().cmp(right.as_str()));

    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_MERGE_PAIRS_OUTPUT_DIR));
    let merge_overlap = config.merge_overlap.unwrap_or(10).max(1);
    let min_length = config.min_length.unwrap_or(30).max(1);
    let unmerged_read_policy =
        resolve_local_merge_pairs_unmerged_read_policy(config.unmerged_read_policy.as_deref())?;
    let mut plans = Vec::new();
    for tool_id in tool_ids {
        let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
        hydrate_smoke_threads(&mut tool_spec, config.threads);
        let tool_id = tool_spec.tool_id.as_str();
        let plan_options = MergePlanOptions {
            threads: Some(tool_spec.resources.threads.max(1)),
            merge_overlap: (!matches!(tool_id, "leehom" | "seqprep")).then_some(merge_overlap),
            min_length: matches!(tool_id, "adapterremoval" | "pear" | "vsearch" | "bbmerge")
                .then_some(min_length),
            unmerged_read_policy: unmerged_read_policy.clone(),
        };
        for case in &config.cases {
            plans.push(build_local_merge_pairs_smoke_case(
                repo_root,
                &tool_spec,
                &plan_options,
                &output_root,
                case.clone(),
            )?);
        }
    }
    Ok(plans)
}

fn build_local_merge_pairs_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    options: &MergePlanOptions,
    output_root: &Path,
    case: LocalMergePairsSmokeCase,
) -> Result<LocalMergePairsSmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.merge_pairs r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    let r2_abs = repo_root.join(&case.r2);
    if !r2_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.merge_pairs r2 fixture is missing: {}",
            r2_abs.display()
        ));
    }

    let out_dir = local_smoke_case_output_dir(
        repo_root,
        output_root,
        &case.sample_id,
        tool_spec.tool_id.as_str(),
    )?;
    let plan = plan_merge_with_options(tool_spec, &case.r1, &case.r2, &out_dir, options)?;

    Ok(LocalMergePairsSmokeCasePlan {
        sample_id: case.sample_id,
        r1: case.r1,
        r2: case.r2,
        merge_overlap: options.merge_overlap.unwrap_or(10),
        min_length: options.min_length.unwrap_or(30),
        unmerged_read_policy: options.unmerged_read_policy.clone(),
        plan,
    })
}

fn load_local_merge_pairs_smoke_config(repo_root: &Path) -> Result<LocalMergePairsSmokeConfig> {
    let path = repo_root.join(LOCAL_MERGE_PAIRS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalMergePairsSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_merge_pairs.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.merge_pairs schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.merge_pairs must declare at least one governed case"
        ));
    }
    Ok(config)
}

fn resolve_local_merge_pairs_unmerged_read_policy(
    raw: Option<&str>,
) -> Result<bijux_dna_domain_fastq::params::merge::UnmergedReadPolicy> {
    match raw.unwrap_or("emit_unmerged_pairs") {
        "emit_unmerged_pairs" => {
            Ok(bijux_dna_domain_fastq::params::merge::UnmergedReadPolicy::EmitUnmergedPairs)
        }
        "omit_unmerged_pairs" => {
            Ok(bijux_dna_domain_fastq::params::merge::UnmergedReadPolicy::OmitUnmergedPairs)
        }
        other => {
            Err(anyhow!("unsupported local-smoke fastq.merge_pairs unmerged_read_policy `{other}`"))
        }
    }
}
