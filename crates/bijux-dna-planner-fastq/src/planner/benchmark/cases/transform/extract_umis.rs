//! Governed FASTQ benchmark transform extract umis.
use crate::planner::benchmark::cases::admission::admit_unique_case_samples;
use crate::planner::benchmark::cases::admission::local_smoke_case_output_dir;
use crate::planner::benchmark::cases::hydrate_smoke_threads;
use crate::selection::load_fastq_domain_tool_execution_spec;
use crate::selection::select_umi_tools;
use crate::tool_adapters::fastq::extract_umis::plan_umi_with_options;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_core::prelude::StageId;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_core::prelude::ToolId;
use bijux_dna_domain_fastq::params::umi::UmiFailedExtractionPolicy;
use bijux_dna_domain_fastq::params::umi::UmiReadNameTransform;
use bijux_dna_domain_fastq::stages::ids::STAGE_EXTRACT_UMIS;
use serde::Deserialize;
use std::path::Path;
use std::path::PathBuf;

const LOCAL_EXTRACT_UMIS_CONFIG_PATH: &str = "configs/bench/local/fastq-extract-umis.toml";

const DEFAULT_LOCAL_EXTRACT_UMIS_OUTPUT_DIR: &str =
    "artifacts/runs/bench/local-smoke/fastq.extract_umis";

#[derive(Debug, Clone)]
pub struct LocalExtractUmisSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: PathBuf,
    pub umi_pattern: String,
    pub read_name_transform: UmiReadNameTransform,
    pub failed_extraction_policy: UmiFailedExtractionPolicy,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalExtractUmisSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    umi_pattern: Option<String>,
    #[serde(default)]
    extraction_location: Option<String>,
    #[serde(default)]
    read_name_transform: Option<String>,
    #[serde(default)]
    failed_extraction_policy: Option<String>,
    #[serde(default)]
    grouping_policy: Option<String>,
    #[serde(default)]
    downstream_dedup_policy: Option<String>,
    #[serde(default)]
    downstream_propagation: Option<String>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalExtractUmisSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalExtractUmisSmokeCase {
    sample_id: String,
    r1: PathBuf,
    r2: PathBuf,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_extract_umis_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalExtractUmisSmokeCasePlan>> {
    let config = load_local_extract_umis_smoke_config(repo_root)?;
    admit_unique_case_samples(
        "fastq.extract_umis",
        config.cases.iter().map(|case| case.sample_id.as_str()),
    )?;

    let stage_id = StageId::new(STAGE_EXTRACT_UMIS.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools = select_umi_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.extract_umis tool selection normalized unexpectedly: {normalized_tools:?}"
        ));
    }
    if !crate::stage_api::tool_supports_input_layout(&stage_id, &tool_id, true) {
        return Err(anyhow!(
            "local-smoke fastq.extract_umis tool_id `{}` does not support governed paired-end smoke inputs",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_EXTRACT_UMIS_OUTPUT_DIR));
    let plan_options = crate::ExtractUmisStageParams {
        threads: Some(tool_spec.resources.threads.max(1)),
        umi_pattern: config.umi_pattern,
        extraction_location: config.extraction_location,
        read_name_transform: config.read_name_transform,
        failed_extraction_policy: config.failed_extraction_policy,
        grouping_policy: config.grouping_policy,
        downstream_dedup_policy: config.downstream_dedup_policy,
        downstream_propagation: config.downstream_propagation,
    };

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_extract_umis_smoke_case(
                repo_root,
                &tool_spec,
                &plan_options,
                &output_root,
                case,
            )
        })
        .collect()
}

fn build_local_extract_umis_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    options: &crate::ExtractUmisStageParams,
    output_root: &Path,
    case: LocalExtractUmisSmokeCase,
) -> Result<LocalExtractUmisSmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.extract_umis r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    let r2_abs = repo_root.join(&case.r2);
    if !r2_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.extract_umis r2 fixture is missing: {}",
            r2_abs.display()
        ));
    }

    let out_dir = local_smoke_case_output_dir(
        repo_root,
        output_root,
        &case.sample_id,
        tool_spec.tool_id.as_str(),
    )?;
    let plan = plan_umi_with_options(tool_spec, &case.r1, &case.r2, &out_dir, options)?;
    let effective_params = serde_json::from_value::<bijux_dna_domain_fastq::FastqUmiParams>(
        plan.effective_params.clone(),
    )
    .map_err(|error| anyhow!("decode local-smoke fastq.extract_umis effective params: {error}"))?;

    Ok(LocalExtractUmisSmokeCasePlan {
        sample_id: case.sample_id,
        r1: case.r1,
        r2: case.r2,
        umi_pattern: effective_params.umi_pattern.unwrap_or_else(|| "NNNNNNNN".to_string()),
        read_name_transform: effective_params.read_name_transform,
        failed_extraction_policy: effective_params.failed_extraction_policy,
        plan,
    })
}

fn load_local_extract_umis_smoke_config(repo_root: &Path) -> Result<LocalExtractUmisSmokeConfig> {
    let path = repo_root.join(LOCAL_EXTRACT_UMIS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalExtractUmisSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_extract_umis.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.extract_umis schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.extract_umis must declare at least one governed case"
        ));
    }
    Ok(config)
}
