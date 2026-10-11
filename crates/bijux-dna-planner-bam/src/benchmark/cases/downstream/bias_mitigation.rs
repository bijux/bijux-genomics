//! Governed BAM benchmark downstream bias mitigation.
use crate::benchmark::cases::admission::float_matches;
use crate::benchmark::cases::admission::local_smoke_case_output_dir;
use crate::benchmark::cases::hydrate_smoke_threads;
use crate::selection::allowed_tools_for_stage;
use crate::selection::load_bam_domain_tool_planning_spec;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_core::prelude::ArtifactId;
use bijux_dna_core::prelude::ArtifactRole;
use bijux_dna_core::prelude::StageId;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_core::prelude::ToolId;
use bijux_dna_domain_bam::BamStage;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

#[cfg(feature = "bam_downstream")]
const LOCAL_BIAS_MITIGATION_CONFIG_PATH: &str = "configs/bench/local/bam-bias-mitigation.toml";

#[cfg(feature = "bam_downstream")]
const DEFAULT_LOCAL_BIAS_MITIGATION_OUTPUT_DIR: &str =
    "artifacts/runs/bench/local-smoke/bam.bias_mitigation";

#[cfg(feature = "bam_downstream")]
#[derive(Debug, Clone)]
pub struct LocalBiasMitigationSmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub reference: PathBuf,
    pub window_size: u32,
    pub expected_metric_name: String,
    pub expected_pre_mitigation_metric: f64,
    pub expected_post_mitigation_metric: f64,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[cfg(feature = "bam_downstream")]
#[derive(Debug, Deserialize)]
struct LocalBiasMitigationSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalBiasMitigationSmokeCase>,
}

#[cfg(feature = "bam_downstream")]
#[derive(Debug, Deserialize)]
struct LocalBiasMitigationSmokeCase {
    sample_id: String,
    bam: PathBuf,
    reference: PathBuf,
    window_size: u32,
    gc_bias_correction: bool,
    map_bias_correction: bool,
    expected_metric_name: String,
    expected_pre_mitigation_metric: f64,
    expected_post_mitigation_metric: f64,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.bias_mitigation` plans cannot be built.
#[cfg(feature = "bam_downstream")]
pub fn local_bias_mitigation_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalBiasMitigationSmokeCasePlan>> {
    let config = load_local_bias_mitigation_smoke_config(repo_root)?;
    ensure_unique_bias_mitigation_sample_ids(&config.cases)?;

    let stage = BamStage::BiasMitigation;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.bias_mitigation tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root = config
        .output_dir
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_BIAS_MITIGATION_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_bias_mitigation_smoke_case(repo_root, &tool_spec, &output_root, case)
        })
        .collect()
}

#[cfg(feature = "bam_downstream")]
fn build_local_bias_mitigation_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalBiasMitigationSmokeCase,
) -> Result<LocalBiasMitigationSmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.bias_mitigation BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }
    let reference_abs = repo_root.join(&case.reference);
    if !reference_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.bias_mitigation reference fixture is missing: {}",
            reference_abs.display()
        ));
    }
    if case.window_size == 0 {
        return Err(anyhow!(
            "local-smoke bam.bias_mitigation case `{}` must declare window_size greater than zero",
            case.sample_id
        ));
    }
    if case.expected_metric_name.trim().is_empty() {
        return Err(anyhow!(
            "local-smoke bam.bias_mitigation case `{}` must declare a non-empty expected_metric_name",
            case.sample_id
        ));
    }
    if case.expected_pre_mitigation_metric < 0.0 || case.expected_post_mitigation_metric < 0.0 {
        return Err(anyhow!(
            "local-smoke bam.bias_mitigation case `{}` must keep expected metrics non-negative",
            case.sample_id
        ));
    }
    if case.expected_post_mitigation_metric > case.expected_pre_mitigation_metric {
        return Err(anyhow!(
            "local-smoke bam.bias_mitigation case `{}` must not raise the expected post-mitigation metric above the pre-mitigation metric",
            case.sample_id
        ));
    }
    let governed_summary = bijux_dna_domain_bam::summarize_tiny_bam_bias_mitigation(
        &bam_abs,
        &reference_abs,
        tool_spec.tool_id.as_str(),
        case.window_size,
        case.gc_bias_correction,
        case.map_bias_correction,
    )?;
    if case.expected_metric_name != governed_summary.metric_name {
        return Err(anyhow!(
            "local-smoke bam.bias_mitigation case `{}` must keep expected_metric_name aligned with the governed bias summary",
            case.sample_id
        ));
    }
    let governed_pre_metric = governed_summary.pre_mitigation_metric.ok_or_else(|| {
        anyhow!(
            "local-smoke bam.bias_mitigation case `{}` governed bias summary is missing pre_mitigation_metric",
            case.sample_id
        )
    })?;
    if !float_matches(case.expected_pre_mitigation_metric, governed_pre_metric) {
        return Err(anyhow!(
            "local-smoke bam.bias_mitigation case `{}` must keep expected_pre_mitigation_metric aligned with the governed bias summary",
            case.sample_id
        ));
    }
    let governed_post_metric = governed_summary.post_mitigation_metric.ok_or_else(|| {
        anyhow!(
            "local-smoke bam.bias_mitigation case `{}` governed bias summary is missing post_mitigation_metric",
            case.sample_id
        )
    })?;
    if !float_matches(case.expected_post_mitigation_metric, governed_post_metric) {
        return Err(anyhow!(
            "local-smoke bam.bias_mitigation case `{}` must keep expected_post_mitigation_metric aligned with the governed bias summary",
            case.sample_id
        ));
    }

    let params = bijux_dna_domain_bam::params::BiasMitigationEffectiveParams {
        gc_bias_correction: case.gc_bias_correction,
        map_bias_correction: case.map_bias_correction,
    };
    let out_dir = local_smoke_case_output_dir(
        repo_root,
        output_root,
        &case.sample_id,
        tool_spec.tool_id.as_str(),
    )?;
    let mut plan =
        crate::tool_adapters::bam::bias_mitigation::plan(tool_spec, &case.bam, &out_dir, &params)?;
    plan.io.inputs.push(bijux_dna_stage_contract::ArtifactRef::required(
        ArtifactId::from_static("reference"),
        case.reference.clone(),
        ArtifactRole::Reference,
    ));
    let plan_params = plan.params.as_object_mut().ok_or_else(|| {
        anyhow!("local-smoke bam.bias_mitigation planner params must serialize as an object")
    })?;
    plan_params.insert("reference".to_string(), serde_json::json!(case.reference));
    plan_params.insert("window_size".to_string(), serde_json::json!(case.window_size));

    Ok(LocalBiasMitigationSmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        reference: case.reference,
        window_size: case.window_size,
        expected_metric_name: case.expected_metric_name,
        expected_pre_mitigation_metric: case.expected_pre_mitigation_metric,
        expected_post_mitigation_metric: case.expected_post_mitigation_metric,
        plan,
    })
}

#[cfg(feature = "bam_downstream")]
fn ensure_unique_bias_mitigation_sample_ids(cases: &[LocalBiasMitigationSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.bias_mitigation sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke bam.bias_mitigation sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

#[cfg(feature = "bam_downstream")]
fn load_local_bias_mitigation_smoke_config(
    repo_root: &Path,
) -> Result<LocalBiasMitigationSmokeConfig> {
    let path = repo_root.join(LOCAL_BIAS_MITIGATION_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalBiasMitigationSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_bias_mitigation.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.bias_mitigation schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.bias_mitigation must declare at least one governed case"
        ));
    }
    Ok(config)
}
