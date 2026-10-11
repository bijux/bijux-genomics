//! Governed BAM benchmark pre length filter.
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
use bijux_dna_domain_bam::params::FilterEffectiveParams;
use bijux_dna_domain_bam::BamStage;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

const LOCAL_LENGTH_FILTER_CONFIG_PATH: &str = "configs/bench/local/bam-length-filter.toml";

const DEFAULT_LOCAL_LENGTH_FILTER_OUTPUT_DIR: &str =
    "artifacts/runs/bench/local-smoke/bam.length_filter";

#[derive(Debug, Clone)]
pub struct LocalLengthFilterSmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub min_length: u32,
    pub expected_input_reads: u64,
    pub expected_kept_reads: u64,
    pub expected_removed_reads: u64,
    pub expected_observed_min_length: u32,
    pub expected_observed_max_length: u32,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalLengthFilterSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalLengthFilterSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalLengthFilterSmokeCase {
    sample_id: String,
    bam: PathBuf,
    min_length: u32,
    expected_input_reads: u64,
    expected_kept_reads: u64,
    expected_removed_reads: u64,
    expected_observed_min_length: u32,
    expected_observed_max_length: u32,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.length_filter` plans cannot be built.
pub fn local_length_filter_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalLengthFilterSmokeCasePlan>> {
    let config = load_local_length_filter_smoke_config(repo_root)?;
    ensure_unique_length_filter_sample_ids(&config.cases)?;

    let stage = BamStage::LengthFilter;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.length_filter tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_LENGTH_FILTER_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| build_local_length_filter_smoke_case(repo_root, &tool_spec, &output_root, case))
        .collect()
}

fn build_local_length_filter_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalLengthFilterSmokeCase,
) -> Result<LocalLengthFilterSmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.length_filter BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }
    if case.min_length == 0 {
        return Err(anyhow!(
            "local-smoke bam.length_filter case `{}` must declare a non-zero min_length",
            case.sample_id
        ));
    }
    if case.expected_kept_reads > case.expected_input_reads {
        return Err(anyhow!(
            "local-smoke bam.length_filter case `{}` cannot declare kept reads greater than input reads",
            case.sample_id
        ));
    }
    if case.expected_removed_reads
        != case.expected_input_reads.saturating_sub(case.expected_kept_reads)
    {
        return Err(anyhow!(
            "local-smoke bam.length_filter case `{}` must keep expected removed reads aligned with input and kept reads",
            case.sample_id
        ));
    }
    if case.expected_observed_min_length > case.expected_observed_max_length {
        return Err(anyhow!(
            "local-smoke bam.length_filter case `{}` must declare observed min length less than or equal to observed max length",
            case.sample_id
        ));
    }
    if case.expected_observed_min_length < case.min_length {
        return Err(anyhow!(
            "local-smoke bam.length_filter case `{}` must keep observed min length at or above the filter threshold",
            case.sample_id
        ));
    }

    let params = FilterEffectiveParams {
        mapq_threshold: 0,
        include_flags: Vec::new(),
        exclude_flags: Vec::new(),
        min_length: case.min_length,
        remove_duplicates: false,
        base_quality_threshold: 20,
    };
    let out_dir = local_smoke_case_output_dir(
        repo_root,
        output_root,
        &case.sample_id,
        tool_spec.tool_id.as_str(),
    )?;
    let plan =
        crate::tool_adapters::bam::length_filter::plan(tool_spec, &case.bam, &out_dir, &params)?;

    Ok(LocalLengthFilterSmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        min_length: case.min_length,
        expected_input_reads: case.expected_input_reads,
        expected_kept_reads: case.expected_kept_reads,
        expected_removed_reads: case.expected_removed_reads,
        expected_observed_min_length: case.expected_observed_min_length,
        expected_observed_max_length: case.expected_observed_max_length,
        plan,
    })
}

fn ensure_unique_length_filter_sample_ids(cases: &[LocalLengthFilterSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.length_filter sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke bam.length_filter sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn load_local_length_filter_smoke_config(repo_root: &Path) -> Result<LocalLengthFilterSmokeConfig> {
    let path = repo_root.join(LOCAL_LENGTH_FILTER_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalLengthFilterSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_length_filter.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.length_filter schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.length_filter must declare at least one governed case"
        ));
    }
    Ok(config)
}
