use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use super::bam_contamination_sex_haplogroups_ready::{
    render_bam_contamination_sex_haplogroups_ready, BamContaminationSexHaplogroupsReadyRow,
    DEFAULT_BAM_CONTAMINATION_SEX_HAPLOGROUPS_READY_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_BAM_CONTAMINATION_COMPLETE_PATH: &str =
    "benchmarks/readiness/bam/stages/bam.contamination.complete.json";
const BAM_CONTAMINATION_COMPLETE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_contamination_complete.v1";
const EXPECTED_STAGE_ID: &str = "bam.contamination";
const EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION: &str = "bijux.bam.contamination.local_smoke.report.v1";
const EXPECTED_STAGE_METRICS_SCHEMA_VERSION: &str = "bijux.bam.contamination.stage_metrics.v1";
const EXPECTED_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.contamination_evidence.v1";
const CHECKED_SURFACE_COUNT: usize = 14;
const EXPECTED_TOOL_IDS: [&str; 3] = ["contammix", "schmutzi", "verifybamid2"];
const REQUIRED_OUTPUT_IDS: [&str; 3] = ["contamination_report", "summary", "stage_metrics"];

#[derive(Debug, Clone, Deserialize)]
struct LocalContaminationSmokeReport {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    tool_ids: Vec<String>,
    case_count: usize,
    rows: Vec<LocalContaminationSmokeCaseReport>,
}

#[derive(Debug, Clone, Deserialize)]
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
    contamination_estimate: Option<String>,
    contammix_report: Option<String>,
    mt_consensus: Option<String>,
    advisory_boundary: String,
    contamination_modes: String,
    contamination_stratified: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamContaminationCompleteRow {
    pub(crate) result_id: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) sample_scope: String,
    pub(crate) benchmark_status: String,
    pub(crate) support_status: String,
    pub(crate) adapter_status: String,
    pub(crate) parser_status: String,
    pub(crate) corpus_status: String,
    pub(crate) report_section_id: String,
    pub(crate) summary_table_id: String,
    pub(crate) command_readiness_kind: String,
    pub(crate) required_output_ids: Vec<String>,
    pub(crate) stage_output_ids: Vec<String>,
    pub(crate) expected_schema_extension_id: String,
    pub(crate) schema_extension_id: String,
    pub(crate) active_scope_proof_path: String,
    pub(crate) command_proof_path: String,
    pub(crate) output_contract_proof_path: String,
    pub(crate) parser_proof_path: String,
    pub(crate) expected_result_proof_path: String,
    pub(crate) report_map_proof_path: String,
    pub(crate) schema_proof_path: String,
    pub(crate) local_smoke_proof_path: String,
    pub(crate) ready_case_report_path: String,
    pub(crate) ready_case_summary_path: String,
    pub(crate) ready_case_stage_metrics_path: String,
    pub(crate) insufficient_case_summary_path: String,
    pub(crate) insufficient_case_stage_metrics_path: String,
    pub(crate) ready_case_raw_method: String,
    pub(crate) ready_case_raw_estimate: f64,
    pub(crate) ready_case_raw_ci_low: f64,
    pub(crate) ready_case_raw_ci_high: f64,
    pub(crate) ready_case_summary_schema_version: String,
    pub(crate) insufficient_case_summary_schema_version: String,
    pub(crate) ready_case_prerequisites_passed: bool,
    pub(crate) insufficient_case_prerequisites_passed: bool,
    pub(crate) insufficient_case_refusal_codes: Vec<String>,
    pub(crate) ready_case_stage_metrics_schema_version: String,
    pub(crate) insufficient_case_stage_metrics_schema_version: String,
    pub(crate) local_smoke_sample_id: String,
    pub(crate) local_smoke_ready_case_scope: String,
    pub(crate) local_smoke_ready_case_tool_scope: String,
    pub(crate) local_smoke_input_bam: String,
    pub(crate) local_smoke_reference_fasta: String,
    pub(crate) local_smoke_minimum_mean_coverage: f64,
    pub(crate) tool_specific_artifact_path: Option<String>,
    pub(crate) active_scope_ready: bool,
    pub(crate) command_ready: bool,
    pub(crate) output_ready: bool,
    pub(crate) parser_ready: bool,
    pub(crate) expected_result_ready: bool,
    pub(crate) report_ready: bool,
    pub(crate) schema_ready: bool,
    pub(crate) local_smoke_ready: bool,
    pub(crate) ready_case_ready: bool,
    pub(crate) parser_contract_ready: bool,
    pub(crate) ready_summary_ready: bool,
    pub(crate) ready_stage_metrics_ready: bool,
    pub(crate) insufficient_summary_ready: bool,
    pub(crate) insufficient_stage_metrics_ready: bool,
    pub(crate) insufficiency_behavior_ready: bool,
    pub(crate) coverage_status: String,
    pub(crate) missing_surfaces: Vec<String>,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamContaminationCompleteReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) active_row_count: usize,
    pub(crate) complete_row_count: usize,
    pub(crate) incomplete_row_count: usize,
    pub(crate) checked_surface_count: usize,
    pub(crate) expected_tool_ids: Vec<String>,
    pub(crate) required_output_ids: Vec<String>,
    pub(crate) local_smoke_sample_id: String,
    pub(crate) local_smoke_case_count: usize,
    pub(crate) toolset_ready: bool,
    pub(crate) violation_count: usize,
    pub(crate) ok: bool,
    pub(crate) rows: Vec<BamContaminationCompleteRow>,
    pub(crate) violations: Vec<BamContaminationCompleteRow>,
}

pub(crate) fn run_render_bam_contamination_complete(
    args: &parse::BenchReadinessRenderBamContaminationCompleteArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_contamination_complete(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_CONTAMINATION_COMPLETE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_bam_contamination_complete(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamContaminationCompleteReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let report = build_bam_contamination_complete_report(repo_root, &output_path)?;
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    if !report.ok {
        return Err(anyhow!(
            "bam contamination completion must keep active scope, command, output, parser, expected-result, report, schema, ready smoke, and insufficiency smoke for all retained tools"
        ));
    }
    Ok(report)
}

fn build_bam_contamination_complete_report(
    repo_root: &Path,
    output_path: &Path,
) -> Result<BamContaminationCompleteReport> {
    let readiness = render_bam_contamination_sex_haplogroups_ready(
        repo_root,
        PathBuf::from(DEFAULT_BAM_CONTAMINATION_SEX_HAPLOGROUPS_READY_PATH),
    )?;
    let smoke_path = bijux_dna_api::v1::api::bam::write_local_contamination_smoke_report()?;
    let smoke_report: LocalContaminationSmokeReport = serde_json::from_str(
        &fs::read_to_string(&smoke_path)
            .with_context(|| format!("read {}", smoke_path.display()))?,
    )
    .with_context(|| format!("parse {}", smoke_path.display()))?;

    if smoke_report.schema_version != EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION {
        return Err(anyhow!(
            "unexpected bam.contamination local-smoke schema `{}`",
            smoke_report.schema_version
        ));
    }
    if smoke_report.stage_id != EXPECTED_STAGE_ID {
        return Err(anyhow!(
            "unexpected bam.contamination local-smoke stage `{}`",
            smoke_report.stage_id
        ));
    }

    let mut smoke_tool_ids = smoke_report.tool_ids.clone();
    smoke_tool_ids.sort();
    let expected_tool_ids =
        EXPECTED_TOOL_IDS.iter().map(|value| (*value).to_string()).collect::<Vec<_>>();
    if smoke_tool_ids != expected_tool_ids {
        return Err(anyhow!(
            "bam.contamination local-smoke tool ids drifted: observed={:?} expected={:?}",
            smoke_tool_ids,
            expected_tool_ids
        ));
    }

    let ready_cases = smoke_report
        .rows
        .iter()
        .filter(|row| row.proof_case == "ready")
        .map(|row| (row.tool_id.clone(), row.clone()))
        .collect::<BTreeMap<_, _>>();
    let insufficient_cases = smoke_report
        .rows
        .iter()
        .filter(|row| row.proof_case == "insufficient")
        .map(|row| (row.tool_id.clone(), row.clone()))
        .collect::<BTreeMap<_, _>>();

    let mut rows = readiness
        .rows
        .into_iter()
        .filter(|row| row.stage_id == EXPECTED_STAGE_ID)
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));
    let observed_tool_ids = rows.iter().map(|row| row.tool_id.clone()).collect::<Vec<_>>();
    if observed_tool_ids != expected_tool_ids {
        return Err(anyhow!(
            "bam.contamination readiness rows drifted: observed={:?} expected={:?}",
            observed_tool_ids,
            expected_tool_ids
        ));
    }

    let mut report_rows = Vec::with_capacity(rows.len());
    for readiness_row in rows {
        let ready_case = ready_cases.get(&readiness_row.tool_id).ok_or_else(|| {
            anyhow!("missing bam.contamination ready smoke case for `{}`", readiness_row.tool_id)
        })?;
        let insufficient_case =
            insufficient_cases.get(&readiness_row.tool_id).ok_or_else(|| {
                anyhow!(
                    "missing bam.contamination insufficient smoke case for `{}`",
                    readiness_row.tool_id
                )
            })?;
        report_rows.push(build_bam_contamination_complete_row(
            repo_root,
            &smoke_path,
            &smoke_report,
            &readiness_row,
            ready_case,
            insufficient_case,
        )?);
    }

    let complete_row_count =
        report_rows.iter().filter(|row| row.coverage_status == "complete").count();
    let incomplete_row_count = report_rows.len().saturating_sub(complete_row_count);
    let violations = report_rows
        .iter()
        .filter(|row| row.coverage_status != "complete")
        .cloned()
        .collect::<Vec<_>>();

    Ok(BamContaminationCompleteReport {
        schema_version: BAM_CONTAMINATION_COMPLETE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        active_row_count: report_rows.len(),
        complete_row_count,
        incomplete_row_count,
        checked_surface_count: CHECKED_SURFACE_COUNT,
        expected_tool_ids,
        required_output_ids: REQUIRED_OUTPUT_IDS.iter().map(|value| (*value).to_string()).collect(),
        local_smoke_sample_id: smoke_report.sample_id,
        local_smoke_case_count: smoke_report.case_count,
        toolset_ready: violations.is_empty(),
        violation_count: violations.len(),
        ok: violations.is_empty(),
        rows: report_rows,
        violations,
    })
}

fn build_bam_contamination_complete_row(
    repo_root: &Path,
    smoke_path: &Path,
    smoke_report: &LocalContaminationSmokeReport,
    readiness_row: &BamContaminationSexHaplogroupsReadyRow,
    ready_case: &LocalContaminationSmokeCaseReport,
    insufficient_case: &LocalContaminationSmokeCaseReport,
) -> Result<BamContaminationCompleteRow> {
    let ready_report_path = repo_root.join(&ready_case.contamination_report);
    let ready_summary_path = repo_root.join(&ready_case.contamination_summary);
    let ready_stage_metrics_path = repo_root.join(&ready_case.stage_metrics);
    let insufficient_summary_path = repo_root.join(&insufficient_case.contamination_summary);
    let insufficient_stage_metrics_path = repo_root.join(&insufficient_case.stage_metrics);

    let ready_raw_metrics =
        bijux_dna_domain_bam::metrics::parse_contamination_json(&ready_report_path)
            .with_context(|| format!("parse {}", ready_report_path.display()))?;
    let ready_summary: bijux_dna_domain_bam::BamContaminationEvidenceV1 = serde_json::from_str(
        &fs::read_to_string(&ready_summary_path)
            .with_context(|| format!("read {}", ready_summary_path.display()))?,
    )
    .with_context(|| format!("parse {}", ready_summary_path.display()))?;
    let insufficient_summary: bijux_dna_domain_bam::BamContaminationEvidenceV1 =
        serde_json::from_str(
            &fs::read_to_string(&insufficient_summary_path)
                .with_context(|| format!("read {}", insufficient_summary_path.display()))?,
        )
        .with_context(|| format!("parse {}", insufficient_summary_path.display()))?;
    let ready_stage_metrics: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&ready_stage_metrics_path)
            .with_context(|| format!("read {}", ready_stage_metrics_path.display()))?,
    )
    .with_context(|| format!("parse {}", ready_stage_metrics_path.display()))?;
    let insufficient_stage_metrics: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&insufficient_stage_metrics_path)
            .with_context(|| format!("read {}", insufficient_stage_metrics_path.display()))?,
    )
    .with_context(|| format!("parse {}", insufficient_stage_metrics_path.display()))?;

    let local_smoke_ready = ready_case.schema_version == EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION
        && insufficient_case.schema_version == EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION
        && ready_case.stage_id == EXPECTED_STAGE_ID
        && insufficient_case.stage_id == EXPECTED_STAGE_ID
        && ready_case.expectation_matched
        && insufficient_case.expectation_matched
        && ready_case.sample_id == smoke_report.sample_id
        && insufficient_case.sample_id == smoke_report.sample_id
        && required_outputs_present(ready_case)
        && required_outputs_present(insufficient_case)
        && tool_specific_output_path(ready_case).is_some()
        && tool_specific_output_path(insufficient_case).is_some()
        && !ready_case.artifact_paths.is_empty()
        && !insufficient_case.artifact_paths.is_empty();
    let ready_case_ready = ready_case.prerequisites_passed
        && ready_raw_metrics.method == readiness_row.tool_id
        && float_matches(ready_raw_metrics.estimate, 0.02)
        && float_matches(ready_raw_metrics.ci_low, 0.01)
        && float_matches(ready_raw_metrics.ci_high, 0.03);
    let parser_contract_ready = ready_case
        .artifact_paths
        .iter()
        .any(|path| path == &ready_case.contamination_report)
        && ready_case.artifact_paths.iter().any(|path| path == &ready_case.contamination_summary);
    let ready_summary_ready = ready_summary.schema_version == EXPECTED_SUMMARY_SCHEMA_VERSION
        && ready_summary.stage_id == EXPECTED_STAGE_ID
        && ready_summary.tool == readiness_row.tool_id
        && ready_summary.prerequisites_passed
        && ready_summary.estimate.is_some_and(|value| float_matches(value, 0.02))
        && ready_summary.ci_low.is_some_and(|value| float_matches(value, 0.01))
        && ready_summary.ci_high.is_some_and(|value| float_matches(value, 0.03));
    let ready_stage_metrics_ready =
        required_stage_metrics_ready(&ready_stage_metrics, "ready", true, true);
    let insufficient_summary_ready = insufficient_summary.schema_version
        == EXPECTED_SUMMARY_SCHEMA_VERSION
        && insufficient_summary.stage_id == EXPECTED_STAGE_ID
        && insufficient_summary.tool == readiness_row.tool_id
        && !insufficient_summary.prerequisites_passed
        && insufficient_summary.estimate.is_none()
        && insufficient_summary.ci_low.is_none()
        && insufficient_summary.ci_high.is_none()
        && !insufficient_summary.refusal_codes.is_empty();
    let insufficient_stage_metrics_ready =
        required_stage_metrics_ready(&insufficient_stage_metrics, "insufficient", false, false);
    let insufficiency_behavior_ready = insufficient_summary_ready
        && insufficient_stage_metrics_ready
        && !insufficient_case.refusal_codes.is_empty()
        && !insufficient_case.prerequisites_passed;

    let mut missing_surfaces = Vec::new();
    if !readiness_row.active_scope_ready {
        missing_surfaces.push("active_scope".to_string());
    }
    if !readiness_row.command_ready {
        missing_surfaces.push("command".to_string());
    }
    if !readiness_row.output_ready {
        missing_surfaces.push("output".to_string());
    }
    if !readiness_row.parser_ready {
        missing_surfaces.push("parser".to_string());
    }
    if !readiness_row.expected_result_ready {
        missing_surfaces.push("expected_result".to_string());
    }
    if !readiness_row.report_ready {
        missing_surfaces.push("report".to_string());
    }
    if !readiness_row.schema_ready {
        missing_surfaces.push("schema".to_string());
    }
    if !local_smoke_ready {
        missing_surfaces.push("local_smoke".to_string());
    }
    if !ready_case_ready {
        missing_surfaces.push("ready_case".to_string());
    }
    if !parser_contract_ready {
        missing_surfaces.push("parser_contract".to_string());
    }
    if !ready_summary_ready {
        missing_surfaces.push("ready_summary".to_string());
    }
    if !ready_stage_metrics_ready {
        missing_surfaces.push("ready_stage_metrics".to_string());
    }
    if !insufficient_summary_ready {
        missing_surfaces.push("insufficient_summary".to_string());
    }
    if !insufficient_stage_metrics_ready {
        missing_surfaces.push("insufficient_stage_metrics".to_string());
    }
    if !insufficiency_behavior_ready {
        missing_surfaces.push("insufficiency_behavior".to_string());
    }

    let coverage_status =
        if missing_surfaces.is_empty() { "complete".to_string() } else { "incomplete".to_string() };
    let reason = if missing_surfaces.is_empty() {
        format!(
            "binding `{}` / `{}` keeps retained readiness plus ready and insufficient contamination smoke proofs",
            readiness_row.stage_id, readiness_row.tool_id
        )
    } else {
        format!(
            "binding `{}` / `{}` is missing completion proof for {}",
            readiness_row.stage_id,
            readiness_row.tool_id,
            missing_surfaces.join(", ")
        )
    };

    Ok(BamContaminationCompleteRow {
        result_id: readiness_row.result_id.clone(),
        stage_id: readiness_row.stage_id.clone(),
        tool_id: readiness_row.tool_id.clone(),
        sample_scope: readiness_row.sample_scope.clone(),
        benchmark_status: readiness_row.benchmark_status.clone(),
        support_status: readiness_row.support_status.clone(),
        adapter_status: readiness_row.adapter_status.clone(),
        parser_status: readiness_row.parser_status.clone(),
        corpus_status: readiness_row.corpus_status.clone(),
        report_section_id: readiness_row.report_section_id.clone(),
        summary_table_id: readiness_row.summary_table_id.clone(),
        command_readiness_kind: readiness_row.command_readiness_kind.clone(),
        required_output_ids: REQUIRED_OUTPUT_IDS.iter().map(|value| (*value).to_string()).collect(),
        stage_output_ids: readiness_row.stage_output_ids.clone(),
        expected_schema_extension_id: readiness_row.expected_schema_extension_id.clone(),
        schema_extension_id: readiness_row.schema_extension_id.clone(),
        active_scope_proof_path: readiness_row.active_scope_proof_path.clone(),
        command_proof_path: readiness_row.command_proof_path.clone(),
        output_contract_proof_path: readiness_row.output_contract_proof_path.clone(),
        parser_proof_path: readiness_row.parser_proof_path.clone(),
        expected_result_proof_path: readiness_row.expected_result_proof_path.clone(),
        report_map_proof_path: readiness_row.report_map_proof_path.clone(),
        schema_proof_path: readiness_row.schema_proof_path.clone(),
        local_smoke_proof_path: path_relative_to_repo(repo_root, smoke_path),
        ready_case_report_path: ready_case.contamination_report.clone(),
        ready_case_summary_path: ready_case.contamination_summary.clone(),
        ready_case_stage_metrics_path: ready_case.stage_metrics.clone(),
        insufficient_case_summary_path: insufficient_case.contamination_summary.clone(),
        insufficient_case_stage_metrics_path: insufficient_case.stage_metrics.clone(),
        ready_case_raw_method: ready_raw_metrics.method,
        ready_case_raw_estimate: ready_raw_metrics.estimate,
        ready_case_raw_ci_low: ready_raw_metrics.ci_low,
        ready_case_raw_ci_high: ready_raw_metrics.ci_high,
        ready_case_summary_schema_version: ready_summary.schema_version.clone(),
        insufficient_case_summary_schema_version: insufficient_summary.schema_version.clone(),
        ready_case_prerequisites_passed: ready_summary.prerequisites_passed,
        insufficient_case_prerequisites_passed: insufficient_summary.prerequisites_passed,
        insufficient_case_refusal_codes: insufficient_summary.refusal_codes.clone(),
        ready_case_stage_metrics_schema_version: required_string_field(
            &ready_stage_metrics,
            "schema_version",
            &ready_stage_metrics_path,
        )?,
        insufficient_case_stage_metrics_schema_version: required_string_field(
            &insufficient_stage_metrics,
            "schema_version",
            &insufficient_stage_metrics_path,
        )?,
        local_smoke_sample_id: ready_case.sample_id.clone(),
        local_smoke_ready_case_scope: ready_case.scope.clone(),
        local_smoke_ready_case_tool_scope: ready_case.tool_scope.clone(),
        local_smoke_input_bam: ready_case.input_bam.clone(),
        local_smoke_reference_fasta: ready_case.reference_fasta.clone(),
        local_smoke_minimum_mean_coverage: ready_case.minimum_mean_coverage,
        tool_specific_artifact_path: tool_specific_output_path(ready_case),
        active_scope_ready: readiness_row.active_scope_ready,
        command_ready: readiness_row.command_ready,
        output_ready: readiness_row.output_ready,
        parser_ready: readiness_row.parser_ready,
        expected_result_ready: readiness_row.expected_result_ready,
        report_ready: readiness_row.report_ready,
        schema_ready: readiness_row.schema_ready,
        local_smoke_ready,
        ready_case_ready,
        parser_contract_ready,
        ready_summary_ready,
        ready_stage_metrics_ready,
        insufficient_summary_ready,
        insufficient_stage_metrics_ready,
        insufficiency_behavior_ready,
        coverage_status,
        missing_surfaces,
        reason,
    })
}

fn required_outputs_present(case: &LocalContaminationSmokeCaseReport) -> bool {
    REQUIRED_OUTPUT_IDS
        .iter()
        .all(|output_id| case.declared_output_ids.iter().any(|candidate| candidate == output_id))
}

fn tool_specific_output_path(case: &LocalContaminationSmokeCaseReport) -> Option<String> {
    match case.tool_id.as_str() {
        "contammix" => case.contammix_report.clone(),
        "schmutzi" => case.mt_consensus.clone(),
        "verifybamid2" => case.contamination_estimate.clone(),
        _ => None,
    }
}

fn required_stage_metrics_ready(
    payload: &serde_json::Value,
    proof_case: &str,
    expected_prerequisites_passed: bool,
    actual_prerequisites_passed: bool,
) -> bool {
    payload.get("schema_version").and_then(serde_json::Value::as_str)
        == Some(EXPECTED_STAGE_METRICS_SCHEMA_VERSION)
        && payload.get("proof_case").and_then(serde_json::Value::as_str) == Some(proof_case)
        && payload.get("expected_prerequisites_passed").and_then(serde_json::Value::as_bool)
            == Some(expected_prerequisites_passed)
        && payload.get("prerequisites_passed").and_then(serde_json::Value::as_bool)
            == Some(actual_prerequisites_passed)
        && payload.get("expectation_matched").and_then(serde_json::Value::as_bool) == Some(true)
}

fn required_string_field(payload: &serde_json::Value, key: &str, path: &Path) -> Result<String> {
    payload
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("{} is missing string `{key}`", path.display()))
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

fn float_matches(left: f64, right: f64) -> bool {
    (left - right).abs() <= 1e-9
}
