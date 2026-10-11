//! Governed BAM benchmark qc gc bias.
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
use bijux_dna_domain_bam::BamStage;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

const LOCAL_GC_BIAS_CONFIG_PATH: &str = "configs/bench/local/bam-gc-bias.toml";

const DEFAULT_LOCAL_GC_BIAS_OUTPUT_DIR: &str = "artifacts/runs/bench/local-smoke/bam.gc_bias";

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct LocalGcBiasSmokeExpectedRow {
    pub gc_bin: u8,
    pub normalized_coverage: f64,
    pub windows: u64,
    pub read_starts: u64,
}

#[derive(Debug, Clone)]
pub struct LocalGcBiasSmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub reference: PathBuf,
    pub window_size: u32,
    pub expected_rows: Vec<LocalGcBiasSmokeExpectedRow>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalGcBiasSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalGcBiasSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalGcBiasSmokeCase {
    sample_id: String,
    bam: PathBuf,
    reference: PathBuf,
    window_size: u32,
    expected_rows: Vec<LocalGcBiasSmokeExpectedRow>,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.gc_bias` plans cannot be built.
pub fn local_gc_bias_smoke_plans(repo_root: &Path) -> Result<Vec<LocalGcBiasSmokeCasePlan>> {
    let config = load_local_gc_bias_smoke_config(repo_root)?;
    ensure_unique_gc_bias_sample_ids(&config.cases)?;

    let stage = BamStage::GcBias;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.gc_bias tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_GC_BIAS_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| build_local_gc_bias_smoke_case(repo_root, &tool_spec, &output_root, case))
        .collect()
}

fn build_local_gc_bias_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalGcBiasSmokeCase,
) -> Result<LocalGcBiasSmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.gc_bias BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }
    let reference_abs = repo_root.join(&case.reference);
    if !reference_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.gc_bias reference fixture is missing: {}",
            reference_abs.display()
        ));
    }
    if case.window_size == 0 {
        return Err(anyhow!(
            "local-smoke bam.gc_bias case `{}` must declare window_size greater than zero",
            case.sample_id
        ));
    }
    validate_local_gc_bias_expected_rows(&case.sample_id, &case.expected_rows)?;

    let params = CoverageEffectiveParams {
        regions: None,
        depth_thresholds: vec![1],
        regime_mode: "advisory_and_enforced".to_string(),
    };
    let out_dir = local_smoke_case_output_dir(
        repo_root,
        output_root,
        &case.sample_id,
        tool_spec.tool_id.as_str(),
    )?;
    let mut plan = crate::tool_adapters::bam::gc_bias::plan(
        tool_spec,
        &case.bam,
        &case.reference,
        &out_dir,
        &params,
    )?;
    let plan_params = plan.params.as_object_mut().ok_or_else(|| {
        anyhow!("local-smoke bam.gc_bias planner params must serialize as an object")
    })?;
    plan_params.insert("window_size".to_string(), serde_json::json!(case.window_size));

    Ok(LocalGcBiasSmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        reference: case.reference,
        window_size: case.window_size,
        expected_rows: case.expected_rows,
        plan,
    })
}

fn validate_local_gc_bias_expected_rows(
    sample_id: &str,
    expected_rows: &[LocalGcBiasSmokeExpectedRow],
) -> Result<()> {
    if expected_rows.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.gc_bias case `{sample_id}` must declare at least one expected GC bin row"
        ));
    }

    let mut seen_gc_bins = BTreeSet::new();
    for row in expected_rows {
        if !seen_gc_bins.insert(row.gc_bin) {
            return Err(anyhow!(
                "local-smoke bam.gc_bias case `{}` must not repeat expected gc_bin `{}`",
                sample_id,
                row.gc_bin
            ));
        }
        if row.windows == 0 {
            return Err(anyhow!(
                "local-smoke bam.gc_bias case `{}` must keep expected gc_bin `{}` windows greater than zero",
                sample_id,
                row.gc_bin
            ));
        }
        if !row.normalized_coverage.is_finite() || row.normalized_coverage < 0.0 {
            return Err(anyhow!(
                "local-smoke bam.gc_bias case `{}` must keep expected gc_bin `{}` normalized_coverage finite and non-negative",
                sample_id,
                row.gc_bin
            ));
        }
    }

    Ok(())
}

fn ensure_unique_gc_bias_sample_ids(cases: &[LocalGcBiasSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.gc_bias sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke bam.gc_bias sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn load_local_gc_bias_smoke_config(repo_root: &Path) -> Result<LocalGcBiasSmokeConfig> {
    let path = repo_root.join(LOCAL_GC_BIAS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalGcBiasSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_gc_bias.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.gc_bias schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!("local-smoke bam.gc_bias must declare at least one governed case"));
    }
    Ok(config)
}
