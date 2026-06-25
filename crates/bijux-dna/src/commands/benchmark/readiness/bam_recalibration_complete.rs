use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use super::bam_recalibration_genotyping_ready::{
    render_bam_recalibration_genotyping_ready, BamRecalibrationGenotypingReadyRow,
    DEFAULT_BAM_RECALIBRATION_GENOTYPING_READY_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_BAM_RECALIBRATION_COMPLETE_PATH: &str =
    "benchmarks/readiness/bam/stages/bam.recalibration.complete.json";
const BAM_RECALIBRATION_COMPLETE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_recalibration_complete.v1";
const EXPECTED_STAGE_ID: &str = "bam.recalibration";
const EXPECTED_TOOL_ID: &str = "gatk";
const EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION: &str = "bijux.bam.recalibration.local_smoke.report.v1";
const EXPECTED_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.recalibration.v1";
const EXPECTED_STAGE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bam.recalibration.local_smoke.metrics.v1";
const EXPECTED_SAMPLE_ID: &str = "human_like_recalibration_low_coverage";
const EXPECTED_KNOWN_SITES_PATH: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/variants/human_like_recalibration_known_sites.vcf";
const EXPECTED_KNOWN_SITES_ASSET_ID: &str = "human_like_recalibration_known_sites";
const EXPECTED_REFERENCE_FASTA: &str =
    "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta";
const EXPECTED_REQUESTED_MODE: &str = "standard";
const EXPECTED_EFFECTIVE_MODE: &str = "skip";
const EXPECTED_STATUS: &str = "skipped";
const EXPECTED_REASON: &str = "coverage_below_gate";
const EXPECTED_MIN_MEAN_COVERAGE: f64 = 0.2;
const EXPECTED_MIN_BREADTH_1X: f64 = 0.2;
const EXPECTED_OBSERVED_MEAN_COVERAGE: f64 = 0.192;
const EXPECTED_OBSERVED_BREADTH_1X: f64 = 0.192;
const CHECKED_SURFACE_COUNT: usize = 13;
const REQUIRED_OUTPUT_IDS: [&str; 5] =
    ["recal_bam", "recal_bai", "recal_report", "summary", "stage_metrics"];

#[derive(Debug, Clone, Deserialize)]
struct LocalRecalibrationSmokeReport {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    sample_id: String,
    expectation_matched: bool,
    input_bam: String,
    reference_fasta: String,
    known_sites: Vec<String>,
    known_sites_asset_ids: Vec<String>,
    requested_mode: String,
    effective_mode: String,
    status: String,
    reason: String,
    coverage_gate: LocalRecalibrationCoverageGate,
    observed_mean_coverage: f64,
    observed_breadth_1x: f64,
    output_bam_present: bool,
    recalibration_report_present: bool,
    recalibrated_bam: String,
    recalibration_report: String,
    recalibration_summary: String,
    stage_metrics: String,
}

#[derive(Debug, Clone, Deserialize)]
struct LocalRecalibrationCoverageGate {
    min_mean_coverage: f64,
    min_breadth_1x: f64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamRecalibrationCompleteRow {
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
    pub(crate) local_smoke_summary_path: String,
    pub(crate) local_smoke_stage_metrics_path: String,
    pub(crate) local_smoke_recalibration_report_path: String,
    pub(crate) local_smoke_recalibrated_bam_path: String,
    pub(crate) local_smoke_summary_schema_version: String,
    pub(crate) local_smoke_stage_metrics_schema_version: String,
    pub(crate) local_smoke_sample_id: String,
    pub(crate) local_smoke_input_bam: String,
    pub(crate) local_smoke_reference_fasta: String,
    pub(crate) local_smoke_known_sites: Vec<String>,
    pub(crate) local_smoke_known_sites_asset_ids: Vec<String>,
    pub(crate) local_smoke_requested_mode: String,
    pub(crate) local_smoke_effective_mode: String,
    pub(crate) local_smoke_status: String,
    pub(crate) local_smoke_reason: String,
    pub(crate) local_smoke_min_mean_coverage: f64,
    pub(crate) local_smoke_min_breadth_1x: f64,
    pub(crate) local_smoke_observed_mean_coverage: f64,
    pub(crate) local_smoke_observed_breadth_1x: f64,
    pub(crate) active_scope_ready: bool,
    pub(crate) command_ready: bool,
    pub(crate) output_ready: bool,
    pub(crate) parser_ready: bool,
    pub(crate) expected_result_ready: bool,
    pub(crate) report_ready: bool,
    pub(crate) schema_ready: bool,
    pub(crate) local_smoke_ready: bool,
    pub(crate) summary_ready: bool,
    pub(crate) stage_metrics_ready: bool,
    pub(crate) skip_behavior_ready: bool,
    pub(crate) known_sites_identity_ready: bool,
    pub(crate) recalibration_report_ready: bool,
    pub(crate) coverage_status: String,
    pub(crate) missing_surfaces: Vec<String>,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamRecalibrationCompleteReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) active_row_count: usize,
    pub(crate) complete_row_count: usize,
    pub(crate) incomplete_row_count: usize,
    pub(crate) checked_surface_count: usize,
    pub(crate) expected_tool_ids: Vec<String>,
    pub(crate) required_output_ids: Vec<String>,
    pub(crate) local_smoke_sample_id: String,
    pub(crate) toolset_ready: bool,
    pub(crate) violation_count: usize,
    pub(crate) ok: bool,
    pub(crate) rows: Vec<BamRecalibrationCompleteRow>,
    pub(crate) violations: Vec<BamRecalibrationCompleteRow>,
}

pub(crate) fn run_render_bam_recalibration_complete(
    args: &parse::BenchReadinessRenderBamRecalibrationCompleteArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_recalibration_complete(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_RECALIBRATION_COMPLETE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_bam_recalibration_complete(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamRecalibrationCompleteReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let report = build_bam_recalibration_complete_report(repo_root, &output_path)?;
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    if !report.ok {
        return Err(anyhow!(
            "bam recalibration completion must keep active scope, command, output, parser, expected-result, report, schema, local smoke, known-sites identity, and deterministic skip proof"
        ));
    }
    Ok(report)
}

fn build_bam_recalibration_complete_report(
    repo_root: &Path,
    output_path: &Path,
) -> Result<BamRecalibrationCompleteReport> {
    let readiness = render_bam_recalibration_genotyping_ready(
        repo_root,
        PathBuf::from(DEFAULT_BAM_RECALIBRATION_GENOTYPING_READY_PATH),
    )?;
    let smoke_path = bijux_dna_api::v1::api::bam::write_local_recalibration_smoke_report()?;
    let smoke_report: LocalRecalibrationSmokeReport = serde_json::from_str(
        &fs::read_to_string(&smoke_path)
            .with_context(|| format!("read {}", smoke_path.display()))?,
    )
    .with_context(|| format!("parse {}", smoke_path.display()))?;

    if smoke_report.schema_version != EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION {
        return Err(anyhow!(
            "unexpected bam.recalibration local-smoke schema `{}`",
            smoke_report.schema_version
        ));
    }
    if smoke_report.stage_id != EXPECTED_STAGE_ID {
        return Err(anyhow!(
            "unexpected bam.recalibration local-smoke stage `{}`",
            smoke_report.stage_id
        ));
    }
    if smoke_report.tool_id != EXPECTED_TOOL_ID {
        return Err(anyhow!(
            "unexpected bam.recalibration local-smoke tool `{}`",
            smoke_report.tool_id
        ));
    }

    let mut rows = readiness
        .rows
        .into_iter()
        .filter(|row| row.stage_id == EXPECTED_STAGE_ID)
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));
    let expected_tool_ids = vec![EXPECTED_TOOL_ID.to_string()];
    let observed_tool_ids = rows.iter().map(|row| row.tool_id.clone()).collect::<Vec<_>>();
    if observed_tool_ids != expected_tool_ids {
        return Err(anyhow!(
            "bam.recalibration readiness rows drifted: observed={:?} expected={:?}",
            observed_tool_ids,
            expected_tool_ids
        ));
    }

    let report_rows = rows
        .iter()
        .map(|readiness_row| {
            build_bam_recalibration_complete_row(
                repo_root,
                &smoke_path,
                readiness_row,
                &smoke_report,
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

    Ok(BamRecalibrationCompleteReport {
        schema_version: BAM_RECALIBRATION_COMPLETE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        active_row_count: report_rows.len(),
        complete_row_count,
        incomplete_row_count,
        checked_surface_count: CHECKED_SURFACE_COUNT,
        expected_tool_ids,
        required_output_ids: REQUIRED_OUTPUT_IDS.iter().map(|value| (*value).to_string()).collect(),
        local_smoke_sample_id: smoke_report.sample_id.clone(),
        toolset_ready: violations.is_empty(),
        violation_count: violations.len(),
        ok: violations.is_empty(),
        rows: report_rows,
        violations,
    })
}

fn build_bam_recalibration_complete_row(
    repo_root: &Path,
    smoke_path: &Path,
    readiness_row: &BamRecalibrationGenotypingReadyRow,
    smoke_report: &LocalRecalibrationSmokeReport,
) -> Result<BamRecalibrationCompleteRow> {
    let summary_path = repo_root.join(&smoke_report.recalibration_summary);
    let stage_metrics_path = repo_root.join(&smoke_report.stage_metrics);
    let recalibration_report_path = repo_root.join(&smoke_report.recalibration_report);
    let recalibrated_bam_path = repo_root.join(&smoke_report.recalibrated_bam);

    let summary: bijux_dna_domain_bam::BamRecalibrationSummaryV1 = serde_json::from_str(
        &fs::read_to_string(&summary_path)
            .with_context(|| format!("read {}", summary_path.display()))?,
    )
    .with_context(|| format!("parse {}", summary_path.display()))?;
    let stage_metrics: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&stage_metrics_path)
            .with_context(|| format!("read {}", stage_metrics_path.display()))?,
    )
    .with_context(|| format!("parse {}", stage_metrics_path.display()))?;
    let recalibration_report = fs::read_to_string(&recalibration_report_path)
        .with_context(|| format!("read {}", recalibration_report_path.display()))?;

    let local_smoke_ready = smoke_report.expectation_matched
        && smoke_report.sample_id == EXPECTED_SAMPLE_ID
        && smoke_report.input_bam
            == "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_recalibration_low_coverage.sam"
        && smoke_report.reference_fasta == EXPECTED_REFERENCE_FASTA
        && smoke_report.known_sites == vec![EXPECTED_KNOWN_SITES_PATH.to_string()]
        && smoke_report.known_sites_asset_ids
            == vec![EXPECTED_KNOWN_SITES_ASSET_ID.to_string()]
        && smoke_report.requested_mode == EXPECTED_REQUESTED_MODE
        && smoke_report.effective_mode == EXPECTED_EFFECTIVE_MODE
        && smoke_report.status == EXPECTED_STATUS
        && smoke_report.reason == EXPECTED_REASON
        && float_matches(smoke_report.coverage_gate.min_mean_coverage, EXPECTED_MIN_MEAN_COVERAGE)
        && float_matches(smoke_report.coverage_gate.min_breadth_1x, EXPECTED_MIN_BREADTH_1X)
        && float_matches(smoke_report.observed_mean_coverage, EXPECTED_OBSERVED_MEAN_COVERAGE)
        && float_matches(smoke_report.observed_breadth_1x, EXPECTED_OBSERVED_BREADTH_1X)
        && smoke_report.output_bam_present
        && smoke_report.recalibration_report_present
        && recalibrated_bam_path.is_file()
        && recalibration_report_path.is_file()
        && summary_path.is_file()
        && stage_metrics_path.is_file();

    let summary_ready = summary.schema_version == EXPECTED_SUMMARY_SCHEMA_VERSION
        && summary.stage_id == EXPECTED_STAGE_ID
        && path_relative_to_repo(repo_root, &summary.input_bam) == smoke_report.input_bam
        && summary
            .reference_fasta
            .as_ref()
            .is_some_and(|path| path_relative_to_repo(repo_root, path) == EXPECTED_REFERENCE_FASTA)
        && summary.known_sites.len() == 1
        && path_relative_to_repo(repo_root, &summary.known_sites[0]) == EXPECTED_KNOWN_SITES_PATH
        && summary.requested_mode == bijux_dna_domain_bam::params::BqsrMode::Standard
        && summary.effective_mode == bijux_dna_domain_bam::params::BqsrMode::Skip
        && summary.status == EXPECTED_STATUS
        && summary.reason == EXPECTED_REASON
        && float_matches(summary.coverage_gate.min_mean_coverage, EXPECTED_MIN_MEAN_COVERAGE)
        && float_matches(summary.coverage_gate.min_breadth_1x, EXPECTED_MIN_BREADTH_1X)
        && float_matches(summary.observed_mean_coverage, EXPECTED_OBSERVED_MEAN_COVERAGE)
        && float_matches(summary.observed_breadth_1x, EXPECTED_OBSERVED_BREADTH_1X)
        && summary.output_bam_present
        && summary.recalibration_report_present;

    let stage_metrics_ready = stage_metrics
        .get("schema_version")
        .and_then(serde_json::Value::as_str)
        == Some(EXPECTED_STAGE_METRICS_SCHEMA_VERSION)
        && stage_metrics.get("expected_requested_mode").and_then(serde_json::Value::as_str)
            == Some(EXPECTED_REQUESTED_MODE)
        && stage_metrics.get("requested_mode").and_then(serde_json::Value::as_str)
            == Some(EXPECTED_REQUESTED_MODE)
        && stage_metrics.get("expected_effective_mode").and_then(serde_json::Value::as_str)
            == Some(EXPECTED_EFFECTIVE_MODE)
        && stage_metrics.get("effective_mode").and_then(serde_json::Value::as_str)
            == Some(EXPECTED_EFFECTIVE_MODE)
        && stage_metrics.get("expected_status").and_then(serde_json::Value::as_str)
            == Some(EXPECTED_STATUS)
        && stage_metrics.get("status").and_then(serde_json::Value::as_str) == Some(EXPECTED_STATUS)
        && stage_metrics.get("expected_reason").and_then(serde_json::Value::as_str)
            == Some(EXPECTED_REASON)
        && stage_metrics.get("reason").and_then(serde_json::Value::as_str) == Some(EXPECTED_REASON)
        && stage_metrics.get("expected_known_sites").and_then(serde_json::Value::as_array)
            == Some(&vec![serde_json::json!(EXPECTED_KNOWN_SITES_PATH)])
        && stage_metrics.get("known_sites").and_then(serde_json::Value::as_array)
            == Some(&vec![serde_json::json!(EXPECTED_KNOWN_SITES_PATH)])
        && stage_metrics
            .get("expected_known_sites_asset_ids")
            .and_then(serde_json::Value::as_array)
            == Some(&vec![serde_json::json!(EXPECTED_KNOWN_SITES_ASSET_ID)])
        && stage_metrics.get("known_sites_asset_ids").and_then(serde_json::Value::as_array)
            == Some(&vec![serde_json::json!(EXPECTED_KNOWN_SITES_ASSET_ID)])
        && stage_metrics.get("expectation_matched").and_then(serde_json::Value::as_bool)
            == Some(true)
        && stage_metrics.get("output_bam_present").and_then(serde_json::Value::as_bool)
            == Some(true)
        && stage_metrics.get("recalibration_report_present").and_then(serde_json::Value::as_bool)
            == Some(true);

    let skip_behavior_ready = smoke_report.requested_mode == EXPECTED_REQUESTED_MODE
        && smoke_report.effective_mode == EXPECTED_EFFECTIVE_MODE
        && smoke_report.status == EXPECTED_STATUS
        && smoke_report.reason == EXPECTED_REASON
        && smoke_report.observed_mean_coverage < smoke_report.coverage_gate.min_mean_coverage
        && smoke_report.observed_breadth_1x < smoke_report.coverage_gate.min_breadth_1x;

    let known_sites_identity_ready = smoke_report.known_sites
        == vec![EXPECTED_KNOWN_SITES_PATH.to_string()]
        && smoke_report.known_sites_asset_ids == vec![EXPECTED_KNOWN_SITES_ASSET_ID.to_string()]
        && summary.known_sites.len() == 1
        && path_relative_to_repo(repo_root, &summary.known_sites[0]) == EXPECTED_KNOWN_SITES_PATH;

    let recalibration_report_ready = recalibration_report.contains("status=skipped")
        && recalibration_report.contains("reason=coverage_below_gate")
        && recalibration_report.contains("requested_mode=standard")
        && recalibration_report.contains("effective_mode=skip")
        && recalibration_report.contains(
            "known_sites=benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/variants/human_like_recalibration_known_sites.vcf",
        )
        && recalibration_report.contains("coverage_gate.min_mean_coverage=0.2")
        && recalibration_report.contains("coverage_gate.min_breadth_1x=0.2");

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
    if !summary_ready {
        missing_surfaces.push("summary".to_string());
    }
    if !stage_metrics_ready {
        missing_surfaces.push("stage_metrics".to_string());
    }
    if !skip_behavior_ready {
        missing_surfaces.push("skip_behavior".to_string());
    }
    if !known_sites_identity_ready {
        missing_surfaces.push("known_sites_identity".to_string());
    }
    if !recalibration_report_ready {
        missing_surfaces.push("recalibration_report".to_string());
    }

    let coverage_status =
        if missing_surfaces.is_empty() { "complete".to_string() } else { "incomplete".to_string() };
    let reason = if missing_surfaces.is_empty() {
        format!(
            "binding `{}` / `{}` keeps retained readiness plus deterministic recalibration skip proof",
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

    Ok(BamRecalibrationCompleteRow {
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
        local_smoke_summary_path: smoke_report.recalibration_summary.clone(),
        local_smoke_stage_metrics_path: smoke_report.stage_metrics.clone(),
        local_smoke_recalibration_report_path: smoke_report.recalibration_report.clone(),
        local_smoke_recalibrated_bam_path: smoke_report.recalibrated_bam.clone(),
        local_smoke_summary_schema_version: summary.schema_version.clone(),
        local_smoke_stage_metrics_schema_version: required_string_field(
            &stage_metrics,
            "schema_version",
            &stage_metrics_path,
        )?,
        local_smoke_sample_id: smoke_report.sample_id.clone(),
        local_smoke_input_bam: smoke_report.input_bam.clone(),
        local_smoke_reference_fasta: smoke_report.reference_fasta.clone(),
        local_smoke_known_sites: smoke_report.known_sites.clone(),
        local_smoke_known_sites_asset_ids: smoke_report.known_sites_asset_ids.clone(),
        local_smoke_requested_mode: smoke_report.requested_mode.clone(),
        local_smoke_effective_mode: smoke_report.effective_mode.clone(),
        local_smoke_status: smoke_report.status.clone(),
        local_smoke_reason: smoke_report.reason.clone(),
        local_smoke_min_mean_coverage: smoke_report.coverage_gate.min_mean_coverage,
        local_smoke_min_breadth_1x: smoke_report.coverage_gate.min_breadth_1x,
        local_smoke_observed_mean_coverage: smoke_report.observed_mean_coverage,
        local_smoke_observed_breadth_1x: smoke_report.observed_breadth_1x,
        active_scope_ready: readiness_row.active_scope_ready,
        command_ready: readiness_row.command_ready,
        output_ready: readiness_row.output_ready,
        parser_ready: readiness_row.parser_ready,
        expected_result_ready: readiness_row.expected_result_ready,
        report_ready: readiness_row.report_ready,
        schema_ready: readiness_row.schema_ready,
        local_smoke_ready,
        summary_ready,
        stage_metrics_ready,
        skip_behavior_ready,
        known_sites_identity_ready,
        recalibration_report_ready,
        coverage_status,
        missing_surfaces,
        reason,
    })
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
    (left - right).abs() <= 1e-12
}
