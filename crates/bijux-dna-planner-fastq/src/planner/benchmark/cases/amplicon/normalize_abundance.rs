//! Governed FASTQ benchmark amplicon normalize abundance.
use crate::planner::benchmark::cases::admission::local_smoke_case_output_dir;
use crate::planner::benchmark::cases::hydrate_smoke_threads;
use crate::selection::load_fastq_domain_tool_execution_spec;
use crate::selection::select_normalize_abundance_tools;
use crate::tool_adapters::fastq::normalize_abundance::plan_with_options as plan_normalize_abundance_with_options;
use crate::tool_adapters::fastq::normalize_abundance::NormalizeAbundancePlanOptions;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_core::prelude::StageId;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_core::prelude::ToolId;
use bijux_dna_domain_fastq::stages::ids::STAGE_NORMALIZE_ABUNDANCE;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

const LOCAL_NORMALIZE_ABUNDANCE_CONFIG_PATH: &str =
    "configs/bench/local/fastq-normalize-abundance.toml";

const DEFAULT_LOCAL_NORMALIZE_ABUNDANCE_OUTPUT_DIR: &str =
    "artifacts/runs/bench/local-smoke/fastq.normalize_abundance";

#[derive(Debug, Clone)]
pub struct LocalNormalizeAbundanceSmokeCasePlan {
    pub sample_id: String,
    pub abundance_table: PathBuf,
    pub method: String,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalNormalizeAbundanceSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    method: Option<String>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalNormalizeAbundanceSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalNormalizeAbundanceSmokeCase {
    sample_id: String,
    abundance_table: PathBuf,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_normalize_abundance_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalNormalizeAbundanceSmokeCasePlan>> {
    let config = load_local_normalize_abundance_smoke_config(repo_root)?;
    ensure_unique_normalize_abundance_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_NORMALIZE_ABUNDANCE.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools = select_normalize_abundance_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.normalize_abundance tool selection normalized unexpectedly: {normalized_tools:?}"
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root = config
        .output_dir
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_NORMALIZE_ABUNDANCE_OUTPUT_DIR));
    let plan_options = NormalizeAbundancePlanOptions {
        method: config.method.unwrap_or_else(|| "relative_abundance".to_string()),
    };

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_normalize_abundance_smoke_case(
                repo_root,
                &tool_spec,
                &plan_options,
                &output_root,
                case,
            )
        })
        .collect()
}

fn build_local_normalize_abundance_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    options: &NormalizeAbundancePlanOptions,
    output_root: &Path,
    case: LocalNormalizeAbundanceSmokeCase,
) -> Result<LocalNormalizeAbundanceSmokeCasePlan> {
    let abundance_table_abs = repo_root.join(&case.abundance_table);
    if !abundance_table_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.normalize_abundance abundance_table fixture is missing: {}",
            abundance_table_abs.display()
        ));
    }

    let out_dir = local_smoke_case_output_dir(
        repo_root,
        output_root,
        &case.sample_id,
        tool_spec.tool_id.as_str(),
    )?;
    let plan =
        plan_normalize_abundance_with_options(tool_spec, &case.abundance_table, &out_dir, options)?;

    Ok(LocalNormalizeAbundanceSmokeCasePlan {
        sample_id: case.sample_id,
        abundance_table: case.abundance_table,
        method: options.method.clone(),
        plan,
    })
}

fn ensure_unique_normalize_abundance_sample_ids(
    cases: &[LocalNormalizeAbundanceSmokeCase],
) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "local-smoke fastq.normalize_abundance sample_id must not be empty"
            ));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.normalize_abundance sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn load_local_normalize_abundance_smoke_config(
    repo_root: &Path,
) -> Result<LocalNormalizeAbundanceSmokeConfig> {
    let path = repo_root.join(LOCAL_NORMALIZE_ABUNDANCE_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalNormalizeAbundanceSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_normalize_abundance.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.normalize_abundance schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.normalize_abundance must declare at least one governed case"
        ));
    }
    Ok(config)
}
