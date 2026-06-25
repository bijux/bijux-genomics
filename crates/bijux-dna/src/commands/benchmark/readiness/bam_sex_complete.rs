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

pub(crate) const DEFAULT_BAM_SEX_COMPLETE_PATH: &str =
    "benchmarks/readiness/bam/stages/bam.sex.complete.json";
const BAM_SEX_COMPLETE_SCHEMA_VERSION: &str = "bijux.bench.readiness.bam_sex_complete.v1";
const EXPECTED_STAGE_ID: &str = "bam.sex";
const EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION: &str = "bijux.bam.sex.tool_smoke.report.v1";
const EXPECTED_STAGE_METRICS_SCHEMA_VERSION: &str = "bijux.bam.sex.stage_metrics.v1";
const EXPECTED_REPORT_SCHEMA_VERSION: &str = "bijux.bam.sex.v1";
const EXPECTED_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.sex_summary.v1";
const EXPECTED_INSUFFICIENCY_REASON: &str = "insufficient_chromosomes";
const CHECKED_SURFACE_COUNT: usize = 15;
const EXPECTED_TOOL_IDS: [&str; 3] = ["angsd", "rxy", "yleaf"];
const REQUIRED_OUTPUT_IDS: [&str; 3] = ["sex_report", "summary", "stage_metrics"];

#[derive(Debug, Clone, Deserialize)]
struct LocalSexToolSmokeReport {
    schema_version: String,
    stage_id: String,
    ready_sample_id: String,
    insufficient_sample_id: String,
    tool_ids: Vec<String>,
    case_count: usize,
    rows: Vec<LocalSexToolSmokeCaseReport>,
}

#[derive(Debug, Clone, Deserialize)]
struct LocalSexToolSmokeCaseReport {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    proof_case: String,
    sample_id: String,
    expectation_matched: bool,
    input_bam: String,
    reference_fasta: String,
    method: String,
    chromosome_system: Option<String>,
    minimum_y_sites: Option<u32>,
    x_coverage: f64,
    y_coverage: f64,
    autosomal_coverage: f64,
    x_to_y_ratio: Option<f64>,
    call: bijux_dna_domain_bam::metrics::SexConfidenceClass,
    confidence: f64,
    status: String,
    insufficiency_reason: Option<String>,
    sex_report: String,
    sex_estimate: String,
    population_metrics: String,
    haplogroup_report: String,
    sex_summary: String,
    stage_metrics: String,
    declared_output_ids: Vec<String>,
    artifact_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamSexCompleteRow {
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
    pub(crate) ready_case_tool_specific_artifact_path: String,
    pub(crate) insufficient_case_report_path: String,
    pub(crate) insufficient_case_summary_path: String,
    pub(crate) insufficient_case_stage_metrics_path: String,
    pub(crate) insufficient_case_tool_specific_artifact_path: String,
    pub(crate) ready_case_summary_schema_version: String,
    pub(crate) insufficient_case_summary_schema_version: String,
    pub(crate) ready_case_stage_metrics_schema_version: String,
    pub(crate) insufficient_case_stage_metrics_schema_version: String,
    pub(crate) local_smoke_ready_sample_id: String,
    pub(crate) local_smoke_insufficient_sample_id: String,
    pub(crate) local_smoke_input_bam: String,
    pub(crate) local_smoke_reference_fasta: String,
    pub(crate) local_smoke_chromosome_system: Option<String>,
    pub(crate) local_smoke_minimum_y_sites: Option<u32>,
    pub(crate) ready_case_x_coverage: f64,
    pub(crate) ready_case_y_coverage: f64,
    pub(crate) ready_case_autosomal_coverage: f64,
    pub(crate) ready_case_x_to_y_ratio: Option<f64>,
    pub(crate) ready_case_call: String,
    pub(crate) ready_case_confidence: f64,
    pub(crate) ready_case_status: String,
    pub(crate) insufficient_case_x_coverage: f64,
    pub(crate) insufficient_case_y_coverage: f64,
    pub(crate) insufficient_case_autosomal_coverage: f64,
    pub(crate) insufficient_case_call: String,
    pub(crate) insufficient_case_confidence: f64,
    pub(crate) insufficient_case_status: String,
    pub(crate) insufficient_case_insufficiency_reason: Option<String>,
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
    pub(crate) tool_specific_artifact_ready: bool,
    pub(crate) coverage_status: String,
    pub(crate) missing_surfaces: Vec<String>,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamSexCompleteReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) active_row_count: usize,
    pub(crate) complete_row_count: usize,
    pub(crate) incomplete_row_count: usize,
    pub(crate) checked_surface_count: usize,
    pub(crate) expected_tool_ids: Vec<String>,
    pub(crate) required_output_ids: Vec<String>,
    pub(crate) local_smoke_ready_sample_id: String,
    pub(crate) local_smoke_insufficient_sample_id: String,
    pub(crate) local_smoke_case_count: usize,
    pub(crate) toolset_ready: bool,
    pub(crate) violation_count: usize,
    pub(crate) ok: bool,
    pub(crate) rows: Vec<BamSexCompleteRow>,
    pub(crate) violations: Vec<BamSexCompleteRow>,
}

pub(crate) fn run_render_bam_sex_complete(
    args: &parse::BenchReadinessRenderBamSexCompleteArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_sex_complete(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_SEX_COMPLETE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_bam_sex_complete(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamSexCompleteReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let report = build_bam_sex_complete_report(repo_root, &output_path)?;
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    if !report.ok {
        return Err(anyhow!(
            "bam sex completion must keep active scope, command, output, parser, expected-result, report, schema, ready smoke, insufficient smoke, and tool-specific artifact proof for all retained tools"
        ));
    }
    Ok(report)
}

fn build_bam_sex_complete_report(
    repo_root: &Path,
    output_path: &Path,
) -> Result<BamSexCompleteReport> {
    let readiness = render_bam_contamination_sex_haplogroups_ready(
        repo_root,
        PathBuf::from(DEFAULT_BAM_CONTAMINATION_SEX_HAPLOGROUPS_READY_PATH),
    )?;
    let smoke_path = bijux_dna_api::v1::api::bam::write_local_sex_tool_smoke_report()?;
    let smoke_report: LocalSexToolSmokeReport = serde_json::from_str(
        &fs::read_to_string(&smoke_path)
            .with_context(|| format!("read {}", smoke_path.display()))?,
    )
    .with_context(|| format!("parse {}", smoke_path.display()))?;

    if smoke_report.schema_version != EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION {
        return Err(anyhow!(
            "unexpected bam.sex local-smoke schema `{}`",
            smoke_report.schema_version
        ));
    }
    if smoke_report.stage_id != EXPECTED_STAGE_ID {
        return Err(anyhow!("unexpected bam.sex local-smoke stage `{}`", smoke_report.stage_id));
    }
    if smoke_report.case_count != EXPECTED_TOOL_IDS.len() * 2 {
        return Err(anyhow!(
            "bam.sex local-smoke case count drifted: observed={} expected={}",
            smoke_report.case_count,
            EXPECTED_TOOL_IDS.len() * 2
        ));
    }

    let mut smoke_tool_ids = smoke_report.tool_ids.clone();
    smoke_tool_ids.sort();
    let expected_tool_ids =
        EXPECTED_TOOL_IDS.iter().map(|value| (*value).to_string()).collect::<Vec<_>>();
    if smoke_tool_ids != expected_tool_ids {
        return Err(anyhow!(
            "bam.sex local-smoke tool ids drifted: observed={:?} expected={:?}",
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
            "bam.sex readiness rows drifted: observed={:?} expected={:?}",
            observed_tool_ids,
            expected_tool_ids
        ));
    }

    let mut report_rows = Vec::with_capacity(rows.len());
    for readiness_row in rows {
        let ready_case = ready_cases.get(&readiness_row.tool_id).ok_or_else(|| {
            anyhow!("missing bam.sex ready smoke case for `{}`", readiness_row.tool_id)
        })?;
        let insufficient_case =
            insufficient_cases.get(&readiness_row.tool_id).ok_or_else(|| {
                anyhow!("missing bam.sex insufficient smoke case for `{}`", readiness_row.tool_id)
            })?;
        report_rows.push(build_bam_sex_complete_row(
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

    Ok(BamSexCompleteReport {
        schema_version: BAM_SEX_COMPLETE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        active_row_count: report_rows.len(),
        complete_row_count,
        incomplete_row_count,
        checked_surface_count: CHECKED_SURFACE_COUNT,
        expected_tool_ids,
        required_output_ids: REQUIRED_OUTPUT_IDS.iter().map(|value| (*value).to_string()).collect(),
        local_smoke_ready_sample_id: smoke_report.ready_sample_id,
        local_smoke_insufficient_sample_id: smoke_report.insufficient_sample_id,
        local_smoke_case_count: smoke_report.case_count,
        toolset_ready: violations.is_empty(),
        violation_count: violations.len(),
        ok: violations.is_empty(),
        rows: report_rows,
        violations,
    })
}

fn build_bam_sex_complete_row(
    repo_root: &Path,
    smoke_path: &Path,
    smoke_report: &LocalSexToolSmokeReport,
    readiness_row: &BamContaminationSexHaplogroupsReadyRow,
    ready_case: &LocalSexToolSmokeCaseReport,
    insufficient_case: &LocalSexToolSmokeCaseReport,
) -> Result<BamSexCompleteRow> {
    let ready_report_path = repo_root.join(&ready_case.sex_report);
    let ready_summary_path = repo_root.join(&ready_case.sex_summary);
    let ready_stage_metrics_path = repo_root.join(&ready_case.stage_metrics);
    let insufficient_report_path = repo_root.join(&insufficient_case.sex_report);
    let insufficient_summary_path = repo_root.join(&insufficient_case.sex_summary);
    let insufficient_stage_metrics_path = repo_root.join(&insufficient_case.stage_metrics);
    let ready_tool_specific_artifact_path = repo_root.join(tool_specific_output_path(ready_case));
    let insufficient_tool_specific_artifact_path =
        repo_root.join(tool_specific_output_path(insufficient_case));

    let ready_report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&ready_report_path)
            .with_context(|| format!("read {}", ready_report_path.display()))?,
    )
    .with_context(|| format!("parse {}", ready_report_path.display()))?;
    let ready_summary: bijux_dna_domain_bam::BamSexSummaryV1 = serde_json::from_str(
        &fs::read_to_string(&ready_summary_path)
            .with_context(|| format!("read {}", ready_summary_path.display()))?,
    )
    .with_context(|| format!("parse {}", ready_summary_path.display()))?;
    let ready_stage_metrics: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&ready_stage_metrics_path)
            .with_context(|| format!("read {}", ready_stage_metrics_path.display()))?,
    )
    .with_context(|| format!("parse {}", ready_stage_metrics_path.display()))?;
    let insufficient_report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&insufficient_report_path)
            .with_context(|| format!("read {}", insufficient_report_path.display()))?,
    )
    .with_context(|| format!("parse {}", insufficient_report_path.display()))?;
    let insufficient_summary: bijux_dna_domain_bam::BamSexSummaryV1 = serde_json::from_str(
        &fs::read_to_string(&insufficient_summary_path)
            .with_context(|| format!("read {}", insufficient_summary_path.display()))?,
    )
    .with_context(|| format!("parse {}", insufficient_summary_path.display()))?;
    let insufficient_stage_metrics: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&insufficient_stage_metrics_path)
            .with_context(|| format!("read {}", insufficient_stage_metrics_path.display()))?,
    )
    .with_context(|| format!("parse {}", insufficient_stage_metrics_path.display()))?;

    let local_smoke_ready = ready_case.schema_version == EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION
        && insufficient_case.schema_version == EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION
        && ready_case.stage_id == EXPECTED_STAGE_ID
        && insufficient_case.stage_id == EXPECTED_STAGE_ID
        && ready_case.tool_id == readiness_row.tool_id
        && insufficient_case.tool_id == readiness_row.tool_id
        && ready_case.expectation_matched
        && insufficient_case.expectation_matched
        && ready_case.sample_id == smoke_report.ready_sample_id
        && insufficient_case.sample_id == smoke_report.insufficient_sample_id
        && ready_case.method == readiness_row.tool_id
        && insufficient_case.method == readiness_row.tool_id
        && required_outputs_present(ready_case)
        && required_outputs_present(insufficient_case)
        && case_artifacts_present(ready_case)
        && case_artifacts_present(insufficient_case);
    let ready_case_ready = ready_report_contract_ready(&ready_report, &readiness_row.tool_id)
        && ready_summary.schema_version == EXPECTED_SUMMARY_SCHEMA_VERSION
        && ready_summary.stage_id == EXPECTED_STAGE_ID
        && ready_summary.method == readiness_row.tool_id
        && ready_summary.chromosome_system == ready_case.chromosome_system
        && ready_summary.minimum_y_sites == ready_case.minimum_y_sites
        && float_matches(ready_summary.x_coverage, ready_case.x_coverage)
        && float_matches(ready_summary.y_coverage, ready_case.y_coverage)
        && float_matches(ready_summary.autosomal_coverage, ready_case.autosomal_coverage)
        && ready_summary.x_to_y_ratio == ready_case.x_to_y_ratio
        && ready_summary.call == ready_case.call
        && float_matches(ready_summary.confidence, ready_case.confidence)
        && ready_summary.status == ready_case.status
        && ready_summary.insufficiency_reason.is_none()
        && ready_summary.x_coverage > 0.0
        && ready_summary.y_coverage > 0.0
        && ready_summary.autosomal_coverage > 0.0
        && ready_summary.x_covered_sites > 0
        && ready_summary.y_covered_sites >= u64::from(ready_summary.minimum_y_sites.unwrap_or(0))
        && ready_summary.x_to_y_ratio.is_some();
    let parser_contract_ready =
        ready_case.artifact_paths.iter().any(|path| path == &ready_case.sex_report)
            && ready_case.artifact_paths.iter().any(|path| path == &ready_case.sex_summary)
            && ready_case.artifact_paths.iter().any(|path| path == &ready_case.stage_metrics)
            && insufficient_case
                .artifact_paths
                .iter()
                .any(|path| path == &insufficient_case.sex_report)
            && insufficient_case
                .artifact_paths
                .iter()
                .any(|path| path == &insufficient_case.sex_summary)
            && insufficient_case
                .artifact_paths
                .iter()
                .any(|path| path == &insufficient_case.stage_metrics);
    let ready_summary_ready = ready_case_ready;
    let ready_stage_metrics_ready = stage_metrics_ready(
        &ready_stage_metrics,
        "ready",
        None,
        sex_call_name(ready_case.call),
        ready_case.confidence,
        &ready_case.status,
    );
    let insufficient_summary_ready = insufficient_report_contract_ready(
        &insufficient_report,
        &readiness_row.tool_id,
        EXPECTED_INSUFFICIENCY_REASON,
    ) && insufficient_summary.schema_version
        == EXPECTED_SUMMARY_SCHEMA_VERSION
        && insufficient_summary.stage_id == EXPECTED_STAGE_ID
        && insufficient_summary.method == readiness_row.tool_id
        && insufficient_summary.call
            == bijux_dna_domain_bam::metrics::SexConfidenceClass::Insufficient
        && float_matches(insufficient_summary.confidence, 0.0)
        && insufficient_summary.status == EXPECTED_INSUFFICIENCY_REASON
        && insufficient_summary.insufficiency_reason.as_deref()
            == Some(EXPECTED_INSUFFICIENCY_REASON)
        && float_matches(insufficient_summary.x_coverage, 0.0)
        && float_matches(insufficient_summary.autosomal_coverage, 0.0)
        && float_matches(insufficient_summary.y_coverage, insufficient_case.y_coverage)
        && insufficient_summary.x_covered_sites == 0
        && insufficient_summary.y_covered_sites > 0;
    let insufficient_stage_metrics_ready = stage_metrics_ready(
        &insufficient_stage_metrics,
        "insufficient",
        Some(EXPECTED_INSUFFICIENCY_REASON),
        "insufficient",
        0.0,
        EXPECTED_INSUFFICIENCY_REASON,
    );
    let insufficiency_behavior_ready = insufficient_summary_ready
        && insufficient_stage_metrics_ready
        && insufficient_case.insufficiency_reason.as_deref() == Some(EXPECTED_INSUFFICIENCY_REASON)
        && insufficient_case.call
            == bijux_dna_domain_bam::metrics::SexConfidenceClass::Insufficient
        && float_matches(insufficient_case.confidence, 0.0)
        && insufficient_case.status == EXPECTED_INSUFFICIENCY_REASON;
    let tool_specific_artifact_ready = tool_specific_artifact_ready(
        &readiness_row.tool_id,
        &ready_tool_specific_artifact_path,
        &insufficient_tool_specific_artifact_path,
        ready_case,
        insufficient_case,
    )?;

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
    if !tool_specific_artifact_ready {
        missing_surfaces.push("tool_specific_artifact".to_string());
    }

    let coverage_status =
        if missing_surfaces.is_empty() { "complete".to_string() } else { "incomplete".to_string() };
    let reason = if missing_surfaces.is_empty() {
        format!(
            "binding `{}` / `{}` keeps retained readiness plus ready and insufficient sex smoke proofs",
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

    Ok(BamSexCompleteRow {
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
        ready_case_report_path: ready_case.sex_report.clone(),
        ready_case_summary_path: ready_case.sex_summary.clone(),
        ready_case_stage_metrics_path: ready_case.stage_metrics.clone(),
        ready_case_tool_specific_artifact_path: path_relative_to_repo(
            repo_root,
            &ready_tool_specific_artifact_path,
        ),
        insufficient_case_report_path: insufficient_case.sex_report.clone(),
        insufficient_case_summary_path: insufficient_case.sex_summary.clone(),
        insufficient_case_stage_metrics_path: insufficient_case.stage_metrics.clone(),
        insufficient_case_tool_specific_artifact_path: path_relative_to_repo(
            repo_root,
            &insufficient_tool_specific_artifact_path,
        ),
        ready_case_summary_schema_version: ready_summary.schema_version.clone(),
        insufficient_case_summary_schema_version: insufficient_summary.schema_version.clone(),
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
        local_smoke_ready_sample_id: smoke_report.ready_sample_id.clone(),
        local_smoke_insufficient_sample_id: smoke_report.insufficient_sample_id.clone(),
        local_smoke_input_bam: ready_case.input_bam.clone(),
        local_smoke_reference_fasta: ready_case.reference_fasta.clone(),
        local_smoke_chromosome_system: ready_case.chromosome_system.clone(),
        local_smoke_minimum_y_sites: ready_case.minimum_y_sites,
        ready_case_x_coverage: ready_summary.x_coverage,
        ready_case_y_coverage: ready_summary.y_coverage,
        ready_case_autosomal_coverage: ready_summary.autosomal_coverage,
        ready_case_x_to_y_ratio: ready_summary.x_to_y_ratio,
        ready_case_call: sex_call_name(ready_summary.call).to_string(),
        ready_case_confidence: ready_summary.confidence,
        ready_case_status: ready_summary.status.clone(),
        insufficient_case_x_coverage: insufficient_summary.x_coverage,
        insufficient_case_y_coverage: insufficient_summary.y_coverage,
        insufficient_case_autosomal_coverage: insufficient_summary.autosomal_coverage,
        insufficient_case_call: sex_call_name(insufficient_summary.call).to_string(),
        insufficient_case_confidence: insufficient_summary.confidence,
        insufficient_case_status: insufficient_summary.status.clone(),
        insufficient_case_insufficiency_reason: insufficient_summary.insufficiency_reason.clone(),
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
        tool_specific_artifact_ready,
        coverage_status,
        missing_surfaces,
        reason,
    })
}

fn required_outputs_present(case: &LocalSexToolSmokeCaseReport) -> bool {
    REQUIRED_OUTPUT_IDS
        .iter()
        .all(|output_id| case.declared_output_ids.iter().any(|candidate| candidate == output_id))
}

fn case_artifacts_present(case: &LocalSexToolSmokeCaseReport) -> bool {
    let tool_specific_output_path = tool_specific_output_path(case);
    case.artifact_paths.iter().any(|path| path == &case.sex_report)
        && case.artifact_paths.iter().any(|path| path == &case.sex_summary)
        && case.artifact_paths.iter().any(|path| path == &case.stage_metrics)
        && case.artifact_paths.iter().any(|path| path == tool_specific_output_path)
}

fn tool_specific_output_path(case: &LocalSexToolSmokeCaseReport) -> &str {
    match case.tool_id.as_str() {
        "angsd" => &case.population_metrics,
        "rxy" => &case.sex_estimate,
        "yleaf" => &case.haplogroup_report,
        _ => &case.sex_estimate,
    }
}

fn ready_report_contract_ready(payload: &serde_json::Value, tool_id: &str) -> bool {
    payload.get("schema_version").and_then(serde_json::Value::as_str)
        == Some(EXPECTED_REPORT_SCHEMA_VERSION)
        && payload.get("method").and_then(serde_json::Value::as_str) == Some(tool_id)
        && payload.get("call").and_then(serde_json::Value::as_str).is_some()
        && payload.get("classification").and_then(serde_json::Value::as_str).is_some()
        && payload.get("confidence").and_then(serde_json::Value::as_f64).is_some()
        && payload.get("status").and_then(serde_json::Value::as_str) == Some("ok")
        && payload.get("insufficiency_reason").is_some_and(serde_json::Value::is_null)
}

fn insufficient_report_contract_ready(
    payload: &serde_json::Value,
    tool_id: &str,
    expected_reason: &str,
) -> bool {
    payload.get("schema_version").and_then(serde_json::Value::as_str)
        == Some(EXPECTED_REPORT_SCHEMA_VERSION)
        && payload.get("method").and_then(serde_json::Value::as_str) == Some(tool_id)
        && payload.get("call").and_then(serde_json::Value::as_str) == Some("insufficient")
        && payload.get("classification").and_then(serde_json::Value::as_str) == Some("insufficient")
        && payload.get("confidence").and_then(serde_json::Value::as_f64) == Some(0.0)
        && payload.get("status").and_then(serde_json::Value::as_str) == Some(expected_reason)
        && payload.get("insufficiency_reason").and_then(serde_json::Value::as_str)
            == Some(expected_reason)
}

fn stage_metrics_ready(
    payload: &serde_json::Value,
    proof_case: &str,
    expected_insufficiency_reason: Option<&str>,
    expected_call: &str,
    expected_confidence: f64,
    expected_status: &str,
) -> bool {
    payload.get("schema_version").and_then(serde_json::Value::as_str)
        == Some(EXPECTED_STAGE_METRICS_SCHEMA_VERSION)
        && payload.get("stage_id").and_then(serde_json::Value::as_str) == Some(EXPECTED_STAGE_ID)
        && payload.get("proof_case").and_then(serde_json::Value::as_str) == Some(proof_case)
        && payload.get("expected_call").and_then(serde_json::Value::as_str) == Some(expected_call)
        && payload.get("call").and_then(serde_json::Value::as_str) == Some(expected_call)
        && payload.get("expected_confidence").and_then(serde_json::Value::as_f64)
            == Some(expected_confidence)
        && payload.get("confidence").and_then(serde_json::Value::as_f64)
            == Some(expected_confidence)
        && payload.get("expected_status").and_then(serde_json::Value::as_str)
            == Some(expected_status)
        && payload.get("status").and_then(serde_json::Value::as_str) == Some(expected_status)
        && optional_string_field(payload, "expected_insufficiency_reason")
            == expected_insufficiency_reason
        && optional_string_field(payload, "insufficiency_reason") == expected_insufficiency_reason
        && payload.get("expectation_matched").and_then(serde_json::Value::as_bool) == Some(true)
}

fn optional_string_field<'a>(payload: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    payload.get(key).and_then(serde_json::Value::as_str)
}

fn sex_call_name(call: bijux_dna_domain_bam::metrics::SexConfidenceClass) -> &'static str {
    match call {
        bijux_dna_domain_bam::metrics::SexConfidenceClass::Male => "male",
        bijux_dna_domain_bam::metrics::SexConfidenceClass::Female => "female",
        bijux_dna_domain_bam::metrics::SexConfidenceClass::Ambiguous => "ambiguous",
        bijux_dna_domain_bam::metrics::SexConfidenceClass::Insufficient => "insufficient",
    }
}

fn tool_specific_artifact_ready(
    tool_id: &str,
    ready_path: &Path,
    insufficient_path: &Path,
    ready_case: &LocalSexToolSmokeCaseReport,
    insufficient_case: &LocalSexToolSmokeCaseReport,
) -> Result<bool> {
    let ready_payload: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(ready_path)
            .with_context(|| format!("read {}", ready_path.display()))?,
    )
    .with_context(|| format!("parse {}", ready_path.display()))?;
    let insufficient_payload: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(insufficient_path)
            .with_context(|| format!("read {}", insufficient_path.display()))?,
    )
    .with_context(|| format!("parse {}", insufficient_path.display()))?;

    let ready_common = ready_payload.get("stage_id").and_then(serde_json::Value::as_str)
        == Some(EXPECTED_STAGE_ID)
        && ready_payload.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
        && ready_payload.get("proof_case").and_then(serde_json::Value::as_str) == Some("ready");
    let insufficient_common = insufficient_payload
        .get("stage_id")
        .and_then(serde_json::Value::as_str)
        == Some(EXPECTED_STAGE_ID)
        && insufficient_payload.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
        && insufficient_payload.get("proof_case").and_then(serde_json::Value::as_str)
            == Some("insufficient");

    let ready_ok = match tool_id {
        "angsd" => {
            ready_common
                && ready_payload.get("artifact_id").and_then(serde_json::Value::as_str)
                    == Some("population_metrics")
                && ready_payload.get("chromosome_system").and_then(serde_json::Value::as_str)
                    == ready_case.chromosome_system.as_deref()
                && ready_payload.get("x_coverage").and_then(serde_json::Value::as_f64)
                    == Some(ready_case.x_coverage)
                && ready_payload.get("y_coverage").and_then(serde_json::Value::as_f64)
                    == Some(ready_case.y_coverage)
                && ready_payload.get("autosomal_coverage").and_then(serde_json::Value::as_f64)
                    == Some(ready_case.autosomal_coverage)
        }
        "rxy" => {
            ready_common
                && ready_payload.get("artifact_id").and_then(serde_json::Value::as_str)
                    == Some("sex_estimate")
                && ready_payload.get("call").and_then(serde_json::Value::as_str)
                    == Some(sex_call_name(ready_case.call))
                && ready_payload.get("confidence").and_then(serde_json::Value::as_f64)
                    == Some(ready_case.confidence)
                && ready_payload.get("status").and_then(serde_json::Value::as_str)
                    == Some(ready_case.status.as_str())
        }
        "yleaf" => {
            ready_common
                && ready_payload.get("artifact_id").and_then(serde_json::Value::as_str)
                    == Some("haplogroup_report")
                && ready_payload.get("status").and_then(serde_json::Value::as_str)
                    == Some("not_applicable_for_sex_inference")
        }
        _ => false,
    };

    let insufficient_ok = match tool_id {
        "angsd" => {
            insufficient_common
                && insufficient_payload.get("artifact_id").and_then(serde_json::Value::as_str)
                    == Some("population_metrics")
                && insufficient_payload.get("x_coverage").and_then(serde_json::Value::as_f64)
                    == Some(insufficient_case.x_coverage)
                && insufficient_payload
                    .get("autosomal_coverage")
                    .and_then(serde_json::Value::as_f64)
                    == Some(insufficient_case.autosomal_coverage)
        }
        "rxy" => {
            insufficient_common
                && insufficient_payload.get("artifact_id").and_then(serde_json::Value::as_str)
                    == Some("sex_estimate")
                && insufficient_payload.get("call").and_then(serde_json::Value::as_str)
                    == Some("insufficient")
                && insufficient_payload
                    .get("insufficiency_reason")
                    .and_then(serde_json::Value::as_str)
                    == Some(EXPECTED_INSUFFICIENCY_REASON)
        }
        "yleaf" => {
            insufficient_common
                && insufficient_payload.get("artifact_id").and_then(serde_json::Value::as_str)
                    == Some("haplogroup_report")
                && insufficient_payload.get("status").and_then(serde_json::Value::as_str)
                    == Some("not_applicable_due_to_insufficient_chromosomes")
        }
        _ => false,
    };

    Ok(ready_ok && insufficient_ok)
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
