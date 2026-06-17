use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use super::bam_overlap_endogenous_ready::{
    render_bam_overlap_endogenous_ready, BamOverlapEndogenousReadyRow,
    DEFAULT_BAM_OVERLAP_ENDOGENOUS_READY_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_BAM_ENDOGENOUS_CONTENT_COMPLETE_PATH: &str =
    "benchmarks/readiness/bam/stages/bam.endogenous_content.complete.json";
const BAM_ENDOGENOUS_CONTENT_COMPLETE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_endogenous_content_complete.v1";
const EXPECTED_STAGE_ID: &str = "bam.endogenous_content";
const EXPECTED_TOOL_ID: &str = "samtools";
const EXPECTED_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.endogenous_content.v1";
const EXPECTED_STAGE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bam.endogenous_content.local_smoke.metrics.v1";
const EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bam.endogenous_content.local_smoke.report.v1";
const EXPECTED_SAMPLE_ID: &str = "human_like_endogenous_partial_mapping";
const EXPECTED_METHOD: &str = "mapped_fraction_from_flagstat";
const EXPECTED_HOST_REFERENCE_SCOPE: &str = "human_host";
const EXPECTED_MAPPED_READS: u64 = 3;
const EXPECTED_ENDOGENOUS_READS: u64 = 3;
const EXPECTED_TOTAL_READS: u64 = 5;
const EXPECTED_ENDOGENOUS_FRACTION: f64 = 0.6;
const CHECKED_SURFACE_COUNT: usize = 11;
const REQUIRED_SUMMARY_METRIC_NAMES: [&str; 9] = [
    "method",
    "mapped_reads",
    "endogenous_reads",
    "total_reads",
    "endogenous_fraction",
    "prealignment_fraction",
    "postalignment_fraction",
    "host_reference_scope",
    "caveats",
];
const REQUIRED_NORMALIZED_METRIC_NAMES: [&str; 17] = [
    "expected_method",
    "method",
    "expected_host_reference_scope",
    "host_reference_scope",
    "expected_mapped_reads",
    "mapped_reads",
    "mapped_read_delta",
    "expected_endogenous_reads",
    "endogenous_reads",
    "expected_total_reads",
    "total_reads",
    "total_read_delta",
    "expected_endogenous_fraction",
    "endogenous_fraction",
    "endogenous_fraction_delta",
    "prealignment_fraction",
    "expectation_matched",
];
const OPTIONAL_METRIC_NAMES: [&str; 1] = ["contaminant_reads"];

#[derive(Debug, Clone, Deserialize)]
struct LocalEndogenousContentSmokeReport {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    expectation_matched: bool,
    input_bam: String,
    method: String,
    host_reference_scope: Option<String>,
    mapped_reads: u64,
    endogenous_reads: u64,
    total_reads: u64,
    endogenous_fraction: f64,
    prealignment_fraction: Option<f64>,
    endogenous_report: String,
    endogenous_summary: String,
    stage_metrics: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LocalEndogenousContentSmokeMetrics {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    expected_method: String,
    method: String,
    expected_host_reference_scope: String,
    host_reference_scope: Option<String>,
    expected_mapped_reads: u64,
    mapped_reads: u64,
    mapped_read_delta: i64,
    expected_endogenous_reads: u64,
    endogenous_reads: u64,
    expected_total_reads: u64,
    total_reads: u64,
    total_read_delta: i64,
    expected_endogenous_fraction: f64,
    endogenous_fraction: f64,
    endogenous_fraction_delta: f64,
    prealignment_fraction: Option<f64>,
    expectation_matched: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamEndogenousContentCompleteRow {
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
    pub(crate) required_schema_keys: Vec<String>,
    pub(crate) schema_required_keys: Vec<String>,
    pub(crate) required_local_smoke_fields: Vec<String>,
    pub(crate) required_summary_metric_names: Vec<String>,
    pub(crate) required_normalized_metric_names: Vec<String>,
    pub(crate) optional_metric_names: Vec<String>,
    pub(crate) available_optional_metric_names: Vec<String>,
    pub(crate) active_scope_proof_path: String,
    pub(crate) command_proof_path: String,
    pub(crate) output_contract_proof_path: String,
    pub(crate) parser_proof_path: String,
    pub(crate) expected_result_proof_path: String,
    pub(crate) report_map_proof_path: String,
    pub(crate) schema_proof_path: String,
    pub(crate) local_smoke_proof_path: String,
    pub(crate) endogenous_report_path: String,
    pub(crate) endogenous_summary_path: String,
    pub(crate) stage_metrics_path: String,
    pub(crate) local_smoke_schema_version: String,
    pub(crate) local_smoke_sample_id: String,
    pub(crate) local_smoke_method: String,
    pub(crate) local_smoke_input_bam: String,
    pub(crate) local_smoke_expectation_matched: bool,
    pub(crate) local_smoke_host_reference_scope: Option<String>,
    pub(crate) local_smoke_mapped_reads: u64,
    pub(crate) local_smoke_endogenous_reads: u64,
    pub(crate) local_smoke_total_reads: u64,
    pub(crate) local_smoke_endogenous_fraction: f64,
    pub(crate) local_smoke_prealignment_fraction: Option<f64>,
    pub(crate) summary_schema_version: String,
    pub(crate) summary_method: String,
    pub(crate) summary_host_reference_scope: Option<String>,
    pub(crate) summary_mapped_reads: u64,
    pub(crate) summary_endogenous_reads: u64,
    pub(crate) summary_total_reads: u64,
    pub(crate) summary_endogenous_fraction: f64,
    pub(crate) summary_prealignment_fraction: Option<f64>,
    pub(crate) summary_postalignment_fraction: f64,
    pub(crate) summary_caveats: Vec<String>,
    pub(crate) contaminant_reads: Option<u64>,
    pub(crate) normalized_metrics_schema_version: String,
    pub(crate) normalized_metrics_sample_id: String,
    pub(crate) normalized_metrics: LocalEndogenousContentSmokeMetrics,
    pub(crate) active_scope_ready: bool,
    pub(crate) command_ready: bool,
    pub(crate) output_ready: bool,
    pub(crate) parser_ready: bool,
    pub(crate) expected_result_ready: bool,
    pub(crate) report_ready: bool,
    pub(crate) schema_ready: bool,
    pub(crate) local_smoke_ready: bool,
    pub(crate) summary_ready: bool,
    pub(crate) normalized_metrics_ready: bool,
    pub(crate) endogenous_metric_consistency_ready: bool,
    pub(crate) coverage_status: String,
    pub(crate) missing_surfaces: Vec<String>,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamEndogenousContentCompleteReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) active_row_count: usize,
    pub(crate) complete_row_count: usize,
    pub(crate) incomplete_row_count: usize,
    pub(crate) checked_surface_count: usize,
    pub(crate) required_output_ids: Vec<String>,
    pub(crate) required_summary_metric_names: Vec<String>,
    pub(crate) required_normalized_metric_names: Vec<String>,
    pub(crate) optional_metric_names: Vec<String>,
    pub(crate) violation_count: usize,
    pub(crate) ok: bool,
    pub(crate) rows: Vec<BamEndogenousContentCompleteRow>,
    pub(crate) violations: Vec<BamEndogenousContentCompleteRow>,
}

pub(crate) fn run_render_bam_endogenous_content_complete(
    args: &parse::BenchReadinessRenderBamEndogenousContentCompleteArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_endogenous_content_complete(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_ENDOGENOUS_CONTENT_COMPLETE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_bam_endogenous_content_complete(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamEndogenousContentCompleteReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let report = build_bam_endogenous_content_complete_report(repo_root, &output_path)?;
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    if !report.ok {
        return Err(anyhow!(
            "bam.endogenous_content must keep active scope, command, output, parser, expected-result, report, schema, local-smoke, summary, and normalized endogenous metrics complete"
        ));
    }
    Ok(report)
}

fn build_bam_endogenous_content_complete_report(
    repo_root: &Path,
    output_path: &Path,
) -> Result<BamEndogenousContentCompleteReport> {
    let overlap_endogenous_report = render_bam_overlap_endogenous_ready(
        repo_root,
        PathBuf::from(DEFAULT_BAM_OVERLAP_ENDOGENOUS_READY_PATH),
    )?;
    let endogenous_row = overlap_endogenous_report
        .rows
        .into_iter()
        .find(|row| row.stage_id == EXPECTED_STAGE_ID && row.tool_id == EXPECTED_TOOL_ID)
        .ok_or_else(|| {
            anyhow!(
                "bam overlap/endogenous readiness report is missing `{EXPECTED_STAGE_ID}` / `{EXPECTED_TOOL_ID}`"
            )
        })?;
    let row = build_endogenous_content_complete_row(repo_root, endogenous_row)?;
    let complete_row_count = usize::from(row.coverage_status == "complete");
    let rows = vec![row];
    let violations =
        rows.iter().filter(|row| row.coverage_status != "complete").cloned().collect::<Vec<_>>();

    Ok(BamEndogenousContentCompleteReport {
        schema_version: BAM_ENDOGENOUS_CONTENT_COMPLETE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        active_row_count: rows.len(),
        complete_row_count,
        incomplete_row_count: rows.len().saturating_sub(complete_row_count),
        checked_surface_count: CHECKED_SURFACE_COUNT,
        required_output_ids: rows[0].required_output_ids.clone(),
        required_summary_metric_names: rows[0].required_summary_metric_names.clone(),
        required_normalized_metric_names: rows[0].required_normalized_metric_names.clone(),
        optional_metric_names: rows[0].optional_metric_names.clone(),
        violation_count: violations.len(),
        ok: violations.is_empty(),
        rows,
        violations,
    })
}

fn build_endogenous_content_complete_row(
    repo_root: &Path,
    endogenous_row: BamOverlapEndogenousReadyRow,
) -> Result<BamEndogenousContentCompleteRow> {
    let local_smoke_report_path = repo_root.join(&endogenous_row.local_smoke_proof_path);
    let local_smoke_report: LocalEndogenousContentSmokeReport = serde_json::from_str(
        &fs::read_to_string(&local_smoke_report_path)
            .with_context(|| format!("read {}", local_smoke_report_path.display()))?,
    )
    .with_context(|| format!("parse {}", local_smoke_report_path.display()))?;

    let summary_path = repo_root.join(&local_smoke_report.endogenous_summary);
    let summary: bijux_dna_domain_bam::BamEndogenousContentEstimateV1 = serde_json::from_str(
        &fs::read_to_string(&summary_path)
            .with_context(|| format!("read {}", summary_path.display()))?,
    )
    .with_context(|| format!("parse {}", summary_path.display()))?;

    let stage_metrics_path = repo_root.join(&local_smoke_report.stage_metrics);
    let normalized_metrics: LocalEndogenousContentSmokeMetrics = serde_json::from_str(
        &fs::read_to_string(&stage_metrics_path)
            .with_context(|| format!("read {}", stage_metrics_path.display()))?,
    )
    .with_context(|| format!("parse {}", stage_metrics_path.display()))?;

    let summary_ready = summary.schema_version == EXPECTED_SUMMARY_SCHEMA_VERSION
        && summary.stage_id == EXPECTED_STAGE_ID
        && summary.method == EXPECTED_METHOD
        && summary.mapped_reads == EXPECTED_MAPPED_READS
        && summary.endogenous_reads == EXPECTED_ENDOGENOUS_READS
        && summary.total_reads == EXPECTED_TOTAL_READS
        && float_matches(summary.endogenous_fraction, EXPECTED_ENDOGENOUS_FRACTION)
        && summary.prealignment_fraction.is_none()
        && float_matches(summary.postalignment_fraction, EXPECTED_ENDOGENOUS_FRACTION)
        && summary.host_reference_scope.as_deref() == Some(EXPECTED_HOST_REFERENCE_SCOPE)
        && !summary.caveats.is_empty();

    let normalized_metrics_ready = normalized_metrics.schema_version
        == EXPECTED_STAGE_METRICS_SCHEMA_VERSION
        && normalized_metrics.stage_id == EXPECTED_STAGE_ID
        && normalized_metrics.sample_id == EXPECTED_SAMPLE_ID
        && normalized_metrics.expected_method == EXPECTED_METHOD
        && normalized_metrics.method == EXPECTED_METHOD
        && normalized_metrics.expected_host_reference_scope == EXPECTED_HOST_REFERENCE_SCOPE
        && normalized_metrics.host_reference_scope.as_deref()
            == Some(EXPECTED_HOST_REFERENCE_SCOPE)
        && normalized_metrics.expected_mapped_reads == EXPECTED_MAPPED_READS
        && normalized_metrics.mapped_reads == EXPECTED_MAPPED_READS
        && normalized_metrics.mapped_read_delta == 0
        && normalized_metrics.expected_endogenous_reads == EXPECTED_ENDOGENOUS_READS
        && normalized_metrics.endogenous_reads == EXPECTED_ENDOGENOUS_READS
        && normalized_metrics.expected_total_reads == EXPECTED_TOTAL_READS
        && normalized_metrics.total_reads == EXPECTED_TOTAL_READS
        && normalized_metrics.total_read_delta == 0
        && float_matches(
            normalized_metrics.expected_endogenous_fraction,
            EXPECTED_ENDOGENOUS_FRACTION,
        )
        && float_matches(normalized_metrics.endogenous_fraction, EXPECTED_ENDOGENOUS_FRACTION)
        && float_matches(normalized_metrics.endogenous_fraction_delta, 0.0)
        && normalized_metrics.prealignment_fraction.is_none()
        && normalized_metrics.expectation_matched;

    let endogenous_metric_consistency_ready = local_smoke_report.schema_version
        == EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION
        && local_smoke_report.stage_id == EXPECTED_STAGE_ID
        && local_smoke_report.sample_id == EXPECTED_SAMPLE_ID
        && local_smoke_report.expectation_matched
        && local_smoke_report.method == EXPECTED_METHOD
        && local_smoke_report.host_reference_scope.as_deref()
            == Some(EXPECTED_HOST_REFERENCE_SCOPE)
        && local_smoke_report.mapped_reads == EXPECTED_MAPPED_READS
        && local_smoke_report.endogenous_reads == EXPECTED_ENDOGENOUS_READS
        && local_smoke_report.total_reads == EXPECTED_TOTAL_READS
        && float_matches(local_smoke_report.endogenous_fraction, EXPECTED_ENDOGENOUS_FRACTION)
        && local_smoke_report.prealignment_fraction.is_none()
        && summary.method == local_smoke_report.method
        && summary.host_reference_scope == local_smoke_report.host_reference_scope
        && summary.mapped_reads == local_smoke_report.mapped_reads
        && summary.endogenous_reads == local_smoke_report.endogenous_reads
        && summary.total_reads == local_smoke_report.total_reads
        && float_matches(summary.endogenous_fraction, local_smoke_report.endogenous_fraction)
        && normalized_metrics.method == local_smoke_report.method
        && normalized_metrics.host_reference_scope == local_smoke_report.host_reference_scope
        && normalized_metrics.mapped_reads == local_smoke_report.mapped_reads
        && normalized_metrics.endogenous_reads == local_smoke_report.endogenous_reads
        && normalized_metrics.total_reads == local_smoke_report.total_reads
        && float_matches(
            normalized_metrics.endogenous_fraction,
            local_smoke_report.endogenous_fraction,
        )
        && local_smoke_report.endogenous_report.as_str()
            == local_smoke_report
                .endogenous_summary
                .replace("endogenous.summary.json", "endogenous.content.json");

    let mut missing_surfaces = endogenous_row.missing_surfaces.clone();
    if !summary_ready {
        missing_surfaces.push("endogenous_summary_contract".to_string());
    }
    if !normalized_metrics_ready {
        missing_surfaces.push("normalized_endogenous_metrics".to_string());
    }
    if !endogenous_metric_consistency_ready {
        missing_surfaces.push("endogenous_metric_consistency".to_string());
    }

    let coverage_status =
        if endogenous_row.coverage_status == "complete" && missing_surfaces.is_empty() {
            "complete".to_string()
        } else {
            "incomplete".to_string()
        };
    let reason = if coverage_status == "complete" {
        "binding `bam.endogenous_content` / `samtools` keeps active scope, command, output, parser, expected-result, report, schema, local-smoke, summary, and normalized endogenous metrics complete".to_string()
    } else {
        format!(
            "binding `bam.endogenous_content` / `samtools` is missing readiness proof for {}",
            missing_surfaces.join(", ")
        )
    };

    Ok(BamEndogenousContentCompleteRow {
        result_id: endogenous_row.result_id,
        stage_id: endogenous_row.stage_id,
        tool_id: endogenous_row.tool_id,
        sample_scope: endogenous_row.sample_scope,
        benchmark_status: endogenous_row.benchmark_status,
        support_status: endogenous_row.support_status,
        adapter_status: endogenous_row.adapter_status,
        parser_status: endogenous_row.parser_status,
        corpus_status: endogenous_row.corpus_status,
        report_section_id: endogenous_row.report_section_id,
        summary_table_id: endogenous_row.summary_table_id,
        command_readiness_kind: endogenous_row.command_readiness_kind,
        required_output_ids: endogenous_row.required_output_ids,
        stage_output_ids: endogenous_row.stage_output_ids,
        expected_schema_extension_id: endogenous_row.expected_schema_extension_id,
        schema_extension_id: endogenous_row.schema_extension_id,
        required_schema_keys: endogenous_row.required_schema_keys,
        schema_required_keys: endogenous_row.schema_required_keys,
        required_local_smoke_fields: endogenous_row.required_local_smoke_fields,
        required_summary_metric_names: REQUIRED_SUMMARY_METRIC_NAMES
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        required_normalized_metric_names: REQUIRED_NORMALIZED_METRIC_NAMES
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        optional_metric_names: OPTIONAL_METRIC_NAMES
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        available_optional_metric_names: Vec::new(),
        active_scope_proof_path: endogenous_row.active_scope_proof_path,
        command_proof_path: endogenous_row.command_proof_path,
        output_contract_proof_path: endogenous_row.output_contract_proof_path,
        parser_proof_path: endogenous_row.parser_proof_path,
        expected_result_proof_path: endogenous_row.expected_result_proof_path,
        report_map_proof_path: endogenous_row.report_map_proof_path,
        schema_proof_path: endogenous_row.schema_proof_path,
        local_smoke_proof_path: endogenous_row.local_smoke_proof_path,
        endogenous_report_path: local_smoke_report.endogenous_report.clone(),
        endogenous_summary_path: local_smoke_report.endogenous_summary.clone(),
        stage_metrics_path: local_smoke_report.stage_metrics.clone(),
        local_smoke_schema_version: local_smoke_report.schema_version,
        local_smoke_sample_id: local_smoke_report.sample_id.clone(),
        local_smoke_method: local_smoke_report.method.clone(),
        local_smoke_input_bam: local_smoke_report.input_bam,
        local_smoke_expectation_matched: local_smoke_report.expectation_matched,
        local_smoke_host_reference_scope: local_smoke_report.host_reference_scope.clone(),
        local_smoke_mapped_reads: local_smoke_report.mapped_reads,
        local_smoke_endogenous_reads: local_smoke_report.endogenous_reads,
        local_smoke_total_reads: local_smoke_report.total_reads,
        local_smoke_endogenous_fraction: local_smoke_report.endogenous_fraction,
        local_smoke_prealignment_fraction: local_smoke_report.prealignment_fraction,
        summary_schema_version: summary.schema_version,
        summary_method: summary.method,
        summary_host_reference_scope: summary.host_reference_scope,
        summary_mapped_reads: summary.mapped_reads,
        summary_endogenous_reads: summary.endogenous_reads,
        summary_total_reads: summary.total_reads,
        summary_endogenous_fraction: summary.endogenous_fraction,
        summary_prealignment_fraction: summary.prealignment_fraction,
        summary_postalignment_fraction: summary.postalignment_fraction,
        summary_caveats: summary.caveats,
        contaminant_reads: None,
        normalized_metrics_schema_version: normalized_metrics.schema_version.clone(),
        normalized_metrics_sample_id: normalized_metrics.sample_id.clone(),
        normalized_metrics,
        active_scope_ready: endogenous_row.active_scope_ready,
        command_ready: endogenous_row.command_ready,
        output_ready: endogenous_row.output_ready,
        parser_ready: endogenous_row.parser_ready,
        expected_result_ready: endogenous_row.expected_result_ready,
        report_ready: endogenous_row.report_ready,
        schema_ready: endogenous_row.schema_ready,
        local_smoke_ready: endogenous_row.local_smoke_ready,
        summary_ready,
        normalized_metrics_ready,
        endogenous_metric_consistency_ready,
        coverage_status,
        missing_surfaces,
        reason,
    })
}

fn float_matches(left: f64, right: f64) -> bool {
    (left - right).abs() <= 1e-9
}

fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_bam_endogenous_content_complete, BAM_ENDOGENOUS_CONTENT_COMPLETE_SCHEMA_VERSION,
        DEFAULT_BAM_ENDOGENOUS_CONTENT_COMPLETE_PATH,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn render_bam_endogenous_content_complete_reports_governed_metrics() {
        let root = repo_root();
        let report = render_bam_endogenous_content_complete(
            &root,
            PathBuf::from(DEFAULT_BAM_ENDOGENOUS_CONTENT_COMPLETE_PATH),
        )
        .expect("render BAM endogenous-content completion report");

        assert_eq!(report.schema_version, BAM_ENDOGENOUS_CONTENT_COMPLETE_SCHEMA_VERSION);
        assert_eq!(report.active_row_count, 1);
        assert_eq!(report.complete_row_count, 1);
        assert_eq!(report.incomplete_row_count, 0);
        assert_eq!(report.checked_surface_count, 11);
        assert_eq!(report.violation_count, 0);
        assert!(report.ok);

        let row = report.rows.first().expect("endogenous-content row");
        assert_eq!(row.stage_id, "bam.endogenous_content");
        assert_eq!(row.tool_id, "samtools");
        assert_eq!(row.summary_schema_version, "bijux.bam.endogenous_content.v1");
        assert_eq!(
            row.normalized_metrics_schema_version,
            "bijux.bam.endogenous_content.local_smoke.metrics.v1"
        );
        assert_eq!(row.summary_mapped_reads, 3);
        assert_eq!(row.summary_endogenous_reads, 3);
        assert_eq!(row.summary_total_reads, 5);
        assert!((row.summary_endogenous_fraction - 0.6).abs() <= 1e-9);
        assert!(row.local_smoke_expectation_matched);
        assert!(row.summary_ready);
        assert!(row.normalized_metrics_ready);
        assert!(row.endogenous_metric_consistency_ready);
        assert_eq!(row.coverage_status, "complete");
        assert_eq!(row.contaminant_reads, None);
    }
}
