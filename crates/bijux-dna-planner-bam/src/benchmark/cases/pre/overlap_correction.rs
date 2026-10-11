//! Governed BAM benchmark pre overlap correction.
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

const LOCAL_OVERLAP_CORRECTION_CONFIG_PATH: &str =
    "configs/bench/local/bam-overlap-correction.toml";

const DEFAULT_LOCAL_OVERLAP_CORRECTION_OUTPUT_DIR: &str =
    "artifacts/runs/bench/local-smoke/bam.overlap_correction";

#[derive(Debug, Clone)]
pub struct LocalOverlapCorrectionSmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub expected_pair_count: u64,
    pub expected_corrected_pairs: u64,
    pub expected_corrected_overlap_bases: u64,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalOverlapCorrectionSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalOverlapCorrectionSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalOverlapCorrectionSmokeCase {
    sample_id: String,
    bam: PathBuf,
    expected_pair_count: u64,
    expected_corrected_pairs: u64,
    expected_corrected_overlap_bases: u64,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.overlap_correction` plans cannot be built.
pub fn local_overlap_correction_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalOverlapCorrectionSmokeCasePlan>> {
    let config = load_local_overlap_correction_smoke_config(repo_root)?;
    ensure_unique_overlap_correction_sample_ids(&config.cases)?;

    let stage = BamStage::OverlapCorrection;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.overlap_correction tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root = config
        .output_dir
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_OVERLAP_CORRECTION_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_overlap_correction_smoke_case(repo_root, &tool_spec, &output_root, case)
        })
        .collect()
}

fn build_local_overlap_correction_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalOverlapCorrectionSmokeCase,
) -> Result<LocalOverlapCorrectionSmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.overlap_correction BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }
    if case.expected_pair_count == 0 {
        return Err(anyhow!(
            "local-smoke bam.overlap_correction case `{}` must declare expected_pair_count greater than zero",
            case.sample_id
        ));
    }
    if case.expected_corrected_pairs > case.expected_pair_count {
        return Err(anyhow!(
            "local-smoke bam.overlap_correction case `{}` cannot declare corrected pairs greater than pair count",
            case.sample_id
        ));
    }
    if case.expected_corrected_pairs > 0 && case.expected_corrected_overlap_bases == 0 {
        return Err(anyhow!(
            "local-smoke bam.overlap_correction case `{}` must declare positive expected_corrected_overlap_bases when expected_corrected_pairs is greater than zero",
            case.sample_id
        ));
    }
    if case.expected_corrected_pairs == 0 && case.expected_corrected_overlap_bases > 0 {
        return Err(anyhow!(
            "local-smoke bam.overlap_correction case `{}` must keep expected_corrected_overlap_bases at zero when expected_corrected_pairs is zero",
            case.sample_id
        ));
    }

    let params = FilterEffectiveParams {
        mapq_threshold: 0,
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
    let plan = crate::tool_adapters::bam::overlap_correction::plan(
        tool_spec, &case.bam, &out_dir, &params,
    )?;

    Ok(LocalOverlapCorrectionSmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        expected_pair_count: case.expected_pair_count,
        expected_corrected_pairs: case.expected_corrected_pairs,
        expected_corrected_overlap_bases: case.expected_corrected_overlap_bases,
        plan,
    })
}

fn ensure_unique_overlap_correction_sample_ids(
    cases: &[LocalOverlapCorrectionSmokeCase],
) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.overlap_correction sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke bam.overlap_correction sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn load_local_overlap_correction_smoke_config(
    repo_root: &Path,
) -> Result<LocalOverlapCorrectionSmokeConfig> {
    let path = repo_root.join(LOCAL_OVERLAP_CORRECTION_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalOverlapCorrectionSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_overlap_correction.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.overlap_correction schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.overlap_correction must declare at least one governed case"
        ));
    }
    Ok(config)
}
