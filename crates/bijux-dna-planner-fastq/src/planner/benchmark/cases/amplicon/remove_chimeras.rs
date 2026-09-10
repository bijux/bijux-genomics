//! Governed FASTQ benchmark amplicon remove chimeras.
use crate::planner::benchmark::cases::admission::local_smoke_case_output_dir;
use crate::planner::benchmark::cases::hydrate_smoke_threads;
use crate::selection::load_fastq_domain_tool_execution_spec;
use crate::selection::select_remove_chimeras_tools;
use crate::tool_adapters::fastq::remove_chimeras::plan as plan_remove_chimeras;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_core::prelude::StageId;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_core::prelude::ToolId;
use bijux_dna_domain_fastq::stages::ids::STAGE_REMOVE_CHIMERAS;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

const LOCAL_REMOVE_CHIMERAS_CONFIG_PATH: &str = "configs/bench/local/fastq-remove-chimeras.toml";

const DEFAULT_LOCAL_REMOVE_CHIMERAS_OUTPUT_DIR: &str =
    "artifacts/runs/bench/local-smoke/fastq.remove_chimeras";

#[derive(Debug, Clone)]
pub struct LocalRemoveChimerasSmokeCasePlan {
    pub sample_id: String,
    pub reads: PathBuf,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalRemoveChimerasSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalRemoveChimerasSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalRemoveChimerasSmokeCase {
    sample_id: String,
    reads: PathBuf,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_remove_chimeras_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalRemoveChimerasSmokeCasePlan>> {
    let config = load_local_remove_chimeras_smoke_config(repo_root)?;
    ensure_unique_remove_chimeras_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_REMOVE_CHIMERAS.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools = select_remove_chimeras_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.remove_chimeras tool selection normalized unexpectedly: {normalized_tools:?}"
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root = config
        .output_dir
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_REMOVE_CHIMERAS_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_remove_chimeras_smoke_case(repo_root, &tool_spec, &output_root, case)
        })
        .collect()
}

fn build_local_remove_chimeras_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalRemoveChimerasSmokeCase,
) -> Result<LocalRemoveChimerasSmokeCasePlan> {
    let reads_abs = repo_root.join(&case.reads);
    if !reads_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.remove_chimeras reads fixture is missing: {}",
            reads_abs.display()
        ));
    }

    let out_dir = local_smoke_case_output_dir(
        repo_root,
        output_root,
        &case.sample_id,
        tool_spec.tool_id.as_str(),
    )?;
    let plan = plan_remove_chimeras(tool_spec, &case.reads, None, &out_dir)?;

    Ok(LocalRemoveChimerasSmokeCasePlan { sample_id: case.sample_id, reads: case.reads, plan })
}

fn ensure_unique_remove_chimeras_sample_ids(cases: &[LocalRemoveChimerasSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke fastq.remove_chimeras sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.remove_chimeras sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn load_local_remove_chimeras_smoke_config(
    repo_root: &Path,
) -> Result<LocalRemoveChimerasSmokeConfig> {
    let path = repo_root.join(LOCAL_REMOVE_CHIMERAS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalRemoveChimerasSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_remove_chimeras.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.remove_chimeras schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.remove_chimeras must declare at least one governed case"
        ));
    }
    Ok(config)
}
