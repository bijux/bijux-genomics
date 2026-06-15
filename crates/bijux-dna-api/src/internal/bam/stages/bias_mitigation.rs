use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
#[cfg(feature = "bam_downstream")]
use serde::Serialize;

#[cfg(feature = "bam_downstream")]
const LOCAL_BIAS_MITIGATION_SMOKE_REPORT_SCHEMA_VERSION: &str =
    "bijux.bam.bias_mitigation.local_smoke.report.v1";
#[cfg(feature = "bam_downstream")]
const LOCAL_BIAS_MITIGATION_SMOKE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bam.bias_mitigation.local_smoke.metrics.v1";
const BIAS_MITIGATION_STAGE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bam.bias_mitigation.stage_metrics.v1";
const BIAS_MITIGATION_TOOL_REPORT_SCHEMA_VERSION: &str = "bijux.bam.bias_mitigation.v1";
const DEFAULT_BIAS_METRIC_NAME: &str = "gc_bias_score";

#[cfg(feature = "bam_downstream")]
#[derive(Debug, Clone, Serialize)]
struct LocalBiasMitigationSmokeReport {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    expectation_matched: bool,
    input_bam: String,
    reference_fasta: String,
    method: String,
    metric_name: String,
    pre_mitigation_metric: f64,
    post_mitigation_metric: f64,
    metric_delta: f64,
    mitigation_projection_basis: String,
    mitigation_actions: Vec<String>,
    consumed_metrics: Vec<String>,
    bias_report: String,
    bias_summary: String,
    bias_policy: String,
    stage_metrics: String,
}

#[derive(Debug, Clone, Deserialize)]
struct BiasMitigationToolReportV1 {
    #[serde(default)]
    schema_version: Option<String>,
    metric_name: String,
    #[serde(default)]
    pre_mitigation_metric: Option<f64>,
    #[serde(default)]
    post_mitigation_metric: Option<f64>,
    #[serde(default)]
    mitigation_projection_basis: Option<String>,
    #[serde(default)]
    insufficient_metric_reason: Option<String>,
}

/// Materialize the governed local-smoke `bam.bias_mitigation` artifacts and top-level report.
///
/// The written report lives at `runs/bench/local-smoke/bam.bias_mitigation/bias_mitigation.json`
/// under the active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, governed smoke plans are invalid,
/// or the smoke artifacts cannot be written.
#[cfg(feature = "bam_downstream")]
pub fn write_local_bias_mitigation_smoke_report() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let cases = bijux_dna_planner_bam::stage_api::local_bias_mitigation_smoke_plans(&repo_root)?;
    let [case] = cases.as_slice() else {
        return Err(anyhow!(
            "local-smoke bam.bias_mitigation expects exactly one governed case, found {}",
            cases.len()
        ));
    };

    let output_root = repo_root.join("runs/bench/local-smoke/bam.bias_mitigation");
    bijux_dna_infra::ensure_dir(&output_root)?;
    let report = materialize_local_bias_mitigation_smoke_case(&repo_root, case)?;
    let report_path = output_root.join("bias_mitigation.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;
    Ok(report_path)
}

/// Write durable `bam.bias_mitigation` summary artifacts beside stage outputs.
///
/// # Errors
/// Returns an error if the report payload cannot be parsed or the summary artifacts cannot be
/// written.
pub(crate) fn write_stage_bias_mitigation_artifacts(
    stage_dir: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
) -> Result<PathBuf> {
    let input_bam = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.role == bijux_dna_core::contract::ArtifactRole::Bam)
        .map_or_else(|| stage_dir.join("in.bam"), |artifact| artifact.path.clone());
    let input_bam = resolve_stage_input_path(&input_bam);
    let reference_fasta = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.role == bijux_dna_core::contract::ArtifactRole::Reference)
        .map(|artifact| resolve_stage_input_path(&artifact.path))
        .or_else(|| {
            plan.params
                .get("reference")
                .and_then(serde_json::Value::as_str)
                .map(PathBuf::from)
                .map(|path| resolve_stage_input_path(&path))
        });
    let gc_bias_correction =
        plan.params.get("gc_bias_correction").and_then(serde_json::Value::as_bool).unwrap_or(false);
    let map_bias_correction = plan
        .params
        .get("map_bias_correction")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);

    let policy_path =
        write_stage_bias_mitigation_policy(stage_dir, gc_bias_correction, map_bias_correction)?;
    let report_path = stage_dir.join("bias.json");
    let summary_path = stage_dir.join("bias.summary.json");
    let stage_metrics_path = stage_dir.join("stage.metrics.json");

    let summary = if report_path.exists() {
        let report = read_bias_mitigation_tool_report(&report_path)?;
        if let Some(schema_version) = report.schema_version.as_deref() {
            if schema_version != BIAS_MITIGATION_TOOL_REPORT_SCHEMA_VERSION {
                return Err(anyhow!(
                    "bam.bias_mitigation hard failure: unsupported bias report schema_version `{schema_version}`"
                ));
            }
        }
        bijux_dna_domain_bam::summarize_bam_bias_mitigation(
            "bam.bias_mitigation",
            &input_bam,
            reference_fasta.as_deref(),
            plan.tool_id.as_str(),
            gc_bias_correction,
            map_bias_correction,
            true,
            report.metric_name.as_str(),
            report.pre_mitigation_metric,
            report.post_mitigation_metric,
            report.mitigation_projection_basis,
            report.insufficient_metric_reason,
        )
    } else {
        bijux_dna_domain_bam::summarize_bam_bias_mitigation(
            "bam.bias_mitigation",
            &input_bam,
            reference_fasta.as_deref(),
            plan.tool_id.as_str(),
            gc_bias_correction,
            map_bias_correction,
            false,
            DEFAULT_BIAS_METRIC_NAME,
            None,
            None,
            None,
            Some("bias_metrics_unavailable".to_string()),
        )
    };

    bijux_dna_infra::atomic_write_json(&summary_path, &summary)
        .with_context(|| format!("write {}", summary_path.display()))?;
    bijux_dna_infra::atomic_write_json(
        &stage_metrics_path,
        &serde_json::json!({
            "schema_version": BIAS_MITIGATION_STAGE_METRICS_SCHEMA_VERSION,
            "stage_id": "bam.bias_mitigation",
            "method": summary.method,
            "metric_name": summary.metric_name,
            "pre_mitigation_metric": summary.pre_mitigation_metric,
            "post_mitigation_metric": summary.post_mitigation_metric,
            "metric_delta": summary.metric_delta,
            "mitigation_actions": summary.mitigation_actions,
            "consumed_metrics": summary.consumed_metrics,
            "mitigation_projection_basis": summary.mitigation_projection_basis,
            "report_present": summary.report_present,
            "insufficient_metric_reason": summary.insufficient_metric_reason,
            "bias_policy": policy_path,
        }),
    )
    .with_context(|| format!("write {}", stage_metrics_path.display()))?;
    Ok(summary_path)
}

#[cfg(feature = "bam_downstream")]
fn materialize_local_bias_mitigation_smoke_case(
    repo_root: &Path,
    case: &bijux_dna_planner_bam::stage_api::LocalBiasMitigationSmokeCasePlan,
) -> Result<LocalBiasMitigationSmokeReport> {
    let case_out_dir = resolve_plan_path(repo_root, &case.plan.out_dir);
    bijux_dna_infra::ensure_dir(&case_out_dir)?;

    let bias_report_path = resolve_output_path(repo_root, &case.plan, "bias_report")?;
    let bias_summary_path = resolve_output_path(repo_root, &case.plan, "summary")?;
    let stage_metrics_path = resolve_output_path(repo_root, &case.plan, "stage_metrics")?;
    let bias_policy_path = case_out_dir.join("bias_mitigation.policy.json");
    let input_bam = repo_root.join(&case.bam);
    let reference_fasta = repo_root.join(&case.reference);
    let gc_bias_correction = case
        .plan
        .params
        .get("gc_bias_correction")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let map_bias_correction = case
        .plan
        .params
        .get("map_bias_correction")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);

    let summary = bijux_dna_domain_bam::summarize_tiny_bam_bias_mitigation(
        &input_bam,
        &reference_fasta,
        case.plan.tool_id.as_str(),
        case.window_size,
        gc_bias_correction,
        map_bias_correction,
    )?;
    bijux_dna_infra::atomic_write_json(&bias_report_path, &bias_tool_report(&summary))?;
    write_stage_bias_mitigation_artifacts(&case_out_dir, &case.plan)?;

    let summary_json: bijux_dna_domain_bam::BamBiasMitigationSummaryV1 = serde_json::from_str(
        &std::fs::read_to_string(&bias_summary_path)
            .with_context(|| format!("read {}", bias_summary_path.display()))?,
    )
    .with_context(|| format!("parse {}", bias_summary_path.display()))?;
    let pre_mitigation_metric = summary_json.pre_mitigation_metric.ok_or_else(|| {
        anyhow!("bam.bias_mitigation local-smoke summary is missing pre_mitigation_metric")
    })?;
    let post_mitigation_metric = summary_json.post_mitigation_metric.ok_or_else(|| {
        anyhow!("bam.bias_mitigation local-smoke summary is missing post_mitigation_metric")
    })?;
    let metric_delta = summary_json.metric_delta.ok_or_else(|| {
        anyhow!("bam.bias_mitigation local-smoke summary is missing metric_delta")
    })?;
    let mitigation_projection_basis = summary_json
        .mitigation_projection_basis
        .clone()
        .unwrap_or_else(|| "policy_projection".to_string());

    let expectation_matched = summary_json.method == case.plan.tool_id.as_str()
        && summary_json.metric_name == case.expected_metric_name
        && float_matches(pre_mitigation_metric, case.expected_pre_mitigation_metric)
        && float_matches(post_mitigation_metric, case.expected_post_mitigation_metric);
    let pre_mitigation_metric_delta = pre_mitigation_metric - case.expected_pre_mitigation_metric;
    let post_mitigation_metric_delta =
        post_mitigation_metric - case.expected_post_mitigation_metric;

    bijux_dna_infra::atomic_write_json(
        &stage_metrics_path,
        &serde_json::json!({
            "schema_version": LOCAL_BIAS_MITIGATION_SMOKE_METRICS_SCHEMA_VERSION,
            "stage_id": "bam.bias_mitigation",
            "sample_id": case.sample_id,
            "expected_method": case.plan.tool_id.as_str(),
            "method": summary_json.method,
            "expected_metric_name": case.expected_metric_name,
            "metric_name": summary_json.metric_name,
            "expected_pre_mitigation_metric": case.expected_pre_mitigation_metric,
            "pre_mitigation_metric": pre_mitigation_metric,
            "pre_mitigation_metric_delta": pre_mitigation_metric_delta,
            "expected_post_mitigation_metric": case.expected_post_mitigation_metric,
            "post_mitigation_metric": post_mitigation_metric,
            "post_mitigation_metric_delta": post_mitigation_metric_delta,
            "metric_delta": metric_delta,
            "mitigation_actions": summary_json.mitigation_actions,
            "consumed_metrics": summary_json.consumed_metrics,
            "mitigation_projection_basis": mitigation_projection_basis,
            "expectation_matched": expectation_matched,
        }),
    )?;

    Ok(LocalBiasMitigationSmokeReport {
        schema_version: LOCAL_BIAS_MITIGATION_SMOKE_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: "bam.bias_mitigation".to_string(),
        sample_id: case.sample_id.clone(),
        expectation_matched,
        input_bam: path_relative_to_repo(repo_root, &input_bam),
        reference_fasta: path_relative_to_repo(repo_root, &reference_fasta),
        method: summary_json.method,
        metric_name: summary_json.metric_name,
        pre_mitigation_metric,
        post_mitigation_metric,
        metric_delta,
        mitigation_projection_basis,
        mitigation_actions: summary_json.mitigation_actions,
        consumed_metrics: summary_json.consumed_metrics,
        bias_report: path_relative_to_repo(repo_root, &bias_report_path),
        bias_summary: path_relative_to_repo(repo_root, &bias_summary_path),
        bias_policy: path_relative_to_repo(repo_root, &bias_policy_path),
        stage_metrics: path_relative_to_repo(repo_root, &stage_metrics_path),
    })
}

#[cfg(feature = "bam_downstream")]
fn bias_tool_report(
    summary: &bijux_dna_domain_bam::BamBiasMitigationSummaryV1,
) -> serde_json::Value {
    serde_json::json!({
        "schema_version": BIAS_MITIGATION_TOOL_REPORT_SCHEMA_VERSION,
        "method": summary.method,
        "metric_name": summary.metric_name,
        "gc_bias_correction": summary.gc_bias_correction,
        "map_bias_correction": summary.map_bias_correction,
        "mitigation_actions": summary.mitigation_actions,
        "consumed_metrics": summary.consumed_metrics,
        "pre_mitigation_metric": summary.pre_mitigation_metric,
        "post_mitigation_metric": summary.post_mitigation_metric,
        "metric_delta": summary.metric_delta,
        "mitigation_projection_basis": summary.mitigation_projection_basis,
        "insufficient_metric_reason": summary.insufficient_metric_reason,
    })
}

fn read_bias_mitigation_tool_report(path: &Path) -> Result<BiasMitigationToolReportV1> {
    serde_json::from_str(
        &std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?,
    )
    .with_context(|| format!("parse {}", path.display()))
}

fn write_stage_bias_mitigation_policy(
    stage_dir: &Path,
    gc_bias_correction: bool,
    map_bias_correction: bool,
) -> Result<PathBuf> {
    let path = stage_dir.join("bias_mitigation.policy.json");
    bijux_dna_infra::atomic_write_json(
        &path,
        &serde_json::json!({
            "gc_bias_correction": gc_bias_correction,
            "map_bias_correction": map_bias_correction,
        }),
    )
    .with_context(|| format!("write {}", path.display()))?;
    Ok(path)
}

#[cfg(feature = "bam_downstream")]
fn float_matches(left: f64, right: f64) -> bool {
    (left - right).abs() <= 1e-9
}

#[cfg(feature = "bam_downstream")]
fn resolve_output_path(
    repo_root: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
    output_id: &str,
) -> Result<PathBuf> {
    let path = plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == output_id)
        .map(|artifact| artifact.path.clone())
        .ok_or_else(|| {
            anyhow!("bam.bias_mitigation local-smoke plan is missing governed output `{output_id}`")
        })?;
    Ok(resolve_plan_path(repo_root, &path))
}

#[cfg(feature = "bam_downstream")]
fn resolve_plan_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

#[cfg(feature = "bam_downstream")]
fn relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(repo_root).unwrap_or(path).to_path_buf()
}

#[cfg(feature = "bam_downstream")]
fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    relative_path(repo_root, path).display().to_string()
}

fn resolve_stage_input_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    if let Ok(repo_root) = crate::support::workspace::resolve_repo_root() {
        let candidate = repo_root.join(path);
        if candidate.exists() {
            return candidate;
        }
    }
    path.to_path_buf()
}
