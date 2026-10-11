//! Governed BAM benchmark ancient authenticity.
use crate::benchmark::cases::admission::float_matches;
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
use bijux_dna_domain_bam::params::AuthenticityEffectiveParams;
use bijux_dna_domain_bam::BamStage;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

const LOCAL_AUTHENTICITY_CONFIG_PATH: &str = "configs/bench/local/bam-authenticity.toml";

const DEFAULT_LOCAL_AUTHENTICITY_OUTPUT_DIR: &str =
    "artifacts/runs/bench/local-smoke/bam.authenticity";

const LOCAL_AUTHENTICITY_EXPECTED_METRIC_IDS: [&str; 5] =
    ["damage", "contamination", "complexity", "coverage", "mapping"];

#[derive(Debug, Clone)]
pub struct LocalAuthenticitySmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub damage_terminal_c_to_t_5p: f64,
    pub damage_terminal_g_to_a_3p: f64,
    pub contamination_method: String,
    pub contamination_estimate: f64,
    pub contamination_ci_low: f64,
    pub contamination_ci_high: f64,
    pub complexity_min_reads: u64,
    pub complexity_projection_points: Vec<u64>,
    pub coverage_depth_thresholds: Vec<u32>,
    pub expected_score: f64,
    pub expected_confidence: f64,
    pub expected_pmd_like_signal_present: bool,
    pub expected_consumed_metrics: Vec<String>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalAuthenticitySmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalAuthenticitySmokeCase>,
}

#[derive(Debug, Clone, Deserialize)]
struct LocalAuthenticitySmokeCase {
    sample_id: String,
    bam: PathBuf,
    damage_terminal_c_to_t_5p: f64,
    damage_terminal_g_to_a_3p: f64,
    contamination_method: String,
    contamination_estimate: f64,
    contamination_ci_low: f64,
    contamination_ci_high: f64,
    complexity_min_reads: u64,
    complexity_projection_points: Vec<u64>,
    coverage_depth_thresholds: Vec<u32>,
    expected_score: f64,
    expected_confidence: f64,
    expected_pmd_like_signal_present: bool,
    expected_consumed_metrics: Vec<String>,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.authenticity` plans cannot be built.
pub fn local_authenticity_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalAuthenticitySmokeCasePlan>> {
    let config = load_local_authenticity_smoke_config(repo_root)?;
    ensure_unique_authenticity_sample_ids(&config.cases)?;

    let stage = BamStage::Authenticity;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.authenticity tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_AUTHENTICITY_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| build_local_authenticity_smoke_case(repo_root, &tool_spec, &output_root, case))
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or one of
/// the admitted `bam.authenticity` tool plans cannot be built for output-contract proof coverage.
pub fn local_authenticity_output_contract_plans(
    repo_root: &Path,
) -> Result<Vec<LocalAuthenticitySmokeCasePlan>> {
    let config = load_local_authenticity_smoke_config(repo_root)?;
    ensure_unique_authenticity_sample_ids(&config.cases)?;

    let stage = BamStage::Authenticity;
    let stage_id = StageId::new(stage.as_str().to_string());
    let mut tool_ids = allowed_tools_for_stage(stage);
    tool_ids.sort_by(|left, right| left.as_str().cmp(right.as_str()));
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_AUTHENTICITY_OUTPUT_DIR));
    let cases = config.cases;

    let mut plans = Vec::new();
    for tool_id in tool_ids {
        let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
        hydrate_smoke_threads(&mut tool_spec, config.threads);
        for case in &cases {
            plans.push(build_local_authenticity_smoke_case(
                repo_root,
                &tool_spec,
                &output_root,
                case.clone(),
            )?);
        }
    }
    Ok(plans)
}

fn build_local_authenticity_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalAuthenticitySmokeCase,
) -> Result<LocalAuthenticitySmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.authenticity BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }
    if !(0.0..=1.0).contains(&case.damage_terminal_c_to_t_5p) {
        return Err(anyhow!(
            "local-smoke bam.authenticity case `{}` must keep damage_terminal_c_to_t_5p within [0, 1]",
            case.sample_id
        ));
    }
    if !(0.0..=1.0).contains(&case.damage_terminal_g_to_a_3p) {
        return Err(anyhow!(
            "local-smoke bam.authenticity case `{}` must keep damage_terminal_g_to_a_3p within [0, 1]",
            case.sample_id
        ));
    }
    if !(0.0..=1.0).contains(&case.contamination_estimate)
        || !(0.0..=1.0).contains(&case.contamination_ci_low)
        || !(0.0..=1.0).contains(&case.contamination_ci_high)
    {
        return Err(anyhow!(
            "local-smoke bam.authenticity case `{}` must keep contamination estimate and interval within [0, 1]",
            case.sample_id
        ));
    }
    if case.contamination_ci_low > case.contamination_ci_high {
        return Err(anyhow!(
            "local-smoke bam.authenticity case `{}` must keep contamination_ci_low less than or equal to contamination_ci_high",
            case.sample_id
        ));
    }
    if case.contamination_estimate < case.contamination_ci_low
        || case.contamination_estimate > case.contamination_ci_high
    {
        return Err(anyhow!(
            "local-smoke bam.authenticity case `{}` must keep contamination_estimate within the declared confidence interval",
            case.sample_id
        ));
    }
    if case.contamination_method.trim().is_empty() {
        return Err(anyhow!(
            "local-smoke bam.authenticity case `{}` must declare a non-empty contamination_method",
            case.sample_id
        ));
    }
    if case.complexity_min_reads == 0 {
        return Err(anyhow!(
            "local-smoke bam.authenticity case `{}` must declare complexity_min_reads > 0",
            case.sample_id
        ));
    }
    if case.complexity_projection_points.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.authenticity case `{}` must declare at least one complexity_projection_point",
            case.sample_id
        ));
    }
    if case.coverage_depth_thresholds.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.authenticity case `{}` must declare at least one coverage_depth_threshold",
            case.sample_id
        ));
    }
    if case.expected_consumed_metrics.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.authenticity case `{}` must declare expected_consumed_metrics",
            case.sample_id
        ));
    }
    if case.expected_consumed_metrics != LOCAL_AUTHENTICITY_EXPECTED_METRIC_IDS.map(str::to_string)
    {
        return Err(anyhow!(
            "local-smoke bam.authenticity case `{}` must keep expected_consumed_metrics aligned with the governed composition inputs",
            case.sample_id
        ));
    }

    let governed_advisory = bijux_dna_domain_bam::summarize_tiny_bam_authenticity_advisory(
        &bam_abs,
        &bijux_dna_domain_bam::metrics::DamageMetricsV1 {
            c_to_t_5p: case.damage_terminal_c_to_t_5p,
            g_to_a_3p: case.damage_terminal_g_to_a_3p,
            pmd_score_histogram: Vec::new(),
        },
    )?;
    if !float_matches(case.expected_score, governed_advisory.score) {
        return Err(anyhow!(
            "local-smoke bam.authenticity case `{}` must keep expected_score aligned with the governed authenticity advisory",
            case.sample_id
        ));
    }
    if !float_matches(case.expected_confidence, governed_advisory.confidence) {
        return Err(anyhow!(
            "local-smoke bam.authenticity case `{}` must keep expected_confidence aligned with the governed authenticity advisory",
            case.sample_id
        ));
    }
    if case.expected_pmd_like_signal_present != governed_advisory.pmd_like_signal_present {
        return Err(anyhow!(
            "local-smoke bam.authenticity case `{}` must keep expected_pmd_like_signal_present aligned with the governed authenticity advisory",
            case.sample_id
        ));
    }

    let params = AuthenticityEffectiveParams {
        mode: "aggregate".to_string(),
        evidence_only: true,
        disallow_certification: true,
    };
    let out_dir = local_smoke_case_output_dir(
        repo_root,
        output_root,
        &case.sample_id,
        tool_spec.tool_id.as_str(),
    )?;
    let plan =
        crate::tool_adapters::bam::authenticity::plan(tool_spec, &case.bam, &out_dir, &params)?;

    Ok(LocalAuthenticitySmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        damage_terminal_c_to_t_5p: case.damage_terminal_c_to_t_5p,
        damage_terminal_g_to_a_3p: case.damage_terminal_g_to_a_3p,
        contamination_method: case.contamination_method,
        contamination_estimate: case.contamination_estimate,
        contamination_ci_low: case.contamination_ci_low,
        contamination_ci_high: case.contamination_ci_high,
        complexity_min_reads: case.complexity_min_reads,
        complexity_projection_points: case.complexity_projection_points,
        coverage_depth_thresholds: case.coverage_depth_thresholds,
        expected_score: case.expected_score,
        expected_confidence: case.expected_confidence,
        expected_pmd_like_signal_present: case.expected_pmd_like_signal_present,
        expected_consumed_metrics: case.expected_consumed_metrics,
        plan,
    })
}

fn ensure_unique_authenticity_sample_ids(cases: &[LocalAuthenticitySmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.authenticity sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke bam.authenticity sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn load_local_authenticity_smoke_config(repo_root: &Path) -> Result<LocalAuthenticitySmokeConfig> {
    let path = repo_root.join(LOCAL_AUTHENTICITY_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalAuthenticitySmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_authenticity.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.authenticity schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.authenticity must declare at least one governed case"
        ));
    }
    Ok(config)
}
