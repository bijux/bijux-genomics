//! Governed BAM benchmark qc complexity.
use crate::benchmark::cases::admission::local_smoke_case_output_dir;
use crate::benchmark::cases::hydrate_smoke_threads;
use crate::selection::allowed_tools_for_stage;
use crate::selection::load_bam_domain_tool_planning_spec;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_core::prelude::StageId;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_core::prelude::ToolId;
use bijux_dna_domain_bam::params::ComplexityEffectiveParams;
use bijux_dna_domain_bam::BamStage;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

const LOCAL_COMPLEXITY_CONFIG_PATH: &str = "configs/bench/local/bam-complexity.toml";

const DEFAULT_LOCAL_COMPLEXITY_OUTPUT_DIR: &str = "artifacts/runs/bench/local-smoke/bam.complexity";

#[derive(Debug, Clone)]
pub struct LocalComplexitySmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub min_reads: u64,
    pub projection_points: Vec<u64>,
    pub expected_observed_total_reads: u64,
    pub expected_observed_unique_reads: u64,
    pub expected_estimated_unique_reads: Option<u64>,
    pub expected_insufficient_data_reason: Option<String>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalComplexitySmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalComplexitySmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalComplexitySmokeCase {
    sample_id: String,
    bam: PathBuf,
    min_reads: u64,
    projection_points: Vec<u64>,
    expected_observed_total_reads: u64,
    expected_observed_unique_reads: u64,
    #[serde(default)]
    expected_estimated_unique_reads: Option<u64>,
    #[serde(default)]
    expected_insufficient_data_reason: Option<String>,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.complexity` plans cannot be built.
pub fn local_complexity_smoke_plans(repo_root: &Path) -> Result<Vec<LocalComplexitySmokeCasePlan>> {
    let config = load_local_complexity_smoke_config(repo_root)?;
    ensure_unique_complexity_sample_ids(&config.cases)?;

    let stage = BamStage::Complexity;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.complexity tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_COMPLEXITY_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| build_local_complexity_smoke_case(repo_root, &tool_spec, &output_root, case))
        .collect()
}

fn build_local_complexity_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalComplexitySmokeCase,
) -> Result<LocalComplexitySmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.complexity BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }
    if case.min_reads == 0 {
        return Err(anyhow!(
            "local-smoke bam.complexity case `{}` must declare min_reads greater than zero",
            case.sample_id
        ));
    }
    if case.projection_points.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.complexity case `{}` must declare at least one projection point",
            case.sample_id
        ));
    }
    if case.projection_points.contains(&0) {
        return Err(anyhow!(
            "local-smoke bam.complexity case `{}` must keep projection points greater than zero",
            case.sample_id
        ));
    }
    if case.projection_points.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(anyhow!(
            "local-smoke bam.complexity case `{}` must keep projection points strictly increasing",
            case.sample_id
        ));
    }
    if case.expected_observed_unique_reads > case.expected_observed_total_reads {
        return Err(anyhow!(
            "local-smoke bam.complexity case `{}` cannot declare unique reads greater than observed total reads",
            case.sample_id
        ));
    }
    if case.expected_estimated_unique_reads.is_some()
        == case.expected_insufficient_data_reason.is_some()
    {
        return Err(anyhow!(
            "local-smoke bam.complexity case `{}` must declare exactly one of expected_estimated_unique_reads or expected_insufficient_data_reason",
            case.sample_id
        ));
    }
    if case
        .expected_estimated_unique_reads
        .is_some_and(|value| value < case.expected_observed_unique_reads)
    {
        return Err(anyhow!(
            "local-smoke bam.complexity case `{}` must keep estimated unique reads greater than or equal to observed unique reads",
            case.sample_id
        ));
    }
    if case.expected_insufficient_data_reason.as_deref().is_some_and(str::is_empty) {
        return Err(anyhow!(
            "local-smoke bam.complexity case `{}` must not declare an empty insufficiency reason",
            case.sample_id
        ));
    }

    let params = ComplexityEffectiveParams {
        min_reads: case.min_reads,
        projection_points: case.projection_points.clone(),
    };
    let out_dir = local_smoke_case_output_dir(
        repo_root,
        output_root,
        &case.sample_id,
        tool_spec.tool_id.as_str(),
    )?;
    let plan =
        crate::tool_adapters::bam::complexity::plan(tool_spec, &case.bam, &out_dir, &params)?;

    Ok(LocalComplexitySmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        min_reads: case.min_reads,
        projection_points: case.projection_points,
        expected_observed_total_reads: case.expected_observed_total_reads,
        expected_observed_unique_reads: case.expected_observed_unique_reads,
        expected_estimated_unique_reads: case.expected_estimated_unique_reads,
        expected_insufficient_data_reason: case.expected_insufficient_data_reason,
        plan,
    })
}

fn ensure_unique_complexity_sample_ids(cases: &[LocalComplexitySmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.complexity sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke bam.complexity sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn load_local_complexity_smoke_config(repo_root: &Path) -> Result<LocalComplexitySmokeConfig> {
    let path = repo_root.join(LOCAL_COMPLEXITY_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalComplexitySmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_complexity.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.complexity schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!("local-smoke bam.complexity must declare at least one governed case"));
    }
    Ok(config)
}
