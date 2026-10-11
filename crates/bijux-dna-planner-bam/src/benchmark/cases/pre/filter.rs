//! Governed BAM benchmark pre filter.
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

const LOCAL_FILTER_CONFIG_PATH: &str = "configs/bench/local/bam-filter.toml";

const DEFAULT_LOCAL_FILTER_OUTPUT_DIR: &str = "artifacts/runs/bench/local-smoke/bam.filter";

#[derive(Debug, Clone)]
pub struct LocalFilterSmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub expected_input_reads: u64,
    pub expected_kept_reads: u64,
    pub expected_removed_reads: u64,
    pub expected_active_filters: Vec<String>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalFilterSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalFilterSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalFilterSmokeCase {
    sample_id: String,
    bam: PathBuf,
    expected_input_reads: u64,
    expected_kept_reads: u64,
    expected_removed_reads: u64,
    expected_active_filters: Vec<String>,
    mapq_threshold: u8,
    #[serde(default)]
    include_flags: Vec<u16>,
    #[serde(default)]
    exclude_flags: Vec<u16>,
    min_length: u32,
    remove_duplicates: bool,
    base_quality_threshold: u8,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.filter` plans cannot be built.
pub fn local_filter_smoke_plans(repo_root: &Path) -> Result<Vec<LocalFilterSmokeCasePlan>> {
    let config = load_local_filter_smoke_config(repo_root)?;
    ensure_unique_filter_sample_ids(&config.cases)?;

    let stage = BamStage::Filter;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.filter tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_FILTER_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| build_local_filter_smoke_case(repo_root, &tool_spec, &output_root, case))
        .collect()
}

fn build_local_filter_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalFilterSmokeCase,
) -> Result<LocalFilterSmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.filter BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }
    if case.expected_kept_reads > case.expected_input_reads {
        return Err(anyhow!(
            "local-smoke bam.filter case `{}` cannot declare kept reads greater than input reads",
            case.sample_id
        ));
    }
    if case.expected_removed_reads
        != case.expected_input_reads.saturating_sub(case.expected_kept_reads)
    {
        return Err(anyhow!(
            "local-smoke bam.filter case `{}` must keep expected removed reads aligned with input and kept reads",
            case.sample_id
        ));
    }
    if case.expected_active_filters.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.filter case `{}` must declare at least one active filter",
            case.sample_id
        ));
    }
    let mut seen_filters = BTreeSet::new();
    for filter in &case.expected_active_filters {
        if filter.trim().is_empty() {
            return Err(anyhow!(
                "local-smoke bam.filter case `{}` must not declare empty active filter names",
                case.sample_id
            ));
        }
        if !seen_filters.insert(filter.clone()) {
            return Err(anyhow!(
                "local-smoke bam.filter case `{}` declared duplicate active filter `{}`",
                case.sample_id,
                filter
            ));
        }
    }

    let params = FilterEffectiveParams {
        mapq_threshold: case.mapq_threshold,
        include_flags: case.include_flags,
        exclude_flags: case.exclude_flags,
        min_length: case.min_length,
        remove_duplicates: case.remove_duplicates,
        base_quality_threshold: case.base_quality_threshold,
    };
    let out_dir = local_smoke_case_output_dir(
        repo_root,
        output_root,
        &case.sample_id,
        tool_spec.tool_id.as_str(),
    )?;
    let plan = crate::tool_adapters::bam::filter::plan(tool_spec, &case.bam, &out_dir, &params)?;

    Ok(LocalFilterSmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        expected_input_reads: case.expected_input_reads,
        expected_kept_reads: case.expected_kept_reads,
        expected_removed_reads: case.expected_removed_reads,
        expected_active_filters: case.expected_active_filters,
        plan,
    })
}

fn ensure_unique_filter_sample_ids(cases: &[LocalFilterSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.filter sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!("duplicate local-smoke bam.filter sample_id `{}`", case.sample_id));
        }
    }
    Ok(())
}

fn load_local_filter_smoke_config(repo_root: &Path) -> Result<LocalFilterSmokeConfig> {
    let path = repo_root.join(LOCAL_FILTER_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalFilterSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_filter.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.filter schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!("local-smoke bam.filter must declare at least one governed case"));
    }
    Ok(config)
}
