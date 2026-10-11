//! Governed BAM benchmark pre mapping summary.
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
use bijux_dna_domain_bam::BamStage;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

const LOCAL_MAPPING_SUMMARY_CONFIG_PATH: &str = "configs/bench/local/bam-mapping-summary.toml";

const DEFAULT_LOCAL_MAPPING_SUMMARY_OUTPUT_DIR: &str =
    "artifacts/runs/bench/local-smoke/bam.mapping_summary";

#[derive(Debug, Clone)]
pub struct LocalMappingSummarySmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub expected_total_reads: u64,
    pub expected_mapped_reads: u64,
    pub expected_mapping_fraction: f64,
    pub expected_reference_name: String,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalMappingSummarySmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalMappingSummarySmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalMappingSummarySmokeCase {
    sample_id: String,
    bam: PathBuf,
    expected_total_reads: u64,
    expected_mapped_reads: u64,
    expected_mapping_fraction: f64,
    expected_reference_name: String,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.mapping_summary` plans cannot be built.
pub fn local_mapping_summary_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalMappingSummarySmokeCasePlan>> {
    let config = load_local_mapping_summary_smoke_config(repo_root)?;
    ensure_unique_mapping_summary_sample_ids(&config.cases)?;

    let stage = BamStage::MappingSummary;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.mapping_summary tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root = config
        .output_dir
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_MAPPING_SUMMARY_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_mapping_summary_smoke_case(repo_root, &tool_spec, &output_root, case)
        })
        .collect()
}

fn build_local_mapping_summary_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalMappingSummarySmokeCase,
) -> Result<LocalMappingSummarySmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.mapping_summary BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }
    if case.expected_reference_name.trim().is_empty() {
        return Err(anyhow!(
            "local-smoke bam.mapping_summary case `{}` must declare a non-empty expected reference name",
            case.sample_id
        ));
    }
    if case.expected_mapped_reads > case.expected_total_reads {
        return Err(anyhow!(
            "local-smoke bam.mapping_summary case `{}` cannot declare mapped reads greater than total reads",
            case.sample_id
        ));
    }
    if !(0.0..=1.0).contains(&case.expected_mapping_fraction) {
        return Err(anyhow!(
            "local-smoke bam.mapping_summary case `{}` must declare mapping fraction within [0, 1]",
            case.sample_id
        ));
    }
    let derived_fraction = if case.expected_total_reads == 0 {
        0.0
    } else {
        case.expected_mapped_reads as f64 / case.expected_total_reads as f64
    };
    if (derived_fraction - case.expected_mapping_fraction).abs() > 1e-9 {
        return Err(anyhow!(
            "local-smoke bam.mapping_summary case `{}` must keep expected mapping fraction aligned with mapped and total reads",
            case.sample_id
        ));
    }

    let out_dir = local_smoke_case_output_dir(
        repo_root,
        output_root,
        &case.sample_id,
        tool_spec.tool_id.as_str(),
    )?;
    let plan = crate::tool_adapters::bam::mapping_summary::plan(tool_spec, &case.bam, &out_dir)?;

    Ok(LocalMappingSummarySmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        expected_total_reads: case.expected_total_reads,
        expected_mapped_reads: case.expected_mapped_reads,
        expected_mapping_fraction: case.expected_mapping_fraction,
        expected_reference_name: case.expected_reference_name,
        plan,
    })
}

fn ensure_unique_mapping_summary_sample_ids(cases: &[LocalMappingSummarySmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.mapping_summary sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke bam.mapping_summary sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn load_local_mapping_summary_smoke_config(
    repo_root: &Path,
) -> Result<LocalMappingSummarySmokeConfig> {
    let path = repo_root.join(LOCAL_MAPPING_SUMMARY_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalMappingSummarySmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_mapping_summary.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.mapping_summary schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.mapping_summary must declare at least one governed case"
        ));
    }
    Ok(config)
}
