//! Governed BAM benchmark qc recalibration.
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
use bijux_dna_domain_bam::params::BqsrMode;
use bijux_dna_domain_bam::BamStage;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

const LOCAL_RECALIBRATION_CONFIG_PATH: &str = "configs/bench/local/bam-recalibration.toml";

const DEFAULT_LOCAL_RECALIBRATION_OUTPUT_DIR: &str =
    "artifacts/runs/bench/local-smoke/bam.recalibration";

#[derive(Debug, Clone)]
pub struct LocalRecalibrationSmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub reference: PathBuf,
    pub known_sites: Vec<PathBuf>,
    pub requested_mode: BqsrMode,
    pub effective_mode: BqsrMode,
    pub min_mean_coverage: f64,
    pub min_breadth_1x: f64,
    pub observed_mean_coverage: f64,
    pub observed_breadth_1x: f64,
    pub expected_status: String,
    pub expected_reason: String,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalRecalibrationSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalRecalibrationSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalRecalibrationSmokeCase {
    sample_id: String,
    bam: PathBuf,
    reference: PathBuf,
    known_sites: Vec<PathBuf>,
    mode: BqsrMode,
    min_mean_coverage: f64,
    min_breadth_1x: f64,
    expected_status: String,
    expected_reason: String,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.recalibration` plans cannot be built.
pub fn local_recalibration_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalRecalibrationSmokeCasePlan>> {
    let config = load_local_recalibration_smoke_config(repo_root)?;
    ensure_unique_recalibration_sample_ids(&config.cases)?;

    let stage = BamStage::Recalibration;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.recalibration tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_RECALIBRATION_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| build_local_recalibration_smoke_case(repo_root, &tool_spec, &output_root, case))
        .collect()
}

fn build_local_recalibration_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalRecalibrationSmokeCase,
) -> Result<LocalRecalibrationSmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.recalibration BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }
    let reference_abs = repo_root.join(&case.reference);
    if !reference_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.recalibration reference fixture is missing: {}",
            reference_abs.display()
        ));
    }
    if case.known_sites.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.recalibration case `{}` must declare at least one known_sites path",
            case.sample_id
        ));
    }
    for known_sites in &case.known_sites {
        let known_sites_abs = repo_root.join(known_sites);
        if !known_sites_abs.is_file() {
            return Err(anyhow!(
                "local-smoke bam.recalibration known-sites fixture is missing: {}",
                known_sites_abs.display()
            ));
        }
    }
    if case.min_mean_coverage < 0.0 || case.min_breadth_1x < 0.0 {
        return Err(anyhow!(
            "local-smoke bam.recalibration case `{}` must keep coverage gates non-negative",
            case.sample_id
        ));
    }
    if case.expected_status.trim().is_empty() || case.expected_reason.trim().is_empty() {
        return Err(anyhow!(
            "local-smoke bam.recalibration case `{}` must declare non-empty expected status and reason",
            case.sample_id
        ));
    }

    let coverage = bijux_dna_domain_bam::summarize_tiny_bam_coverage(&bam_abs, &[1])?;
    let observed_mean_coverage = coverage.mean_depth.unwrap_or(0.0);
    let observed_breadth_1x = coverage.regime.as_ref().map_or(0.0, |regime| regime.breadth_1x);
    let gate_tripped = observed_mean_coverage < case.min_mean_coverage
        || observed_breadth_1x < case.min_breadth_1x;
    let effective_mode = if gate_tripped { BqsrMode::Skip } else { case.mode };
    let decision_reason = if gate_tripped {
        "coverage_below_gate"
    } else {
        match case.mode {
            BqsrMode::Skip => "requested_skip_mode",
            BqsrMode::EmitOnly => "emit_only_requested",
            BqsrMode::Standard => "coverage_gate_passed",
        }
    };
    let status = match effective_mode {
        BqsrMode::Skip => "skipped",
        BqsrMode::EmitOnly => "emitted_only",
        BqsrMode::Standard => "ready_to_run",
    };
    if case.expected_status != status {
        return Err(anyhow!(
            "local-smoke bam.recalibration case `{}` must keep expected_status aligned with the governed recalibration decision",
            case.sample_id
        ));
    }
    if case.expected_reason != decision_reason {
        return Err(anyhow!(
            "local-smoke bam.recalibration case `{}` must keep expected_reason aligned with the governed recalibration decision",
            case.sample_id
        ));
    }
    let params = bijux_dna_domain_bam::params::BqsrEffectiveParams {
        known_sites: case.known_sites.iter().map(|path| path.display().to_string()).collect(),
        mode: effective_mode,
        skip_criteria: bijux_dna_domain_bam::params::RecalibrationSkipCriteria {
            min_mean_coverage: case.min_mean_coverage,
            min_breadth_1x: case.min_breadth_1x,
        },
    };
    let out_dir = local_smoke_case_output_dir(
        repo_root,
        output_root,
        &case.sample_id,
        tool_spec.tool_id.as_str(),
    )?;
    let mut plan = crate::tool_adapters::bam::recalibration::plan(
        tool_spec,
        &case.bam,
        Some(&case.reference),
        &out_dir,
        &params,
    )?;
    let plan_params = plan.params.as_object_mut().ok_or_else(|| {
        anyhow!("local-smoke bam.recalibration planner params must serialize as an object")
    })?;
    // Keep the mocked skip summary aligned with the governed local-smoke decision.
    for fragment in &mut plan.command.template {
        *fragment = fragment.replace("requested_skip_mode", decision_reason);
    }
    plan_params.insert("reference".to_string(), serde_json::json!(case.reference));
    plan_params.insert("requested_mode".to_string(), serde_json::json!(case.mode));
    plan_params.insert("status".to_string(), serde_json::json!(status));
    plan_params.insert("decision_reason".to_string(), serde_json::json!(decision_reason));
    plan_params
        .insert("observed_mean_coverage".to_string(), serde_json::json!(observed_mean_coverage));
    plan_params.insert("observed_breadth_1x".to_string(), serde_json::json!(observed_breadth_1x));

    Ok(LocalRecalibrationSmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        reference: case.reference,
        known_sites: case.known_sites,
        requested_mode: case.mode,
        effective_mode,
        min_mean_coverage: case.min_mean_coverage,
        min_breadth_1x: case.min_breadth_1x,
        observed_mean_coverage,
        observed_breadth_1x,
        expected_status: case.expected_status,
        expected_reason: case.expected_reason,
        plan,
    })
}

fn ensure_unique_recalibration_sample_ids(cases: &[LocalRecalibrationSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.recalibration sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke bam.recalibration sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn load_local_recalibration_smoke_config(
    repo_root: &Path,
) -> Result<LocalRecalibrationSmokeConfig> {
    let path = repo_root.join(LOCAL_RECALIBRATION_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalRecalibrationSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_recalibration.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.recalibration schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.recalibration must declare at least one governed case"
        ));
    }
    Ok(config)
}
