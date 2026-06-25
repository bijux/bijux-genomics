#![cfg(feature = "bam_downstream")]

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_bam::metrics::BamMetricsV1;
use serde::Serialize;

const LOCAL_HAPLOGROUPS_SMOKE_REPORT_SCHEMA_VERSION: &str =
    "bijux.bam.haplogroups.local_smoke.report.v1";
const LOCAL_HAPLOGROUPS_STAGE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bam.haplogroups.stage_metrics.v1";
const HAPLOGROUPS_REPORT_SCHEMA_VERSION: &str = "bijux.bam.haplogroups.v1";
const EXPECTED_TOOL_ID: &str = "yleaf";
const EXPECTED_STAGE_ID: &str = "bam.haplogroups";
const EXPECTED_SAMPLE_ID: &str = "adna_y_haplogroup_panel";
const READY_STATUS: &str = "ready";
const INSUFFICIENT_STATUS: &str = "coverage_gate_not_met";
const READY_CASE: &str = "ready";
const INSUFFICIENT_CASE: &str = "insufficient";
const INSUFFICIENT_COVERAGE_GATE: f64 = 2.5;
const CONTAMINATION_ESTIMATE: f64 = 0.02;

#[derive(Debug, Clone, Serialize)]
struct LocalHaplogroupsSmokeReport {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    sample_id: String,
    reference_panel_id: String,
    reference_build: String,
    case_count: usize,
    rows: Vec<LocalHaplogroupsSmokeCaseReport>,
}

#[derive(Debug, Clone, Serialize)]
struct LocalHaplogroupsSmokeCaseReport {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    proof_case: String,
    sample_id: String,
    expectation_matched: bool,
    input_bam: String,
    reference_fasta: String,
    reference_panel: String,
    reference_panel_id: String,
    reference_build: String,
    population_scope: String,
    minimum_coverage: f64,
    observed_mean_coverage: f64,
    contamination_estimate: f64,
    ready: bool,
    haplogroup_call: Option<String>,
    confidence: f64,
    status: String,
    markers_total: usize,
    markers_supported: usize,
    supported_marker_ids: Vec<String>,
    refusal_codes: Vec<String>,
    caveats: Vec<String>,
    haplogroups_report: String,
    haplogroups_summary: String,
    haplogroup_report: String,
    stage_metrics: String,
    declared_output_ids: Vec<String>,
    artifact_paths: Vec<String>,
}

#[derive(Debug, Clone)]
struct HaplogroupMarker {
    marker_id: String,
    contig: String,
    position: u64,
    haplogroup: String,
    lineage_scope: String,
}

struct LocalHaplogroupsOutputPaths {
    haplogroups_report: PathBuf,
    haplogroups_summary: PathBuf,
    haplogroup_report: PathBuf,
    stage_metrics: PathBuf,
}

struct LocalHaplogroupsCaseArtifacts {
    reference_panel: PathBuf,
    reference_panel_id: String,
    reference_build: String,
    population_scope: String,
    minimum_coverage: f64,
    observed_mean_coverage: f64,
    haplogroup_call: Option<String>,
    confidence: f64,
    status: String,
    ready: bool,
    markers_total: usize,
    markers_supported: usize,
    supported_marker_ids: Vec<String>,
    lineage_scope: Option<String>,
    summary: bijux_dna_domain_bam::BamHaplogroupReadinessV1,
    output_paths: LocalHaplogroupsOutputPaths,
    expectation_matched: bool,
    declared_output_ids: Vec<String>,
    artifact_paths: Vec<String>,
}

/// Materialize governed local-smoke `bam.haplogroups` artifacts and report bundle.
///
/// The written report lives at `runs/bench/local-smoke/bam.haplogroups/haplogroups.json`
/// under the active repository root.
///
/// # Errors
/// Returns an error if the repository root cannot be resolved, the governed local-ready plan is
/// invalid, panel inputs are missing, or the proof artifacts cannot be written.
#[cfg(feature = "bam_downstream")]
pub fn write_local_haplogroups_smoke_report() -> Result<PathBuf> {
    let repo_root = crate::support::workspace::resolve_repo_root()?;
    let plan = bijux_dna_planner_bam::stage_api::local_haplogroups_plan(&repo_root)?;
    if plan.stage_id.as_str() != EXPECTED_STAGE_ID {
        return Err(anyhow!(
            "local-ready haplogroups plan drifted to stage `{}`",
            plan.stage_id.as_str()
        ));
    }
    if plan.tool_id.as_str() != EXPECTED_TOOL_ID {
        return Err(anyhow!("local-ready haplogroups tool drifted to `{}`", plan.tool_id.as_str()));
    }

    let output_root = repo_root.join("runs/bench/local-smoke/bam.haplogroups");
    bijux_dna_infra::ensure_dir(&output_root)?;

    let ready_case = materialize_local_haplogroups_case(&repo_root, &plan, READY_CASE)?;
    let insufficient_case =
        materialize_local_haplogroups_case(&repo_root, &plan, INSUFFICIENT_CASE)?;

    let report = LocalHaplogroupsSmokeReport {
        schema_version: LOCAL_HAPLOGROUPS_SMOKE_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: EXPECTED_STAGE_ID.to_string(),
        tool_id: EXPECTED_TOOL_ID.to_string(),
        sample_id: EXPECTED_SAMPLE_ID.to_string(),
        reference_panel_id: ready_case.reference_panel_id.clone(),
        reference_build: ready_case.reference_build.clone(),
        case_count: 2,
        rows: vec![ready_case, insufficient_case],
    };
    let report_path = output_root.join("haplogroups.json");
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;
    Ok(report_path)
}

#[cfg(feature = "bam_downstream")]
fn materialize_local_haplogroups_case(
    repo_root: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
    proof_case: &str,
) -> Result<LocalHaplogroupsSmokeCaseReport> {
    let case_dir = repo_root.join("runs/bench/local-smoke/bam.haplogroups").join(proof_case);
    bijux_dna_infra::ensure_dir(&case_dir)?;

    let input_bam = resolve_input_path(repo_root, plan, "bam")?;
    let reference_fasta = resolve_input_path(repo_root, plan, "reference")?;
    let reference_panel = resolve_input_path(repo_root, plan, "reference_panel")?;
    let reference_build = required_plan_string(plan, "reference_build")?;
    let reference_panel_id = required_plan_string(plan, "reference_panel_id")?;
    let population_scope = required_plan_string(plan, "population_scope")?;
    let ready_minimum_coverage = required_minimum_coverage(plan)?;
    let minimum_coverage =
        if proof_case == READY_CASE { ready_minimum_coverage } else { INSUFFICIENT_COVERAGE_GATE };
    let observed_mean_coverage = observed_mean_coverage(&input_bam)?;
    let markers = load_haplogroup_panel(&reference_panel)?;
    let supported_markers = covered_markers_from_sam(&input_bam, &markers)?;
    let markers_total = markers.len();
    let markers_supported = supported_markers.len();
    let ready = proof_case == READY_CASE;
    let haplogroup_call =
        if ready { select_haplogroup_call(&supported_markers) } else { None::<String> };
    let confidence = if ready && markers_total > 0 {
        let supported = f64::from(u32::try_from(markers_supported).unwrap_or(u32::MAX));
        let total = f64::from(u32::try_from(markers_total).unwrap_or(u32::MAX));
        supported / total
    } else {
        0.0
    };
    let status = if ready { READY_STATUS } else { INSUFFICIENT_STATUS };

    let mut metrics = BamMetricsV1::empty();
    metrics.coverage.mean = observed_mean_coverage;
    metrics.contamination.estimate = CONTAMINATION_ESTIMATE;
    metrics.haplogroup_sufficiency.sufficient =
        markers_supported == markers_total && markers_total > 0;
    metrics.haplogroup_sufficiency.min_coverage = minimum_coverage;
    metrics.haplogroup_sufficiency.reason = if metrics.haplogroup_sufficiency.sufficient {
        "markers_supported".to_string()
    } else {
        "markers_missing".to_string()
    };
    let summary = bijux_dna_domain_bam::evaluate_haplogroup_readiness(
        &metrics,
        Some(reference_build.as_str()),
        true,
    );

    let output_paths = LocalHaplogroupsOutputPaths {
        haplogroups_report: case_dir.join("haplogroups.json"),
        haplogroups_summary: case_dir.join("haplogroups.summary.json"),
        haplogroup_report: case_dir.join("haplogroup_report.json"),
        stage_metrics: case_dir.join("stage.metrics.json"),
    };

    let expectation_matched = haplogroups_expectation_matched(
        &summary,
        ready,
        minimum_coverage,
        observed_mean_coverage,
        &reference_build,
    );
    let declared_output_ids =
        plan.io.outputs.iter().map(|artifact| artifact.name.to_string()).collect();
    let artifact_paths = vec![
        path_relative_to_repo(repo_root, &output_paths.haplogroups_report),
        path_relative_to_repo(repo_root, &output_paths.haplogroups_summary),
        path_relative_to_repo(repo_root, &output_paths.haplogroup_report),
        path_relative_to_repo(repo_root, &output_paths.stage_metrics),
    ];
    let artifacts = LocalHaplogroupsCaseArtifacts {
        reference_panel,
        reference_panel_id,
        reference_build,
        population_scope,
        minimum_coverage,
        observed_mean_coverage,
        haplogroup_call,
        confidence,
        status: status.to_string(),
        ready: summary.ready,
        markers_total,
        markers_supported,
        supported_marker_ids: supported_markers
            .iter()
            .map(|marker| marker.marker_id.clone())
            .collect(),
        lineage_scope: supported_markers.last().map(|marker| marker.lineage_scope.clone()),
        summary,
        output_paths,
        expectation_matched,
        declared_output_ids,
        artifact_paths,
    };

    write_local_haplogroups_case_report(
        repo_root,
        proof_case,
        &input_bam,
        &reference_fasta,
        artifacts,
    )
}

#[cfg(feature = "bam_downstream")]
fn haplogroups_expectation_matched(
    summary: &bijux_dna_domain_bam::BamHaplogroupReadinessV1,
    ready: bool,
    minimum_coverage: f64,
    observed_mean_coverage: f64,
    reference_build: &str,
) -> bool {
    summary.ready == ready
        && float_matches(summary.minimum_coverage, minimum_coverage)
        && float_matches(summary.observed_mean_coverage, observed_mean_coverage)
        && summary.reference_build.as_deref() == Some(reference_build)
        && summary.contamination_estimate == Some(CONTAMINATION_ESTIMATE)
        && ((ready && summary.refusal_codes.is_empty())
            || (!ready
                && summary.refusal_codes == vec!["coverage_below_haplogroup_minimum".to_string()]))
}

#[allow(clippy::too_many_arguments)]
#[cfg(feature = "bam_downstream")]
fn write_local_haplogroups_outputs(
    repo_root: &Path,
    proof_case: &str,
    reference_panel: &Path,
    reference_panel_id: &str,
    reference_build: &str,
    population_scope: &str,
    minimum_coverage: f64,
    observed_mean_coverage: f64,
    haplogroup_call: Option<&String>,
    confidence: f64,
    status: &str,
    ready: bool,
    markers_total: usize,
    markers_supported: usize,
    supported_marker_ids: &[String],
    lineage_scope: Option<&str>,
    summary: &bijux_dna_domain_bam::BamHaplogroupReadinessV1,
    output_paths: &LocalHaplogroupsOutputPaths,
    expectation_matched: bool,
) -> Result<()> {
    bijux_dna_infra::atomic_write_json(
        &output_paths.haplogroups_report,
        &serde_json::json!({
            "schema_version": HAPLOGROUPS_REPORT_SCHEMA_VERSION,
            "stage_id": EXPECTED_STAGE_ID,
            "tool_id": EXPECTED_TOOL_ID,
            "proof_case": proof_case,
            "sample_id": EXPECTED_SAMPLE_ID,
            "reference_panel_id": reference_panel_id,
            "reference_panel": path_relative_to_repo(repo_root, reference_panel),
            "reference_build": reference_build,
            "population_scope": population_scope,
            "coverage_gate": {
                "min_coverage": minimum_coverage,
                "observed_mean_coverage": observed_mean_coverage,
            },
            "haplogroup_call": haplogroup_call,
            "confidence": confidence,
            "status": status,
            "markers_total": markers_total,
            "markers_supported": markers_supported,
            "supported_marker_ids": supported_marker_ids,
            "lineage_scope": lineage_scope,
        }),
    )?;
    bijux_dna_infra::atomic_write_json(&output_paths.haplogroups_summary, summary)?;
    bijux_dna_infra::atomic_write_json(
        &output_paths.haplogroup_report,
        &serde_json::json!({
            "artifact_id": "haplogroup_report",
            "stage_id": EXPECTED_STAGE_ID,
            "tool_id": EXPECTED_TOOL_ID,
            "proof_case": proof_case,
            "haplogroup": haplogroup_call,
            "confidence": confidence,
            "status": status,
            "reference_panel_id": reference_panel_id,
            "markers_total": markers_total,
            "markers_supported": markers_supported,
            "supported_marker_ids": supported_marker_ids,
        }),
    )?;
    bijux_dna_infra::atomic_write_json(
        &output_paths.stage_metrics,
        &serde_json::json!({
            "schema_version": LOCAL_HAPLOGROUPS_STAGE_METRICS_SCHEMA_VERSION,
            "stage_id": EXPECTED_STAGE_ID,
            "tool_id": EXPECTED_TOOL_ID,
            "sample_id": EXPECTED_SAMPLE_ID,
            "proof_case": proof_case,
            "reference_panel_id": reference_panel_id,
            "reference_build": reference_build,
            "population_scope": population_scope,
            "expected_ready": ready,
            "ready": summary.ready,
            "minimum_coverage": minimum_coverage,
            "observed_mean_coverage": observed_mean_coverage,
            "haplogroup_call": haplogroup_call,
            "confidence": confidence,
            "status": status,
            "markers_total": markers_total,
            "markers_supported": markers_supported,
            "supported_marker_ids": supported_marker_ids,
            "refusal_codes": summary.refusal_codes,
            "expectation_matched": expectation_matched,
        }),
    )?;
    Ok(())
}

#[cfg(feature = "bam_downstream")]
fn write_local_haplogroups_case_report(
    repo_root: &Path,
    proof_case: &str,
    input_bam: &Path,
    reference_fasta: &Path,
    artifacts: LocalHaplogroupsCaseArtifacts,
) -> Result<LocalHaplogroupsSmokeCaseReport> {
    write_local_haplogroups_outputs(
        repo_root,
        proof_case,
        &artifacts.reference_panel,
        &artifacts.reference_panel_id,
        &artifacts.reference_build,
        &artifacts.population_scope,
        artifacts.minimum_coverage,
        artifacts.observed_mean_coverage,
        artifacts.haplogroup_call.as_ref(),
        artifacts.confidence,
        &artifacts.status,
        artifacts.ready,
        artifacts.markers_total,
        artifacts.markers_supported,
        &artifacts.supported_marker_ids,
        artifacts.lineage_scope.as_deref(),
        &artifacts.summary,
        &artifacts.output_paths,
        artifacts.expectation_matched,
    )?;

    Ok(LocalHaplogroupsSmokeCaseReport {
        schema_version: LOCAL_HAPLOGROUPS_SMOKE_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: EXPECTED_STAGE_ID.to_string(),
        tool_id: EXPECTED_TOOL_ID.to_string(),
        proof_case: proof_case.to_string(),
        sample_id: EXPECTED_SAMPLE_ID.to_string(),
        expectation_matched: artifacts.expectation_matched,
        input_bam: path_relative_to_repo(repo_root, input_bam),
        reference_fasta: path_relative_to_repo(repo_root, reference_fasta),
        reference_panel: path_relative_to_repo(repo_root, &artifacts.reference_panel),
        reference_panel_id: artifacts.reference_panel_id,
        reference_build: artifacts.reference_build,
        population_scope: artifacts.population_scope,
        minimum_coverage: artifacts.minimum_coverage,
        observed_mean_coverage: artifacts.observed_mean_coverage,
        contamination_estimate: CONTAMINATION_ESTIMATE,
        ready: artifacts.summary.ready,
        haplogroup_call: artifacts.haplogroup_call,
        confidence: artifacts.confidence,
        status: artifacts.status,
        markers_total: artifacts.markers_total,
        markers_supported: artifacts.markers_supported,
        supported_marker_ids: artifacts.supported_marker_ids,
        refusal_codes: artifacts.summary.refusal_codes.clone(),
        caveats: artifacts.summary.caveats.clone(),
        haplogroups_report: path_relative_to_repo(
            repo_root,
            &artifacts.output_paths.haplogroups_report,
        ),
        haplogroups_summary: path_relative_to_repo(
            repo_root,
            &artifacts.output_paths.haplogroups_summary,
        ),
        haplogroup_report: path_relative_to_repo(
            repo_root,
            &artifacts.output_paths.haplogroup_report,
        ),
        stage_metrics: path_relative_to_repo(repo_root, &artifacts.output_paths.stage_metrics),
        declared_output_ids: artifacts.declared_output_ids,
        artifact_paths: artifacts.artifact_paths,
    })
}

#[cfg(feature = "bam_downstream")]
fn observed_mean_coverage(input_bam: &Path) -> Result<f64> {
    let summary = bijux_dna_domain_bam::summarize_tiny_bam_coverage(input_bam, &[1])?;
    summary
        .mean_depth
        .ok_or_else(|| anyhow!("haplogroups smoke coverage summary is missing mean depth"))
}

#[cfg(feature = "bam_downstream")]
fn load_haplogroup_panel(reference_panel: &Path) -> Result<Vec<HaplogroupMarker>> {
    let raw = fs::read_to_string(reference_panel)
        .with_context(|| format!("read {}", reference_panel.display()))?;
    let mut markers = Vec::new();
    for line in raw.lines() {
        if line.is_empty() || line.starts_with('#') || line.starts_with("marker_id\t") {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() < 7 {
            return Err(anyhow!("haplogroup panel row must have 7 tab-delimited fields: `{line}`"));
        }
        markers.push(HaplogroupMarker {
            marker_id: fields[0].to_string(),
            contig: fields[1].to_string(),
            position: fields[2]
                .parse::<u64>()
                .with_context(|| format!("parse haplogroup panel position `{}`", fields[2]))?,
            haplogroup: fields[5].to_string(),
            lineage_scope: fields[6].to_string(),
        });
    }
    if markers.is_empty() {
        return Err(anyhow!(
            "haplogroup panel `{}` must carry at least one marker row",
            reference_panel.display()
        ));
    }
    Ok(markers)
}

#[cfg(feature = "bam_downstream")]
fn covered_markers_from_sam(
    input_bam: &Path,
    markers: &[HaplogroupMarker],
) -> Result<Vec<HaplogroupMarker>> {
    let raw =
        fs::read_to_string(input_bam).with_context(|| format!("read {}", input_bam.display()))?;
    let mut supported = Vec::new();
    for marker in markers {
        let covered = raw.lines().filter(|line| !line.starts_with('@')).any(|line| {
            let fields = line.split('\t').collect::<Vec<_>>();
            if fields.len() < 11 || fields[2] != marker.contig {
                return false;
            }
            let Ok(start) = fields[3].parse::<u64>() else {
                return false;
            };
            let read_len = fields[9].len() as u64;
            let end = start.saturating_add(read_len.saturating_sub(1));
            marker.position >= start && marker.position <= end
        });
        if covered {
            supported.push(marker.clone());
        }
    }
    Ok(supported)
}

#[cfg(feature = "bam_downstream")]
fn select_haplogroup_call(markers: &[HaplogroupMarker]) -> Option<String> {
    markers
        .iter()
        .max_by_key(|marker| (marker.haplogroup.len(), marker.position))
        .map(|marker| marker.haplogroup.clone())
}

#[cfg(feature = "bam_downstream")]
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
        .ok_or_else(|| anyhow!("bam.haplogroups local-ready plan is missing input `{input_id}`"))?;
    Ok(resolve_plan_path(repo_root, &path))
}

#[cfg(feature = "bam_downstream")]
fn required_plan_string(plan: &bijux_dna_stage_contract::StagePlanV1, key: &str) -> Result<String> {
    plan.params
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("bam.haplogroups local-ready plan is missing string param `{key}`"))
}

#[cfg(feature = "bam_downstream")]
fn required_minimum_coverage(plan: &bijux_dna_stage_contract::StagePlanV1) -> Result<f64> {
    plan.params
        .get("coverage_gate")
        .and_then(|value| value.get("min_coverage"))
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| {
            anyhow!("bam.haplogroups local-ready plan is missing `coverage_gate.min_coverage`")
        })
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
fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

#[cfg(feature = "bam_downstream")]
fn float_matches(left: f64, right: f64) -> bool {
    (left - right).abs() <= 1e-9
}
