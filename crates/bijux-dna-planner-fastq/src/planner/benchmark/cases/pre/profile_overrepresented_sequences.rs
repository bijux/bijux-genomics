//! Governed FASTQ benchmark pre profile overrepresented sequences.
use crate::planner::benchmark::cases::admission::admit_unique_case_samples;
use crate::planner::benchmark::cases::admission::local_smoke_case_output_dir;
use crate::planner::benchmark::cases::hydrate_smoke_threads;
use crate::selection::load_fastq_domain_tool_execution_spec;
use crate::selection::select_profile_overrepresented_tools;
use crate::tool_adapters::fastq::profile_overrepresented_sequences::plan_with_options as plan_profile_overrepresented_sequences_with_options;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_core::prelude::StageId;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_core::prelude::ToolId;
use bijux_dna_domain_fastq::stages::ids::STAGE_PROFILE_OVERREPRESENTED_SEQUENCES;
use serde::Deserialize;
use std::path::Path;
use std::path::PathBuf;

const LOCAL_PROFILE_OVERREPRESENTED_SEQUENCES_CONFIG_PATH: &str =
    "configs/bench/local/fastq-profile-overrepresented-sequences.toml";

const DEFAULT_LOCAL_PROFILE_OVERREPRESENTED_SEQUENCES_OUTPUT_DIR: &str =
    "artifacts/runs/bench/local-smoke/fastq.profile_overrepresented_sequences";

#[derive(Debug, Clone)]
pub struct LocalProfileOverrepresentedSequencesSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalProfileOverrepresentedSequencesSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    top_k: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalProfileOverrepresentedSequencesSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalProfileOverrepresentedSequencesSmokeCase {
    sample_id: String,
    r1: PathBuf,
    #[serde(default)]
    r2: Option<PathBuf>,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_profile_overrepresented_sequences_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalProfileOverrepresentedSequencesSmokeCasePlan>> {
    let config = load_local_profile_overrepresented_sequences_smoke_config(repo_root)?;
    admit_unique_case_samples(
        "fastq.profile_overrepresented_sequences",
        config.cases.iter().map(|case| case.sample_id.as_str()),
    )?;

    let stage_id = StageId::new(STAGE_PROFILE_OVERREPRESENTED_SEQUENCES.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools =
        select_profile_overrepresented_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.profile_overrepresented_sequences tool selection normalized unexpectedly: {normalized_tools:?}"
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let top_k = config.top_k;
    let output_root = config.output_dir.unwrap_or_else(|| {
        PathBuf::from(DEFAULT_LOCAL_PROFILE_OVERREPRESENTED_SEQUENCES_OUTPUT_DIR)
    });

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_profile_overrepresented_sequences_smoke_case(
                repo_root,
                &tool_spec,
                top_k,
                &output_root,
                case,
            )
        })
        .collect()
}

fn build_local_profile_overrepresented_sequences_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    top_k: Option<u32>,
    output_root: &Path,
    case: LocalProfileOverrepresentedSequencesSmokeCase,
) -> Result<LocalProfileOverrepresentedSequencesSmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.profile_overrepresented_sequences r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    if let Some(r2) = case.r2.as_ref() {
        let r2_abs = repo_root.join(r2);
        if !r2_abs.is_file() {
            return Err(anyhow!(
                "local-smoke fastq.profile_overrepresented_sequences r2 fixture is missing: {}",
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
    let plan = plan_profile_overrepresented_sequences_with_options(
        tool_spec,
        &case.r1,
        case.r2.as_deref(),
        &out_dir,
        Some(tool_spec.resources.threads.max(1)),
        top_k,
    )?;

    Ok(LocalProfileOverrepresentedSequencesSmokeCasePlan {
        sample_id: case.sample_id,
        r1: case.r1,
        r2: case.r2,
        plan,
    })
}

fn load_local_profile_overrepresented_sequences_smoke_config(
    repo_root: &Path,
) -> Result<LocalProfileOverrepresentedSequencesSmokeConfig> {
    let path = repo_root.join(LOCAL_PROFILE_OVERREPRESENTED_SEQUENCES_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalProfileOverrepresentedSequencesSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_profile_overrepresented_sequences.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.profile_overrepresented_sequences schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.profile_overrepresented_sequences must declare at least one governed case"
        ));
    }
    Ok(config)
}
