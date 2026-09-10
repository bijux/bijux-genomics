//! Governed FASTQ benchmark pre estimate library complexity prealign.
use crate::planner::benchmark::cases::admission::local_smoke_case_output_dir;
use crate::planner::benchmark::cases::hydrate_smoke_threads;
use crate::selection::load_fastq_domain_tool_execution_spec;
use crate::tool_adapters::fastq::estimate_library_complexity_prealign::plan as plan_estimate_library_complexity_prealign;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_core::prelude::StageId;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_core::prelude::ToolId;
use bijux_dna_domain_fastq::stages::ids::STAGE_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

const LOCAL_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN_CONFIG_PATH: &str =
    "configs/bench/local/fastq-estimate-library-complexity-prealign.toml";

const DEFAULT_LOCAL_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN_OUTPUT_DIR: &str =
    "artifacts/runs/bench/local-smoke/fastq.estimate_library_complexity_prealign";

#[derive(Debug, Clone)]
pub struct LocalEstimateLibraryComplexityPrealignSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub kmer_size: u32,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
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
struct LocalEstimateLibraryComplexityPrealignSmokeCase {
    sample_id: String,
    r1: PathBuf,
    #[serde(default)]
    r2: Option<PathBuf>,
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

    let out_dir = local_smoke_case_output_dir(
        repo_root,
        output_root,
        &case.sample_id,
        tool_spec.tool_id.as_str(),
    )?;
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

fn load_local_estimate_library_complexity_prealign_smoke_config(
    repo_root: &Path,
) -> Result<LocalEstimateLibraryComplexityPrealignSmokeConfig> {
    let path = repo_root.join(LOCAL_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalEstimateLibraryComplexityPrealignSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_estimate_library_complexity_prealign.v1" {
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
