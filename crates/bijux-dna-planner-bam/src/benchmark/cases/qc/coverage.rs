//! Governed BAM benchmark qc coverage.
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
use bijux_dna_domain_bam::params::CoverageEffectiveParams;
use bijux_dna_domain_bam::types::BedRegions;
use bijux_dna_domain_bam::BamStage;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

const LOCAL_COVERAGE_CONFIG_PATH: &str = "configs/bench/local/bam-coverage.toml";

const DEFAULT_LOCAL_COVERAGE_OUTPUT_DIR: &str = "artifacts/runs/bench/local-smoke/bam.coverage";

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct LocalCoverageSmokeExpectedRow {
    pub region_id: String,
    pub contig: String,
    pub start: u64,
    pub end: u64,
    pub length: u64,
    pub mean_depth: f64,
    pub breadth_1x: f64,
    pub covered_bases: u64,
}

#[derive(Debug, Clone)]
pub struct LocalCoverageSmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub regions: PathBuf,
    pub depth_thresholds: Vec<u32>,
    pub expected_coverage_regime: String,
    pub expected_rows: Vec<LocalCoverageSmokeExpectedRow>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalCoverageSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalCoverageSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalCoverageSmokeCase {
    sample_id: String,
    bam: PathBuf,
    regions: PathBuf,
    depth_thresholds: Vec<u32>,
    expected_coverage_regime: String,
    expected_rows: Vec<LocalCoverageSmokeExpectedRow>,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.coverage` plans cannot be built.
pub fn local_coverage_smoke_plans(repo_root: &Path) -> Result<Vec<LocalCoverageSmokeCasePlan>> {
    let config = load_local_coverage_smoke_config(repo_root)?;
    ensure_unique_coverage_sample_ids(&config.cases)?;

    let stage = BamStage::Coverage;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.coverage tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_COVERAGE_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| build_local_coverage_smoke_case(repo_root, &tool_spec, &output_root, case))
        .collect()
}

fn build_local_coverage_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalCoverageSmokeCase,
) -> Result<LocalCoverageSmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.coverage BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }
    let regions_abs = repo_root.join(&case.regions);
    if !regions_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.coverage regions fixture is missing: {}",
            regions_abs.display()
        ));
    }
    if case.depth_thresholds.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.coverage case `{}` must declare at least one depth threshold",
            case.sample_id
        ));
    }
    if case.depth_thresholds.contains(&0) {
        return Err(anyhow!(
            "local-smoke bam.coverage case `{}` must keep depth thresholds greater than zero",
            case.sample_id
        ));
    }
    if case.depth_thresholds.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(anyhow!(
            "local-smoke bam.coverage case `{}` must keep depth thresholds strictly increasing",
            case.sample_id
        ));
    }
    if case.expected_coverage_regime.trim().is_empty() {
        return Err(anyhow!(
            "local-smoke bam.coverage case `{}` must declare a non-empty expected coverage regime",
            case.sample_id
        ));
    }
    if case.expected_rows.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.coverage case `{}` must declare at least one expected region row",
            case.sample_id
        ));
    }

    let mut seen_region_ids = BTreeSet::new();
    for row in &case.expected_rows {
        if row.region_id.trim().is_empty() {
            return Err(anyhow!(
                "local-smoke bam.coverage case `{}` must not declare empty region identifiers",
                case.sample_id
            ));
        }
        if !seen_region_ids.insert(row.region_id.clone()) {
            return Err(anyhow!(
                "local-smoke bam.coverage case `{}` declared duplicate region `{}`",
                case.sample_id,
                row.region_id
            ));
        }
        if row.contig.trim().is_empty() {
            return Err(anyhow!(
                "local-smoke bam.coverage case `{}` must not declare empty contig names",
                case.sample_id
            ));
        }
        if row.start == 0 || row.end < row.start {
            return Err(anyhow!(
                "local-smoke bam.coverage case `{}` must keep region coordinates 1-based and ordered",
                case.sample_id
            ));
        }
        if row.length != row.end.saturating_sub(row.start).saturating_add(1) {
            return Err(anyhow!(
                "local-smoke bam.coverage case `{}` must keep expected row length aligned with region coordinates",
                case.sample_id
            ));
        }
        if !(0.0..=1.0).contains(&row.breadth_1x) {
            return Err(anyhow!(
                "local-smoke bam.coverage case `{}` must keep breadth_1x within [0, 1]",
                case.sample_id
            ));
        }
        if row.covered_bases > row.length {
            return Err(anyhow!(
                "local-smoke bam.coverage case `{}` cannot declare covered bases greater than region length",
                case.sample_id
            ));
        }
    }

    let params = CoverageEffectiveParams {
        regions: Some(BedRegions(case.regions.clone())),
        depth_thresholds: case.depth_thresholds.clone(),
        regime_mode: "advisory_and_enforced".to_string(),
    };
    let out_dir = local_smoke_case_output_dir(
        repo_root,
        output_root,
        &case.sample_id,
        tool_spec.tool_id.as_str(),
    )?;
    let plan = crate::tool_adapters::bam::coverage::plan(tool_spec, &case.bam, &out_dir, &params)?;

    Ok(LocalCoverageSmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        regions: case.regions,
        depth_thresholds: case.depth_thresholds,
        expected_coverage_regime: case.expected_coverage_regime,
        expected_rows: case.expected_rows,
        plan,
    })
}

fn ensure_unique_coverage_sample_ids(cases: &[LocalCoverageSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.coverage sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke bam.coverage sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn load_local_coverage_smoke_config(repo_root: &Path) -> Result<LocalCoverageSmokeConfig> {
    let path = repo_root.join(LOCAL_COVERAGE_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalCoverageSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_coverage.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.coverage schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!("local-smoke bam.coverage must declare at least one governed case"));
    }
    Ok(config)
}
