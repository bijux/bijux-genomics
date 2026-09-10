//! Governed BAM benchmark pre mapq filter.
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

const LOCAL_MAPQ_FILTER_CONFIG_PATH: &str = "configs/bench/local/bam-mapq-filter.toml";

const DEFAULT_LOCAL_MAPQ_FILTER_OUTPUT_DIR: &str =
    "artifacts/runs/bench/local-smoke/bam.mapq_filter";

#[derive(Debug, Clone)]
pub struct LocalMapqFilterSmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub mapq_threshold: u8,
    pub expected_input_reads: u64,
    pub expected_kept_reads: u64,
    pub expected_removed_reads: u64,
    pub expected_mapped_reads_removed: u64,
    pub expected_mapped_fraction_retained: f64,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalMapqFilterSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalMapqFilterSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalMapqFilterSmokeCase {
    sample_id: String,
    bam: PathBuf,
    mapq_threshold: u8,
    expected_input_reads: u64,
    expected_kept_reads: u64,
    expected_removed_reads: u64,
    expected_mapped_reads_removed: u64,
    expected_mapped_fraction_retained: f64,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.mapq_filter` plans cannot be built.
pub fn local_mapq_filter_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalMapqFilterSmokeCasePlan>> {
    let config = load_local_mapq_filter_smoke_config(repo_root)?;
    ensure_unique_mapq_filter_sample_ids(&config.cases)?;

    let stage = BamStage::MapqFilter;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.mapq_filter tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_MAPQ_FILTER_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| build_local_mapq_filter_smoke_case(repo_root, &tool_spec, &output_root, case))
        .collect()
}

fn build_local_mapq_filter_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalMapqFilterSmokeCase,
) -> Result<LocalMapqFilterSmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.mapq_filter BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }
    if case.mapq_threshold == 0 {
        return Err(anyhow!(
            "local-smoke bam.mapq_filter case `{}` must declare a non-zero mapq_threshold",
            case.sample_id
        ));
    }
    if case.expected_kept_reads > case.expected_input_reads {
        return Err(anyhow!(
            "local-smoke bam.mapq_filter case `{}` cannot declare kept reads greater than input reads",
            case.sample_id
        ));
    }
    if case.expected_removed_reads
        != case.expected_input_reads.saturating_sub(case.expected_kept_reads)
    {
        return Err(anyhow!(
            "local-smoke bam.mapq_filter case `{}` must keep expected removed reads aligned with input and kept reads",
            case.sample_id
        ));
    }
    if !(0.0..=1.0).contains(&case.expected_mapped_fraction_retained) {
        return Err(anyhow!(
            "local-smoke bam.mapq_filter case `{}` must declare mapped_fraction_retained within [0, 1]",
            case.sample_id
        ));
    }

    let params = FilterEffectiveParams {
        mapq_threshold: case.mapq_threshold,
        include_flags: Vec::new(),
        exclude_flags: Vec::new(),
        min_length: 0,
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
        crate::tool_adapters::bam::mapq_filter::plan(tool_spec, &case.bam, &out_dir, &params)?;

    Ok(LocalMapqFilterSmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        mapq_threshold: case.mapq_threshold,
        expected_input_reads: case.expected_input_reads,
        expected_kept_reads: case.expected_kept_reads,
        expected_removed_reads: case.expected_removed_reads,
        expected_mapped_reads_removed: case.expected_mapped_reads_removed,
        expected_mapped_fraction_retained: case.expected_mapped_fraction_retained,
        plan,
    })
}

fn ensure_unique_mapq_filter_sample_ids(cases: &[LocalMapqFilterSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.mapq_filter sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke bam.mapq_filter sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn load_local_mapq_filter_smoke_config(repo_root: &Path) -> Result<LocalMapqFilterSmokeConfig> {
    let path = repo_root.join(LOCAL_MAPQ_FILTER_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalMapqFilterSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_mapq_filter.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.mapq_filter schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!("local-smoke bam.mapq_filter must declare at least one governed case"));
    }
    Ok(config)
}
