use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

const LOCAL_CONTAMINATION_SMOKE_REPORT_SCHEMA_VERSION: &str =
    "bijux.bam.contamination.local_smoke.report.v1";
const LOCAL_CONTAMINATION_STAGE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bam.contamination.stage_metrics.v1";

#[derive(Debug, Clone, Copy)]
enum ProofCaseKind {
    Ready,
    Insufficient,
}

impl ProofCaseKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Insufficient => "insufficient",
        }
    }

    const fn expected_prerequisites_passed(self) -> bool {
        matches!(self, Self::Ready)
    }
}

#[derive(Debug, Clone, Serialize)]
struct LocalContaminationSmokeCaseReport {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    proof_case: String,
    sample_id: String,
    expectation_matched: bool,
    input_bam: String,
    reference_fasta: String,
    scope: String,
    tool_scope: String,
    minimum_mean_coverage: f64,
    prerequisites_passed: bool,
    refusal_codes: Vec<String>,
    caveats: Vec<String>,
    raw_estimate: f64,
    raw_ci_low: f64,
    raw_ci_high: f64,
    contamination_report: String,
    contamination_summary: String,
    stage_metrics: String,
    declared_output_ids: Vec<String>,
    artifact_paths: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    contamination_estimate: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    contammix_report: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mt_consensus: Option<String>,
    advisory_boundary: String,
    contamination_modes: String,
    contamination_stratified: String,
}

#[derive(Debug, Clone, Serialize)]
struct LocalContaminationSmokeReport {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    tool_ids: Vec<String>,
    case_count: usize,
    rows: Vec<LocalContaminationSmokeCaseReport>,
}

struct LocalContaminationCasePaths {
    contamination_report: PathBuf,
    contamination_summary: PathBuf,
    stage_metrics: PathBuf,
    contamination_estimate: Option<PathBuf>,
    contammix_report: Option<PathBuf>,
    mt_consensus: Option<PathBuf>,
    advisory_boundary: PathBuf,
    contamination_modes: PathBuf,
    contamination_stratified: PathBuf,
}

/// Materialize the governed local-smoke `bam.contamination` bundle for all retained tools.
///
/// The written report lives at `runs/bench/local-smoke/bam.contamination/local_smoke.json`
/// under the active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, governed smoke plans are invalid,
/// or the smoke artifacts cannot be written.
pub fn write_local_contamination_smoke_report() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let plans = bijux_dna_planner_bam::stage_api::local_contamination_smoke_plans(&repo_root)?;
    if plans.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.contamination expects governed tool plans, found none"
        ));
    }

    let output_root = repo_root.join("runs/bench/local-smoke/bam.contamination");
    bijux_dna_infra::ensure_dir(&output_root)?;

    let sample_id = required_string_param(&plans[0], "sample_id")?;
    let mut rows = Vec::with_capacity(plans.len() * 2);
    let mut tool_ids = Vec::with_capacity(plans.len());
    for plan in &plans {
        tool_ids.push(plan.tool_id.as_str().to_string());
        rows.push(materialize_local_contamination_case(&repo_root, plan, ProofCaseKind::Ready)?);
        rows.push(materialize_local_contamination_case(
            &repo_root,
            plan,
            ProofCaseKind::Insufficient,
        )?);
    }
    tool_ids.sort();
    rows.sort_by(|left, right| {
        left.tool_id.cmp(&right.tool_id).then_with(|| left.proof_case.cmp(&right.proof_case))
    });

    let report = LocalContaminationSmokeReport {
        schema_version: LOCAL_CONTAMINATION_SMOKE_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: "bam.contamination".to_string(),
        sample_id,
        tool_ids,
        case_count: rows.len(),
        rows,
    };

    let report_path = output_root.join("local_smoke.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;
    Ok(report_path)
}

fn materialize_local_contamination_case(
    repo_root: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
    proof_case: ProofCaseKind,
) -> Result<LocalContaminationSmokeCaseReport> {
    let tool_id = plan.tool_id.as_str();
    let sample_id = required_string_param(plan, "sample_id")?;
    let logical_scope = required_string_param(plan, "scope")?;
    let tool_scope = required_string_param(plan, "tool_scope")?;
    let minimum_mean_coverage = required_f64_param(plan, "minimum_mean_coverage")?;
    let input_bam = resolve_input_path(repo_root, plan, "bam")?;
    let reference_fasta = resolve_input_path(repo_root, plan, "reference")?;

    let raw_metrics = raw_metrics(tool_id, &logical_scope, &tool_scope, proof_case);
    let summary = contamination_summary(tool_id, proof_case, minimum_mean_coverage)?;
    let case_paths = resolve_local_contamination_case_paths(repo_root, plan, tool_id, proof_case)?;

    write_raw_contamination_report(&case_paths.contamination_report, &raw_metrics)?;
    bijux_dna_infra::atomic_write_json(&case_paths.contamination_summary, &summary)
        .with_context(|| format!("write {}", case_paths.contamination_summary.display()))?;
    write_stage_metrics(
        &case_paths.stage_metrics,
        plan,
        proof_case,
        minimum_mean_coverage,
        &raw_metrics,
        &summary,
    )?;
    bijux_dna_infra::atomic_write_json(&case_paths.advisory_boundary, &summary.advisory_boundary)
        .with_context(|| format!("write {}", case_paths.advisory_boundary.display()))?;
    write_contamination_modes(&case_paths.contamination_modes, &logical_scope, &tool_scope, plan)?;
    write_contamination_stratified(
        &case_paths.contamination_stratified,
        tool_id,
        &tool_scope,
        raw_metrics.estimate,
    )?;
    write_local_contamination_optional_artifacts(
        &case_paths,
        tool_id,
        &logical_scope,
        proof_case,
        &raw_metrics,
        &summary,
    )?;

    let expectation_matched = contamination_expectation_matched(&summary, &raw_metrics, proof_case);
    let artifact_paths = contamination_artifact_paths(repo_root, &case_paths);

    let declared_output_ids = plan
        .io
        .outputs
        .iter()
        .map(|artifact| artifact.name.as_str().to_string())
        .collect::<Vec<_>>();

    Ok(LocalContaminationSmokeCaseReport {
        schema_version: LOCAL_CONTAMINATION_SMOKE_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: "bam.contamination".to_string(),
        tool_id: tool_id.to_string(),
        proof_case: proof_case.as_str().to_string(),
        sample_id,
        expectation_matched,
        input_bam: path_relative_to_repo(repo_root, &input_bam),
        reference_fasta: path_relative_to_repo(repo_root, &reference_fasta),
        scope: logical_scope,
        tool_scope,
        minimum_mean_coverage,
        prerequisites_passed: summary.prerequisites_passed,
        refusal_codes: summary.refusal_codes.clone(),
        caveats: summary.caveats.clone(),
        raw_estimate: raw_metrics.estimate,
        raw_ci_low: raw_metrics.ci_low,
        raw_ci_high: raw_metrics.ci_high,
        contamination_report: path_relative_to_repo(repo_root, &case_paths.contamination_report),
        contamination_summary: path_relative_to_repo(repo_root, &case_paths.contamination_summary),
        stage_metrics: path_relative_to_repo(repo_root, &case_paths.stage_metrics),
        declared_output_ids,
        artifact_paths,
        contamination_estimate: case_paths
            .contamination_estimate
            .as_ref()
            .map(|path| path_relative_to_repo(repo_root, path)),
        contammix_report: case_paths
            .contammix_report
            .as_ref()
            .map(|path| path_relative_to_repo(repo_root, path)),
        mt_consensus: case_paths
            .mt_consensus
            .as_ref()
            .map(|path| path_relative_to_repo(repo_root, path)),
        advisory_boundary: path_relative_to_repo(repo_root, &case_paths.advisory_boundary),
        contamination_modes: path_relative_to_repo(repo_root, &case_paths.contamination_modes),
        contamination_stratified: path_relative_to_repo(
            repo_root,
            &case_paths.contamination_stratified,
        ),
    })
}

fn resolve_local_contamination_case_paths(
    repo_root: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
    tool_id: &str,
    proof_case: ProofCaseKind,
) -> Result<LocalContaminationCasePaths> {
    let case_dir = repo_root
        .join("runs/bench/local-smoke/bam.contamination")
        .join(tool_id)
        .join(proof_case.as_str());
    bijux_dna_infra::ensure_dir(&case_dir)?;
    Ok(LocalContaminationCasePaths {
        contamination_report: resolve_case_output_path(plan, "contamination_report", &case_dir)?,
        contamination_summary: resolve_case_output_path(plan, "summary", &case_dir)?,
        stage_metrics: resolve_case_output_path(plan, "stage_metrics", &case_dir)?,
        contamination_estimate: resolve_optional_case_output_path(
            plan,
            "contamination_estimate",
            &case_dir,
        ),
        contammix_report: resolve_optional_case_output_path(plan, "contammix_report", &case_dir),
        mt_consensus: resolve_optional_case_output_path(plan, "mt_consensus", &case_dir),
        advisory_boundary: case_dir.join("advisory_boundary.json"),
        contamination_modes: case_dir.join("contamination_modes.json"),
        contamination_stratified: case_dir.join("contamination.stratified.json"),
    })
}

fn write_local_contamination_optional_artifacts(
    case_paths: &LocalContaminationCasePaths,
    tool_id: &str,
    logical_scope: &str,
    proof_case: ProofCaseKind,
    raw_metrics: &bijux_dna_domain_bam::metrics::ContaminationMetricsV1,
    summary: &bijux_dna_domain_bam::BamContaminationEvidenceV1,
) -> Result<()> {
    if let Some(path) = &case_paths.contamination_estimate {
        bijux_dna_infra::atomic_write_json(
            path,
            &serde_json::json!({
                "stage_id": "bam.contamination",
                "tool_id": tool_id,
                "proof_case": proof_case.as_str(),
                "scope": logical_scope,
                "estimate": raw_metrics.estimate,
                "ci_low": raw_metrics.ci_low,
                "ci_high": raw_metrics.ci_high,
                "prerequisites_passed": summary.prerequisites_passed,
            }),
        )
        .with_context(|| format!("write {}", path.display()))?;
    }
    if let Some(path) = &case_paths.contammix_report {
        bijux_dna_infra::atomic_write_bytes(
            path,
            format!(
                "tool=contammix\nproof_case={}\nestimate={:.4}\nci_low={:.4}\nci_high={:.4}\n",
                proof_case.as_str(),
                raw_metrics.estimate,
                raw_metrics.ci_low,
                raw_metrics.ci_high
            )
            .as_bytes(),
        )
        .with_context(|| format!("write {}", path.display()))?;
    }
    if let Some(path) = &case_paths.mt_consensus {
        bijux_dna_infra::atomic_write_bytes(
            path,
            format!(">adna_contamination_consensus_{}\nACGTACGTACGTACGT\n", proof_case.as_str())
                .as_bytes(),
        )
        .with_context(|| format!("write {}", path.display()))?;
    }
    Ok(())
}

fn contamination_expectation_matched(
    summary: &bijux_dna_domain_bam::BamContaminationEvidenceV1,
    raw_metrics: &bijux_dna_domain_bam::metrics::ContaminationMetricsV1,
    proof_case: ProofCaseKind,
) -> bool {
    summary.prerequisites_passed == proof_case.expected_prerequisites_passed()
        && match proof_case {
            ProofCaseKind::Ready => {
                summary.estimate.is_some_and(|value| float_matches(value, raw_metrics.estimate))
                    && summary.ci_low.is_some_and(|value| float_matches(value, raw_metrics.ci_low))
                    && summary
                        .ci_high
                        .is_some_and(|value| float_matches(value, raw_metrics.ci_high))
                    && summary.refusal_codes.is_empty()
            }
            ProofCaseKind::Insufficient => {
                summary.estimate.is_none()
                    && summary.ci_low.is_none()
                    && summary.ci_high.is_none()
                    && !summary.refusal_codes.is_empty()
            }
        }
}

fn contamination_artifact_paths(
    repo_root: &Path,
    case_paths: &LocalContaminationCasePaths,
) -> Vec<String> {
    let mut artifact_paths = vec![
        path_relative_to_repo(repo_root, &case_paths.contamination_report),
        path_relative_to_repo(repo_root, &case_paths.contamination_summary),
        path_relative_to_repo(repo_root, &case_paths.stage_metrics),
        path_relative_to_repo(repo_root, &case_paths.advisory_boundary),
        path_relative_to_repo(repo_root, &case_paths.contamination_modes),
        path_relative_to_repo(repo_root, &case_paths.contamination_stratified),
    ];
    for path in
        [&case_paths.contamination_estimate, &case_paths.contammix_report, &case_paths.mt_consensus]
            .into_iter()
            .flatten()
    {
        artifact_paths.push(path_relative_to_repo(repo_root, path));
    }
    artifact_paths
}

fn raw_metrics(
    tool_id: &str,
    logical_scope: &str,
    tool_scope: &str,
    proof_case: ProofCaseKind,
) -> bijux_dna_domain_bam::metrics::ContaminationMetricsV1 {
    let (estimate, ci_low, ci_high) = match proof_case {
        ProofCaseKind::Ready => (0.02, 0.01, 0.03),
        ProofCaseKind::Insufficient => (0.08, 0.05, 0.12),
    };
    bijux_dna_domain_bam::metrics::ContaminationMetricsV1 {
        method: tool_id.to_string(),
        estimate,
        ci_low,
        ci_high,
        assumptions: vec![
            "governed local contamination smoke proof".to_string(),
            format!("logical_scope:{logical_scope}"),
            format!("tool_scope:{tool_scope}"),
            format!("proof_case:{}", proof_case.as_str()),
        ],
    }
}

fn contamination_summary(
    tool_id: &str,
    proof_case: ProofCaseKind,
    minimum_mean_coverage: f64,
) -> Result<bijux_dna_domain_bam::BamContaminationEvidenceV1> {
    let raw_metrics = raw_metrics(tool_id, "both", "both", proof_case);
    let mut metrics = bijux_dna_domain_bam::metrics::BamMetricsV1::empty();
    metrics.contamination = raw_metrics;
    metrics.coverage.mean = match proof_case {
        ProofCaseKind::Ready => minimum_mean_coverage + 3.5,
        ProofCaseKind::Insufficient => (minimum_mean_coverage - 0.25).max(0.0),
    };
    let evidence = match (tool_id, proof_case) {
        ("schmutzi", ProofCaseKind::Ready) => {
            bijux_dna_domain_bam::execute_mitochondrial_contamination_workflow(
                &metrics,
                true,
                true,
                minimum_mean_coverage,
            )
        }
        ("schmutzi", ProofCaseKind::Insufficient) => {
            bijux_dna_domain_bam::execute_mitochondrial_contamination_workflow(
                &metrics,
                false,
                false,
                minimum_mean_coverage,
            )
        }
        ("verifybamid2", ProofCaseKind::Ready) => {
            bijux_dna_domain_bam::execute_nuclear_contamination_workflow(
                &metrics,
                true,
                true,
                true,
                minimum_mean_coverage,
            )
        }
        ("verifybamid2", ProofCaseKind::Insufficient) => {
            bijux_dna_domain_bam::execute_nuclear_contamination_workflow(
                &metrics,
                true,
                true,
                false,
                minimum_mean_coverage,
            )
        }
        ("contammix", ProofCaseKind::Ready) => {
            let mut evidence = bijux_dna_domain_bam::execute_nuclear_contamination_workflow(
                &metrics,
                true,
                true,
                true,
                minimum_mean_coverage,
            );
            evidence.tool = "contammix".to_string();
            evidence
        }
        ("contammix", ProofCaseKind::Insufficient) => {
            let mut evidence = bijux_dna_domain_bam::execute_nuclear_contamination_workflow(
                &metrics,
                true,
                false,
                true,
                minimum_mean_coverage,
            );
            evidence.tool = "contammix".to_string();
            evidence
        }
        _ => return Err(anyhow!("unsupported local-smoke bam.contamination tool `{tool_id}`")),
    };
    Ok(evidence)
}

fn write_raw_contamination_report(
    path: &Path,
    raw_metrics: &bijux_dna_domain_bam::metrics::ContaminationMetricsV1,
) -> Result<()> {
    bijux_dna_infra::atomic_write_json(
        path,
        &serde_json::json!({
            "method": raw_metrics.method,
            "estimate": raw_metrics.estimate,
            "ci_low": raw_metrics.ci_low,
            "ci_high": raw_metrics.ci_high,
            "assumptions": raw_metrics.assumptions,
        }),
    )
    .with_context(|| format!("write {}", path.display()))
}

fn write_stage_metrics(
    path: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
    proof_case: ProofCaseKind,
    minimum_mean_coverage: f64,
    raw_metrics: &bijux_dna_domain_bam::metrics::ContaminationMetricsV1,
    summary: &bijux_dna_domain_bam::BamContaminationEvidenceV1,
) -> Result<()> {
    let expectation_matched = summary.prerequisites_passed
        == proof_case.expected_prerequisites_passed()
        && match proof_case {
            ProofCaseKind::Ready => {
                summary.estimate.is_some_and(|value| float_matches(value, raw_metrics.estimate))
                    && summary.ci_low.is_some_and(|value| float_matches(value, raw_metrics.ci_low))
                    && summary
                        .ci_high
                        .is_some_and(|value| float_matches(value, raw_metrics.ci_high))
            }
            ProofCaseKind::Insufficient => {
                summary.estimate.is_none()
                    && summary.ci_low.is_none()
                    && summary.ci_high.is_none()
                    && !summary.refusal_codes.is_empty()
            }
        };
    let reason = if summary.prerequisites_passed {
        "ready".to_string()
    } else {
        summary.refusal_codes.join(",")
    };
    bijux_dna_infra::atomic_write_json(
        path,
        &serde_json::json!({
            "schema_version": LOCAL_CONTAMINATION_STAGE_METRICS_SCHEMA_VERSION,
            "stage_id": "bam.contamination",
            "sample_id": plan.params.get("sample_id").and_then(serde_json::Value::as_str).unwrap_or("unknown"),
            "tool_id": plan.tool_id.as_str(),
            "proof_case": proof_case.as_str(),
            "scope": plan.params.get("scope").and_then(serde_json::Value::as_str).unwrap_or("both"),
            "tool_scope": plan.params.get("tool_scope").and_then(serde_json::Value::as_str).unwrap_or("both"),
            "minimum_mean_coverage": minimum_mean_coverage,
            "expected_prerequisites_passed": proof_case.expected_prerequisites_passed(),
            "prerequisites_passed": summary.prerequisites_passed,
            "raw_estimate": raw_metrics.estimate,
            "raw_ci_low": raw_metrics.ci_low,
            "raw_ci_high": raw_metrics.ci_high,
            "reported_estimate": summary.estimate,
            "reported_ci_low": summary.ci_low,
            "reported_ci_high": summary.ci_high,
            "refusal_codes": summary.refusal_codes,
            "reason": reason,
            "expectation_matched": expectation_matched,
        }),
    )
    .with_context(|| format!("write {}", path.display()))
}

fn write_contamination_modes(
    path: &Path,
    logical_scope: &str,
    tool_scope: &str,
    plan: &bijux_dna_stage_contract::StagePlanV1,
) -> Result<()> {
    bijux_dna_infra::atomic_write_json(
        path,
        &serde_json::json!({
            "logical_scope": logical_scope,
            "tool_scope": tool_scope,
            "mitochondrial_mode": tool_scope == "mt" || tool_scope == "both",
            "nuclear_mode": tool_scope == "nuclear" || tool_scope == "both",
            "sex_chr_mode": plan.params.get("sex_specific").and_then(serde_json::Value::as_bool).unwrap_or(false),
        }),
    )
    .with_context(|| format!("write {}", path.display()))
}

fn write_contamination_stratified(
    path: &Path,
    tool_id: &str,
    tool_scope: &str,
    estimate: f64,
) -> Result<()> {
    bijux_dna_infra::atomic_write_json(
        path,
        &serde_json::json!({
            "schema_version": "bijux.bam.contamination_stratified.v1",
            "method": tool_id,
            "scope": tool_scope,
            "mt_estimate": (tool_scope == "mt" || tool_scope == "both").then_some(estimate),
            "nuclear_estimate": (tool_scope == "nuclear" || tool_scope == "both").then_some(estimate),
            "global_estimate": estimate,
        }),
    )
    .with_context(|| format!("write {}", path.display()))
}

fn resolve_case_output_path(
    plan: &bijux_dna_stage_contract::StagePlanV1,
    output_id: &str,
    case_dir: &Path,
) -> Result<PathBuf> {
    let artifact = plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == output_id)
        .ok_or_else(|| {
            anyhow!("bam.contamination local-smoke plan is missing governed output `{output_id}`")
        })?;
    let file_name = artifact.path.file_name().ok_or_else(|| {
        anyhow!(
            "bam.contamination local-smoke output `{output_id}` has no file name: {}",
            artifact.path.display()
        )
    })?;
    Ok(case_dir.join(file_name))
}

fn resolve_optional_case_output_path(
    plan: &bijux_dna_stage_contract::StagePlanV1,
    output_id: &str,
    case_dir: &Path,
) -> Option<PathBuf> {
    plan.io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == output_id)
        .and_then(|artifact| artifact.path.file_name().map(|file_name| case_dir.join(file_name)))
}

fn resolve_input_path(
    repo_root: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
    input_id: &str,
) -> Result<PathBuf> {
    let path = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == input_id)
        .map(|artifact| artifact.path.clone())
        .ok_or_else(|| anyhow!("bam.contamination plan is missing governed input `{input_id}`"))?;
    Ok(resolve_plan_path(repo_root, &path))
}

fn resolve_plan_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn required_string_param(
    plan: &bijux_dna_stage_contract::StagePlanV1,
    key: &str,
) -> Result<String> {
    plan.params
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("bam.contamination plan is missing string param `{key}`"))
}

fn required_f64_param(plan: &bijux_dna_stage_contract::StagePlanV1, key: &str) -> Result<f64> {
    plan.params
        .get(key)
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| anyhow!("bam.contamination plan is missing numeric param `{key}`"))
}

fn relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(repo_root).unwrap_or(path).to_path_buf()
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    relative_path(repo_root, path).display().to_string()
}

fn float_matches(left: f64, right: f64) -> bool {
    (left - right).abs() <= 1e-9
}
