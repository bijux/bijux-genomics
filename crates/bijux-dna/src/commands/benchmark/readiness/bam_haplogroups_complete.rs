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

pub(crate) const DEFAULT_BAM_HAPLOGROUPS_COMPLETE_PATH: &str =
    "benchmarks/readiness/bam/stages/bam.haplogroups.complete.json";
const BAM_HAPLOGROUPS_COMPLETE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_haplogroups_complete.v1";
const EXPECTED_STAGE_ID: &str = "bam.haplogroups";
const EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION: &str = "bijux.bam.haplogroups.local_smoke.report.v1";
const EXPECTED_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.haplogroup_readiness.v1";
const EXPECTED_STAGE_METRICS_SCHEMA_VERSION: &str = "bijux.bam.haplogroups.stage_metrics.v1";
const EXPECTED_REPORT_SCHEMA_VERSION: &str = "bijux.bam.haplogroups.v1";
const EXPECTED_SAMPLE_ID: &str = "adna_y_haplogroup_panel";
const EXPECTED_REFERENCE_PANEL_ID: &str = "adna-y-hg38-mini";
const EXPECTED_REFERENCE_BUILD: &str = "hg38";
const EXPECTED_POPULATION_SCOPE: &str = "adna_y_haplogroup_panel";
const EXPECTED_READY_STATUS: &str = "ready";
const EXPECTED_INSUFFICIENT_STATUS: &str = "coverage_gate_not_met";
const CHECKED_SURFACE_COUNT: usize = 14;
const EXPECTED_TOOL_IDS: [&str; 1] = ["yleaf"];
const REQUIRED_OUTPUT_IDS: [&str; 3] = ["haplogroups", "summary", "stage_metrics"];

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamHaplogroupsCompleteRow {
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
    pub(crate) local_smoke_sample_id: String,
    pub(crate) local_smoke_reference_panel_path: String,
    pub(crate) local_smoke_reference_panel_id: String,
    pub(crate) local_smoke_reference_build: String,
    pub(crate) local_smoke_population_scope: String,
    pub(crate) local_smoke_input_bam: String,
    pub(crate) local_smoke_reference_fasta: String,
    pub(crate) ready_case_haplogroup_call: Option<String>,
    pub(crate) ready_case_confidence: f64,
    pub(crate) ready_case_status: String,
    pub(crate) ready_case_markers_total: usize,
    pub(crate) ready_case_markers_supported: usize,
    pub(crate) insufficient_case_haplogroup_call: Option<String>,
    pub(crate) insufficient_case_confidence: f64,
    pub(crate) insufficient_case_status: String,
    pub(crate) insufficient_case_markers_total: usize,
    pub(crate) insufficient_case_markers_supported: usize,
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
    pub(crate) coverage_gate_ready: bool,
    pub(crate) tool_specific_artifact_ready: bool,
    pub(crate) coverage_status: String,
    pub(crate) missing_surfaces: Vec<String>,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamHaplogroupsCompleteReport {
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
    pub(crate) rows: Vec<BamHaplogroupsCompleteRow>,
    pub(crate) violations: Vec<BamHaplogroupsCompleteRow>,
}

pub(crate) fn run_render_bam_haplogroups_complete(
    args: &parse::BenchReadinessRenderBamHaplogroupsCompleteArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_haplogroups_complete(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_HAPLOGROUPS_COMPLETE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_bam_haplogroups_complete(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamHaplogroupsCompleteReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let report = build_bam_haplogroups_complete_report(repo_root, &output_path)?;
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    if !report.ok {
        return Err(anyhow!(
            "bam haplogroups completion must keep active scope, command, output, parser, expected-result, report, schema, local smoke, marker support, and coverage-gate proof for all retained tools"
        ));
    }
    Ok(report)
}

fn build_bam_haplogroups_complete_report(
    repo_root: &Path,
    output_path: &Path,
) -> Result<BamHaplogroupsCompleteReport> {
    let readiness = render_bam_contamination_sex_haplogroups_ready(
        repo_root,
        PathBuf::from(DEFAULT_BAM_CONTAMINATION_SEX_HAPLOGROUPS_READY_PATH),
    )?;
    let smoke_path = bijux_dna_api::v1::api::bam::write_local_haplogroups_smoke_report()?;
    let smoke_report: LocalHaplogroupsSmokeReport = serde_json::from_str(
        &fs::read_to_string(&smoke_path)
            .with_context(|| format!("read {}", smoke_path.display()))?,
    )
    .with_context(|| format!("parse {}", smoke_path.display()))?;

    if smoke_report.schema_version != EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION {
        return Err(anyhow!(
            "unexpected bam.haplogroups local-smoke schema `{}`",
            smoke_report.schema_version
        ));
    }
    if smoke_report.stage_id != EXPECTED_STAGE_ID {
        return Err(anyhow!(
            "unexpected bam.haplogroups local-smoke stage `{}`",
            smoke_report.stage_id
        ));
    }
    if smoke_report.tool_id != EXPECTED_TOOL_IDS[0] {
        return Err(anyhow!(
            "unexpected bam.haplogroups local-smoke tool `{}`",
            smoke_report.tool_id
        ));
    }
    if smoke_report.sample_id != EXPECTED_SAMPLE_ID {
        return Err(anyhow!(
            "unexpected bam.haplogroups local-smoke sample `{}`",
            smoke_report.sample_id
        ));
    }
    if smoke_report.reference_panel_id != EXPECTED_REFERENCE_PANEL_ID {
        return Err(anyhow!(
            "unexpected bam.haplogroups local-smoke reference_panel_id `{}`",
            smoke_report.reference_panel_id
        ));
    }
    if smoke_report.reference_build != EXPECTED_REFERENCE_BUILD {
        return Err(anyhow!(
            "unexpected bam.haplogroups local-smoke reference_build `{}`",
            smoke_report.reference_build
        ));
    }

    let ready_case = smoke_report
        .rows
        .iter()
        .find(|row| row.proof_case == "ready")
        .cloned()
        .ok_or_else(|| anyhow!("missing bam.haplogroups ready case"))?;
    let insufficient_case = smoke_report
        .rows
        .iter()
        .find(|row| row.proof_case == "insufficient")
        .cloned()
        .ok_or_else(|| anyhow!("missing bam.haplogroups insufficient case"))?;

    let expected_tool_ids =
        EXPECTED_TOOL_IDS.iter().map(|value| (*value).to_string()).collect::<Vec<_>>();
    let mut rows = readiness
        .rows
        .into_iter()
        .filter(|row| row.stage_id == EXPECTED_STAGE_ID)
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));
    let observed_tool_ids = rows.iter().map(|row| row.tool_id.clone()).collect::<Vec<_>>();
    if observed_tool_ids != expected_tool_ids {
        return Err(anyhow!(
            "bam.haplogroups readiness rows drifted: observed={:?} expected={:?}",
            observed_tool_ids,
            expected_tool_ids
        ));
    }

    let report_rows = rows
        .iter()
        .map(|readiness_row| {
            build_bam_haplogroups_complete_row(
                repo_root,
                &smoke_path,
                &smoke_report,
                readiness_row,
                &ready_case,
                &insufficient_case,
            )
        })
        .collect::<Result<Vec<_>>>()?;

    let complete_row_count =
        report_rows.iter().filter(|row| row.coverage_status == "complete").count();
    let incomplete_row_count = report_rows.len().saturating_sub(complete_row_count);
    let violations = report_rows
        .iter()
        .filter(|row| row.coverage_status != "complete")
        .cloned()
        .collect::<Vec<_>>();

    Ok(BamHaplogroupsCompleteReport {
        schema_version: BAM_HAPLOGROUPS_COMPLETE_SCHEMA_VERSION,
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

fn build_bam_haplogroups_complete_row(
    repo_root: &Path,
    smoke_path: &Path,
    smoke_report: &LocalHaplogroupsSmokeReport,
    readiness_row: &BamContaminationSexHaplogroupsReadyRow,
    ready_case: &LocalHaplogroupsSmokeCaseReport,
    insufficient_case: &LocalHaplogroupsSmokeCaseReport,
) -> Result<BamHaplogroupsCompleteRow> {
    let ready_report_path = repo_root.join(&ready_case.haplogroups_report);
    let ready_summary_path = repo_root.join(&ready_case.haplogroups_summary);
    let ready_stage_metrics_path = repo_root.join(&ready_case.stage_metrics);
    let ready_tool_specific_artifact_path = repo_root.join(&ready_case.haplogroup_report);
    let insufficient_report_path = repo_root.join(&insufficient_case.haplogroups_report);
    let insufficient_summary_path = repo_root.join(&insufficient_case.haplogroups_summary);
    let insufficient_stage_metrics_path = repo_root.join(&insufficient_case.stage_metrics);
    let insufficient_tool_specific_artifact_path =
        repo_root.join(&insufficient_case.haplogroup_report);

    let ready_report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&ready_report_path)
            .with_context(|| format!("read {}", ready_report_path.display()))?,
    )
    .with_context(|| format!("parse {}", ready_report_path.display()))?;
    let ready_summary: bijux_dna_domain_bam::BamHaplogroupReadinessV1 = serde_json::from_str(
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
    let insufficient_summary: bijux_dna_domain_bam::BamHaplogroupReadinessV1 =
        serde_json::from_str(
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
        && ready_case.tool_id == EXPECTED_TOOL_IDS[0]
        && insufficient_case.tool_id == EXPECTED_TOOL_IDS[0]
        && ready_case.sample_id == EXPECTED_SAMPLE_ID
        && insufficient_case.sample_id == EXPECTED_SAMPLE_ID
        && ready_case.reference_panel_id == EXPECTED_REFERENCE_PANEL_ID
        && insufficient_case.reference_panel_id == EXPECTED_REFERENCE_PANEL_ID
        && ready_case.reference_build == EXPECTED_REFERENCE_BUILD
        && insufficient_case.reference_build == EXPECTED_REFERENCE_BUILD
        && ready_case.population_scope == EXPECTED_POPULATION_SCOPE
        && insufficient_case.population_scope == EXPECTED_POPULATION_SCOPE
        && ready_case.expectation_matched
        && insufficient_case.expectation_matched
        && required_outputs_present(ready_case)
        && required_outputs_present(insufficient_case)
        && case_artifacts_present(ready_case)
        && case_artifacts_present(insufficient_case)
        && repo_root.join(&ready_case.reference_panel).is_file()
        && repo_root.join(&ready_case.reference_fasta).is_file()
        && repo_root.join(&ready_case.input_bam).is_file();
    let ready_case_ready = report_contract_ready(
        &ready_report,
        true,
        ready_case.haplogroup_call.as_deref(),
        ready_case.confidence,
        EXPECTED_READY_STATUS,
        ready_case.minimum_coverage,
        ready_case.observed_mean_coverage,
        ready_case.markers_total,
        ready_case.markers_supported,
    ) && ready_case.ready
        && ready_case.markers_supported == ready_case.markers_total
        && ready_case.haplogroup_call.as_deref() == Some("R1b1a");
    let parser_contract_ready =
        case_parser_contract_ready(ready_case) && case_parser_contract_ready(insufficient_case);
    let ready_summary_ready = ready_summary.schema_version == EXPECTED_SUMMARY_SCHEMA_VERSION
        && ready_summary.stage_id == EXPECTED_STAGE_ID
        && ready_summary.ready
        && ready_summary.reference_build.as_deref() == Some(EXPECTED_REFERENCE_BUILD)
        && float_matches(ready_summary.minimum_coverage, ready_case.minimum_coverage)
        && float_matches(ready_summary.observed_mean_coverage, ready_case.observed_mean_coverage)
        && ready_summary.contamination_estimate == Some(ready_case.contamination_estimate)
        && ready_summary.refusal_codes.is_empty();
    let ready_stage_metrics_ready = stage_metrics_ready(
        &ready_stage_metrics,
        "ready",
        true,
        ready_case.minimum_coverage,
        ready_case.observed_mean_coverage,
        EXPECTED_READY_STATUS,
    );
    let insufficient_summary_ready = insufficient_report_contract_ready(
        &insufficient_report,
        false,
        None,
        0.0,
        EXPECTED_INSUFFICIENT_STATUS,
        insufficient_case.minimum_coverage,
        insufficient_case.observed_mean_coverage,
        insufficient_case.markers_total,
        insufficient_case.markers_supported,
    ) && !insufficient_summary.ready
        && insufficient_summary.reference_build.as_deref() == Some(EXPECTED_REFERENCE_BUILD)
        && float_matches(insufficient_summary.minimum_coverage, insufficient_case.minimum_coverage)
        && float_matches(
            insufficient_summary.observed_mean_coverage,
            insufficient_case.observed_mean_coverage,
        )
        && insufficient_summary.contamination_estimate
            == Some(insufficient_case.contamination_estimate)
        && insufficient_summary.refusal_codes
            == vec!["coverage_below_haplogroup_minimum".to_string()];
    let insufficient_stage_metrics_ready = stage_metrics_ready(
        &insufficient_stage_metrics,
        "insufficient",
        false,
        insufficient_case.minimum_coverage,
        insufficient_case.observed_mean_coverage,
        EXPECTED_INSUFFICIENT_STATUS,
    );
    let coverage_gate_ready = ready_case.minimum_coverage <= ready_case.observed_mean_coverage
        && insufficient_case.minimum_coverage > insufficient_case.observed_mean_coverage;
    let tool_specific_artifact_ready = tool_specific_artifact_ready(
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
    if !coverage_gate_ready {
        missing_surfaces.push("coverage_gate".to_string());
    }
    if !tool_specific_artifact_ready {
        missing_surfaces.push("tool_specific_artifact".to_string());
    }

    let coverage_status =
        if missing_surfaces.is_empty() { "complete".to_string() } else { "incomplete".to_string() };
    let reason = if missing_surfaces.is_empty() {
        format!(
            "binding `{}` / `{}` keeps retained readiness plus ready and coverage-gated haplogroups proof",
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

    Ok(BamHaplogroupsCompleteRow {
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
        ready_case_report_path: ready_case.haplogroups_report.clone(),
        ready_case_summary_path: ready_case.haplogroups_summary.clone(),
        ready_case_stage_metrics_path: ready_case.stage_metrics.clone(),
        ready_case_tool_specific_artifact_path: ready_case.haplogroup_report.clone(),
        insufficient_case_report_path: insufficient_case.haplogroups_report.clone(),
        insufficient_case_summary_path: insufficient_case.haplogroups_summary.clone(),
        insufficient_case_stage_metrics_path: insufficient_case.stage_metrics.clone(),
        insufficient_case_tool_specific_artifact_path: insufficient_case.haplogroup_report.clone(),
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
        local_smoke_sample_id: smoke_report.sample_id.clone(),
        local_smoke_reference_panel_path: ready_case.reference_panel.clone(),
        local_smoke_reference_panel_id: ready_case.reference_panel_id.clone(),
        local_smoke_reference_build: ready_case.reference_build.clone(),
        local_smoke_population_scope: ready_case.population_scope.clone(),
        local_smoke_input_bam: ready_case.input_bam.clone(),
        local_smoke_reference_fasta: ready_case.reference_fasta.clone(),
        ready_case_haplogroup_call: ready_case.haplogroup_call.clone(),
        ready_case_confidence: ready_case.confidence,
        ready_case_status: ready_case.status.clone(),
        ready_case_markers_total: ready_case.markers_total,
        ready_case_markers_supported: ready_case.markers_supported,
        insufficient_case_haplogroup_call: insufficient_case.haplogroup_call.clone(),
        insufficient_case_confidence: insufficient_case.confidence,
        insufficient_case_status: insufficient_case.status.clone(),
        insufficient_case_markers_total: insufficient_case.markers_total,
        insufficient_case_markers_supported: insufficient_case.markers_supported,
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
        coverage_gate_ready,
        tool_specific_artifact_ready,
        coverage_status,
        missing_surfaces,
        reason,
    })
}

fn required_outputs_present(case: &LocalHaplogroupsSmokeCaseReport) -> bool {
    REQUIRED_OUTPUT_IDS
        .iter()
        .all(|output_id| case.declared_output_ids.iter().any(|candidate| candidate == output_id))
}

fn case_artifacts_present(case: &LocalHaplogroupsSmokeCaseReport) -> bool {
    case.artifact_paths.iter().any(|path| path == &case.haplogroups_report)
        && case.artifact_paths.iter().any(|path| path == &case.haplogroups_summary)
        && case.artifact_paths.iter().any(|path| path == &case.stage_metrics)
        && case.artifact_paths.iter().any(|path| path == &case.haplogroup_report)
}

fn case_parser_contract_ready(case: &LocalHaplogroupsSmokeCaseReport) -> bool {
    case.artifact_paths.iter().any(|path| path == &case.haplogroups_report)
        && case.artifact_paths.iter().any(|path| path == &case.haplogroups_summary)
        && case.artifact_paths.iter().any(|path| path == &case.haplogroup_report)
        && case.artifact_paths.iter().any(|path| path == &case.stage_metrics)
}

fn report_contract_ready(
    payload: &serde_json::Value,
    expected_ready: bool,
    expected_call: Option<&str>,
    expected_confidence: f64,
    expected_status: &str,
    expected_minimum_coverage: f64,
    expected_observed_mean_coverage: f64,
    expected_markers_total: usize,
    expected_markers_supported: usize,
) -> bool {
    payload.get("schema_version").and_then(serde_json::Value::as_str)
        == Some(EXPECTED_REPORT_SCHEMA_VERSION)
        && payload.get("stage_id").and_then(serde_json::Value::as_str) == Some(EXPECTED_STAGE_ID)
        && payload.get("tool_id").and_then(serde_json::Value::as_str) == Some(EXPECTED_TOOL_IDS[0])
        && payload.get("sample_id").and_then(serde_json::Value::as_str) == Some(EXPECTED_SAMPLE_ID)
        && payload.get("reference_panel_id").and_then(serde_json::Value::as_str)
            == Some(EXPECTED_REFERENCE_PANEL_ID)
        && payload.get("reference_build").and_then(serde_json::Value::as_str)
            == Some(EXPECTED_REFERENCE_BUILD)
        && payload.get("population_scope").and_then(serde_json::Value::as_str)
            == Some(EXPECTED_POPULATION_SCOPE)
        && optional_string_field(payload, "haplogroup_call") == expected_call
        && payload.get("confidence").and_then(serde_json::Value::as_f64)
            == Some(expected_confidence)
        && payload.get("status").and_then(serde_json::Value::as_str) == Some(expected_status)
        && payload.get("markers_total").and_then(serde_json::Value::as_u64)
            == Some(expected_markers_total as u64)
        && payload.get("markers_supported").and_then(serde_json::Value::as_u64)
            == Some(expected_markers_supported as u64)
        && payload
            .get("coverage_gate")
            .and_then(|value| value.get("min_coverage"))
            .and_then(serde_json::Value::as_f64)
            == Some(expected_minimum_coverage)
        && payload
            .get("coverage_gate")
            .and_then(|value| value.get("observed_mean_coverage"))
            .and_then(serde_json::Value::as_f64)
            == Some(expected_observed_mean_coverage)
        && (expected_ready
            || payload.get("haplogroup_call").is_some_and(serde_json::Value::is_null))
}

fn insufficient_report_contract_ready(
    payload: &serde_json::Value,
    expected_ready: bool,
    expected_call: Option<&str>,
    expected_confidence: f64,
    expected_status: &str,
    expected_minimum_coverage: f64,
    expected_observed_mean_coverage: f64,
    expected_markers_total: usize,
    expected_markers_supported: usize,
) -> bool {
    report_contract_ready(
        payload,
        expected_ready,
        expected_call,
        expected_confidence,
        expected_status,
        expected_minimum_coverage,
        expected_observed_mean_coverage,
        expected_markers_total,
        expected_markers_supported,
    )
}

fn stage_metrics_ready(
    payload: &serde_json::Value,
    proof_case: &str,
    expected_ready: bool,
    expected_minimum_coverage: f64,
    expected_observed_mean_coverage: f64,
    expected_status: &str,
) -> bool {
    payload.get("schema_version").and_then(serde_json::Value::as_str)
        == Some(EXPECTED_STAGE_METRICS_SCHEMA_VERSION)
        && payload.get("stage_id").and_then(serde_json::Value::as_str) == Some(EXPECTED_STAGE_ID)
        && payload.get("tool_id").and_then(serde_json::Value::as_str) == Some(EXPECTED_TOOL_IDS[0])
        && payload.get("sample_id").and_then(serde_json::Value::as_str) == Some(EXPECTED_SAMPLE_ID)
        && payload.get("proof_case").and_then(serde_json::Value::as_str) == Some(proof_case)
        && payload.get("expected_ready").and_then(serde_json::Value::as_bool)
            == Some(expected_ready)
        && payload.get("ready").and_then(serde_json::Value::as_bool) == Some(expected_ready)
        && payload.get("minimum_coverage").and_then(serde_json::Value::as_f64)
            == Some(expected_minimum_coverage)
        && payload.get("observed_mean_coverage").and_then(serde_json::Value::as_f64)
            == Some(expected_observed_mean_coverage)
        && payload.get("status").and_then(serde_json::Value::as_str) == Some(expected_status)
        && payload.get("expectation_matched").and_then(serde_json::Value::as_bool) == Some(true)
}

fn optional_string_field<'a>(payload: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    payload.get(key).and_then(serde_json::Value::as_str)
}

fn tool_specific_artifact_ready(
    ready_path: &Path,
    insufficient_path: &Path,
    ready_case: &LocalHaplogroupsSmokeCaseReport,
    insufficient_case: &LocalHaplogroupsSmokeCaseReport,
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

    let ready_ok = ready_payload.get("artifact_id").and_then(serde_json::Value::as_str)
        == Some("haplogroup_report")
        && ready_payload.get("stage_id").and_then(serde_json::Value::as_str)
            == Some(EXPECTED_STAGE_ID)
        && ready_payload.get("tool_id").and_then(serde_json::Value::as_str)
            == Some(EXPECTED_TOOL_IDS[0])
        && ready_payload.get("proof_case").and_then(serde_json::Value::as_str) == Some("ready")
        && optional_string_field(&ready_payload, "haplogroup")
            == ready_case.haplogroup_call.as_deref()
        && ready_payload.get("confidence").and_then(serde_json::Value::as_f64)
            == Some(ready_case.confidence)
        && ready_payload.get("status").and_then(serde_json::Value::as_str)
            == Some(EXPECTED_READY_STATUS)
        && ready_payload.get("reference_panel_id").and_then(serde_json::Value::as_str)
            == Some(EXPECTED_REFERENCE_PANEL_ID)
        && ready_payload.get("markers_total").and_then(serde_json::Value::as_u64)
            == Some(ready_case.markers_total as u64)
        && ready_payload.get("markers_supported").and_then(serde_json::Value::as_u64)
            == Some(ready_case.markers_supported as u64);
    let insufficient_ok =
        insufficient_payload.get("artifact_id").and_then(serde_json::Value::as_str)
            == Some("haplogroup_report")
            && insufficient_payload.get("stage_id").and_then(serde_json::Value::as_str)
                == Some(EXPECTED_STAGE_ID)
            && insufficient_payload.get("tool_id").and_then(serde_json::Value::as_str)
                == Some(EXPECTED_TOOL_IDS[0])
            && insufficient_payload.get("proof_case").and_then(serde_json::Value::as_str)
                == Some("insufficient")
            && insufficient_payload.get("haplogroup").is_some_and(serde_json::Value::is_null)
            && insufficient_payload.get("confidence").and_then(serde_json::Value::as_f64)
                == Some(insufficient_case.confidence)
            && insufficient_payload.get("status").and_then(serde_json::Value::as_str)
                == Some(EXPECTED_INSUFFICIENT_STATUS)
            && insufficient_payload.get("reference_panel_id").and_then(serde_json::Value::as_str)
                == Some(EXPECTED_REFERENCE_PANEL_ID)
            && insufficient_payload.get("markers_total").and_then(serde_json::Value::as_u64)
                == Some(insufficient_case.markers_total as u64)
            && insufficient_payload.get("markers_supported").and_then(serde_json::Value::as_u64)
                == Some(insufficient_case.markers_supported as u64);
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
