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

pub(crate) const DEFAULT_BAM_OVERLAP_CORRECTION_COMPLETE_PATH: &str =
    "benchmarks/readiness/bam/stages/bam.overlap_correction.complete.json";
const BAM_OVERLAP_CORRECTION_COMPLETE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_overlap_correction_complete.v1";
const EXPECTED_STAGE_ID: &str = "bam.overlap_correction";
const EXPECTED_TOOL_ID: &str = "bamutil";
const EXPECTED_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.overlap_correction.v1";
const EXPECTED_STAGE_METRICS_SCHEMA_VERSION: &str =
    "bijux.bam.overlap_correction.local_smoke.metrics.v1";
const EXPECTED_CORRECTED_PAIR_COUNT: u64 = 1;
const EXPECTED_CORRECTED_OVERLAP_BASES: u64 = 7;
const EXPECTED_PAIR_COUNT: u64 = 2;
const CHECKED_SURFACE_COUNT: usize = 11;
const REQUIRED_NORMALIZED_METRIC_NAMES: [&str; 11] = [
    "method",
    "expected_pair_count",
    "pair_count",
    "pair_count_delta",
    "expected_corrected_pairs",
    "corrected_pairs",
    "corrected_pair_delta",
    "expected_corrected_overlap_bases",
    "corrected_overlap_bases",
    "corrected_overlap_base_delta",
    "expectation_matched",
];

#[derive(Debug, Clone, Deserialize)]
struct LocalOverlapCorrectionSmokeReport {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    expectation_matched: bool,
    input_bam: String,
    overlap_corrected_bam: String,
    method: String,
    pair_count: u64,
    corrected_pairs: u64,
    corrected_overlap_bases: u64,
    insufficiency_reason: Option<String>,
    overlap_correction_summary: String,
    flagstat_before: String,
    flagstat_after: String,
    idxstats_before: String,
    idxstats_after: String,
    stage_metrics: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LocalOverlapCorrectionSmokeMetrics {
    schema_version: String,
    stage_id: String,
    sample_id: String,
    method: String,
    expected_pair_count: u64,
    pair_count: u64,
    pair_count_delta: i64,
    expected_corrected_pairs: u64,
    corrected_pairs: u64,
    corrected_pair_delta: i64,
    expected_corrected_overlap_bases: u64,
    corrected_overlap_bases: u64,
    corrected_overlap_base_delta: i64,
    insufficiency_reason: Option<String>,
    expectation_matched: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamOverlapCorrectionCompleteRow {
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
    pub(crate) required_normalized_metric_names: Vec<String>,
    pub(crate) active_scope_proof_path: String,
    pub(crate) command_proof_path: String,
    pub(crate) output_contract_proof_path: String,
    pub(crate) parser_proof_path: String,
    pub(crate) expected_result_proof_path: String,
    pub(crate) report_map_proof_path: String,
    pub(crate) schema_proof_path: String,
    pub(crate) local_smoke_proof_path: String,
    pub(crate) overlap_corrected_bam_path: String,
    pub(crate) overlap_corrected_bai_path: String,
    pub(crate) overlap_correction_summary_path: String,
    pub(crate) flagstat_before_path: String,
    pub(crate) flagstat_after_path: String,
    pub(crate) idxstats_before_path: String,
    pub(crate) idxstats_after_path: String,
    pub(crate) stage_metrics_path: String,
    pub(crate) local_smoke_schema_version: String,
    pub(crate) local_smoke_sample_id: String,
    pub(crate) local_smoke_method: String,
    pub(crate) local_smoke_input_bam: String,
    pub(crate) local_smoke_expectation_matched: bool,
    pub(crate) summary_schema_version: String,
    pub(crate) summary_method: String,
    pub(crate) summary_input_bam: String,
    pub(crate) summary_output_bam: String,
    pub(crate) summary_pair_count: u64,
    pub(crate) summary_corrected_pairs: u64,
    pub(crate) summary_corrected_overlap_bases: u64,
    pub(crate) summary_flagstat_before_total_reads: u64,
    pub(crate) summary_flagstat_after_total_reads: u64,
    pub(crate) summary_flagstat_before_mapped_reads: u64,
    pub(crate) summary_flagstat_after_mapped_reads: u64,
    pub(crate) normalized_metrics_schema_version: String,
    pub(crate) normalized_metrics_sample_id: String,
    pub(crate) normalized_metrics: LocalOverlapCorrectionSmokeMetrics,
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
    pub(crate) corrected_overlap_metrics_ready: bool,
    pub(crate) coverage_status: String,
    pub(crate) missing_surfaces: Vec<String>,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamOverlapCorrectionCompleteReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) active_row_count: usize,
    pub(crate) complete_row_count: usize,
    pub(crate) incomplete_row_count: usize,
    pub(crate) checked_surface_count: usize,
    pub(crate) required_output_ids: Vec<String>,
    pub(crate) required_local_smoke_fields: Vec<String>,
    pub(crate) required_normalized_metric_names: Vec<String>,
    pub(crate) violation_count: usize,
    pub(crate) ok: bool,
    pub(crate) rows: Vec<BamOverlapCorrectionCompleteRow>,
    pub(crate) violations: Vec<BamOverlapCorrectionCompleteRow>,
}

pub(crate) fn run_render_bam_overlap_correction_complete(
    args: &parse::BenchReadinessRenderBamOverlapCorrectionCompleteArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_overlap_correction_complete(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_OVERLAP_CORRECTION_COMPLETE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_bam_overlap_correction_complete(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamOverlapCorrectionCompleteReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let report = build_bam_overlap_correction_complete_report(repo_root, &output_path)?;
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    if !report.ok {
        return Err(anyhow!(
            "bam.overlap_correction must keep active scope, command, output, parser, expected-result, report, schema, local-smoke, summary, and normalized corrected-overlap metrics complete"
        ));
    }
    Ok(report)
}

fn build_bam_overlap_correction_complete_report(
    repo_root: &Path,
    output_path: &Path,
) -> Result<BamOverlapCorrectionCompleteReport> {
    let overlap_endogenous_report = render_bam_overlap_endogenous_ready(
        repo_root,
        PathBuf::from(DEFAULT_BAM_OVERLAP_ENDOGENOUS_READY_PATH),
    )?;
    let overlap_row = overlap_endogenous_report
        .rows
        .into_iter()
        .find(|row| row.stage_id == EXPECTED_STAGE_ID && row.tool_id == EXPECTED_TOOL_ID)
        .ok_or_else(|| {
            anyhow!(
                "bam overlap/endogenous readiness report is missing `{EXPECTED_STAGE_ID}` / `{EXPECTED_TOOL_ID}`"
            )
        })?;
    let row = build_overlap_correction_complete_row(repo_root, overlap_row)?;
    let complete_row_count = usize::from(row.coverage_status == "complete");
    let rows = vec![row];
    let violations =
        rows.iter().filter(|row| row.coverage_status != "complete").cloned().collect::<Vec<_>>();

    Ok(BamOverlapCorrectionCompleteReport {
        schema_version: BAM_OVERLAP_CORRECTION_COMPLETE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        active_row_count: rows.len(),
        complete_row_count,
        incomplete_row_count: rows.len().saturating_sub(complete_row_count),
        checked_surface_count: CHECKED_SURFACE_COUNT,
        required_output_ids: rows[0].required_output_ids.clone(),
        required_local_smoke_fields: rows[0].required_local_smoke_fields.clone(),
        required_normalized_metric_names: rows[0].required_normalized_metric_names.clone(),
        violation_count: violations.len(),
        ok: violations.is_empty(),
        rows,
        violations,
    })
}

fn build_overlap_correction_complete_row(
    repo_root: &Path,
    overlap_row: BamOverlapEndogenousReadyRow,
) -> Result<BamOverlapCorrectionCompleteRow> {
    let local_smoke_report_path = repo_root.join(&overlap_row.local_smoke_proof_path);
    let local_smoke_report: LocalOverlapCorrectionSmokeReport = serde_json::from_str(
        &fs::read_to_string(&local_smoke_report_path)
            .with_context(|| format!("read {}", local_smoke_report_path.display()))?,
    )
    .with_context(|| format!("parse {}", local_smoke_report_path.display()))?;
    let summary_path = repo_root.join(&local_smoke_report.overlap_correction_summary);
    let summary: bijux_dna_domain_bam::BamOverlapCorrectionSummaryV1 = serde_json::from_str(
        &fs::read_to_string(&summary_path)
            .with_context(|| format!("read {}", summary_path.display()))?,
    )
    .with_context(|| format!("parse {}", summary_path.display()))?;
    let stage_metrics_path = repo_root.join(&local_smoke_report.stage_metrics);
    let normalized_metrics: LocalOverlapCorrectionSmokeMetrics = serde_json::from_str(
        &fs::read_to_string(&stage_metrics_path)
            .with_context(|| format!("read {}", stage_metrics_path.display()))?,
    )
    .with_context(|| format!("parse {}", stage_metrics_path.display()))?;

    let overlap_corrected_bai_path = overlap_row
        .local_smoke_artifact_paths
        .iter()
        .find(|path| path.ends_with("overlap_corrected.bam.bai"))
        .cloned()
        .ok_or_else(|| anyhow!("missing governed overlap-corrected BAM index path"))?;

    let summary_ready = summary.schema_version == EXPECTED_SUMMARY_SCHEMA_VERSION
        && summary.stage_id == EXPECTED_STAGE_ID
        && summary.method == EXPECTED_TOOL_ID
        && summary.pair_count == Some(EXPECTED_PAIR_COUNT)
        && summary.corrected_pairs == Some(EXPECTED_CORRECTED_PAIR_COUNT)
        && summary.corrected_overlap_bases == Some(EXPECTED_CORRECTED_OVERLAP_BASES)
        && summary.insufficiency_reason.is_none();
    let normalized_metrics_ready = normalized_metrics.schema_version
        == EXPECTED_STAGE_METRICS_SCHEMA_VERSION
        && normalized_metrics.stage_id == EXPECTED_STAGE_ID
        && normalized_metrics.sample_id == local_smoke_report.sample_id
        && normalized_metrics.method == EXPECTED_TOOL_ID
        && normalized_metrics.expected_pair_count == EXPECTED_PAIR_COUNT
        && normalized_metrics.pair_count == EXPECTED_PAIR_COUNT
        && normalized_metrics.pair_count_delta == 0
        && normalized_metrics.expected_corrected_pairs == EXPECTED_CORRECTED_PAIR_COUNT
        && normalized_metrics.corrected_pairs == EXPECTED_CORRECTED_PAIR_COUNT
        && normalized_metrics.corrected_pair_delta == 0
        && normalized_metrics.expected_corrected_overlap_bases == EXPECTED_CORRECTED_OVERLAP_BASES
        && normalized_metrics.corrected_overlap_bases == EXPECTED_CORRECTED_OVERLAP_BASES
        && normalized_metrics.corrected_overlap_base_delta == 0
        && normalized_metrics.expectation_matched
        && normalized_metrics.insufficiency_reason.is_none();
    let corrected_overlap_metrics_ready = local_smoke_report.expectation_matched
        && local_smoke_report.method == EXPECTED_TOOL_ID
        && local_smoke_report.pair_count == EXPECTED_PAIR_COUNT
        && local_smoke_report.corrected_pairs == EXPECTED_CORRECTED_PAIR_COUNT
        && local_smoke_report.corrected_overlap_bases == EXPECTED_CORRECTED_OVERLAP_BASES
        && local_smoke_report.insufficiency_reason.is_none()
        && summary.pair_count == Some(normalized_metrics.pair_count)
        && summary.corrected_pairs == Some(normalized_metrics.corrected_pairs)
        && summary.corrected_overlap_bases == Some(normalized_metrics.corrected_overlap_bases);

    let mut missing_surfaces = overlap_row.missing_surfaces.clone();
    if !summary_ready {
        missing_surfaces.push("overlap_summary_contract".to_string());
    }
    if !normalized_metrics_ready {
        missing_surfaces.push("normalized_corrected_overlap_metrics".to_string());
    }
    if !corrected_overlap_metrics_ready {
        missing_surfaces.push("corrected_overlap_metric_consistency".to_string());
    }

    let coverage_status =
        if overlap_row.coverage_status == "complete" && missing_surfaces.is_empty() {
            "complete".to_string()
        } else {
            "incomplete".to_string()
        };
    let reason = if coverage_status == "complete" {
        "binding `bam.overlap_correction` / `bamutil` keeps active scope, command, output, parser, expected-result, report, schema, local-smoke, summary, and normalized corrected-overlap metrics complete".to_string()
    } else {
        format!(
            "binding `bam.overlap_correction` / `bamutil` is missing readiness proof for {}",
            missing_surfaces.join(", ")
        )
    };

    Ok(BamOverlapCorrectionCompleteRow {
        result_id: overlap_row.result_id,
        stage_id: overlap_row.stage_id,
        tool_id: overlap_row.tool_id,
        sample_scope: overlap_row.sample_scope,
        benchmark_status: overlap_row.benchmark_status,
        support_status: overlap_row.support_status,
        adapter_status: overlap_row.adapter_status,
        parser_status: overlap_row.parser_status,
        corpus_status: overlap_row.corpus_status,
        report_section_id: overlap_row.report_section_id,
        summary_table_id: overlap_row.summary_table_id,
        command_readiness_kind: overlap_row.command_readiness_kind,
        required_output_ids: overlap_row.required_output_ids,
        stage_output_ids: overlap_row.stage_output_ids,
        expected_schema_extension_id: overlap_row.expected_schema_extension_id,
        schema_extension_id: overlap_row.schema_extension_id,
        required_schema_keys: overlap_row.required_schema_keys,
        schema_required_keys: overlap_row.schema_required_keys,
        required_local_smoke_fields: overlap_row.required_local_smoke_fields,
        required_normalized_metric_names: REQUIRED_NORMALIZED_METRIC_NAMES
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        active_scope_proof_path: overlap_row.active_scope_proof_path,
        command_proof_path: overlap_row.command_proof_path,
        output_contract_proof_path: overlap_row.output_contract_proof_path,
        parser_proof_path: overlap_row.parser_proof_path,
        expected_result_proof_path: overlap_row.expected_result_proof_path,
        report_map_proof_path: overlap_row.report_map_proof_path,
        schema_proof_path: overlap_row.schema_proof_path,
        local_smoke_proof_path: overlap_row.local_smoke_proof_path,
        overlap_corrected_bam_path: local_smoke_report.overlap_corrected_bam.clone(),
        overlap_corrected_bai_path,
        overlap_correction_summary_path: local_smoke_report.overlap_correction_summary.clone(),
        flagstat_before_path: local_smoke_report.flagstat_before.clone(),
        flagstat_after_path: local_smoke_report.flagstat_after.clone(),
        idxstats_before_path: local_smoke_report.idxstats_before.clone(),
        idxstats_after_path: local_smoke_report.idxstats_after.clone(),
        stage_metrics_path: local_smoke_report.stage_metrics.clone(),
        local_smoke_schema_version: local_smoke_report.schema_version,
        local_smoke_sample_id: local_smoke_report.sample_id.clone(),
        local_smoke_method: local_smoke_report.method.clone(),
        local_smoke_input_bam: local_smoke_report.input_bam,
        local_smoke_expectation_matched: local_smoke_report.expectation_matched,
        summary_schema_version: summary.schema_version,
        summary_method: summary.method,
        summary_input_bam: path_relative_to_repo(repo_root, &summary.input_bam),
        summary_output_bam: path_relative_to_repo(repo_root, &summary.output_bam),
        summary_pair_count: summary.pair_count.unwrap_or(0),
        summary_corrected_pairs: summary.corrected_pairs.unwrap_or(0),
        summary_corrected_overlap_bases: summary.corrected_overlap_bases.unwrap_or(0),
        summary_flagstat_before_total_reads: summary.flagstat_before.total_reads.unwrap_or(0),
        summary_flagstat_after_total_reads: summary.flagstat_after.total_reads.unwrap_or(0),
        summary_flagstat_before_mapped_reads: summary.flagstat_before.mapped_reads.unwrap_or(0),
        summary_flagstat_after_mapped_reads: summary.flagstat_after.mapped_reads.unwrap_or(0),
        normalized_metrics_schema_version: normalized_metrics.schema_version.clone(),
        normalized_metrics_sample_id: normalized_metrics.sample_id.clone(),
        normalized_metrics,
        active_scope_ready: overlap_row.active_scope_ready,
        command_ready: overlap_row.command_ready,
        output_ready: overlap_row.output_ready,
        parser_ready: overlap_row.parser_ready,
        expected_result_ready: overlap_row.expected_result_ready,
        report_ready: overlap_row.report_ready,
        schema_ready: overlap_row.schema_ready,
        local_smoke_ready: overlap_row.local_smoke_ready,
        summary_ready,
        normalized_metrics_ready,
        corrected_overlap_metrics_ready,
        coverage_status,
        missing_surfaces,
        reason,
    })
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
        render_bam_overlap_correction_complete, BAM_OVERLAP_CORRECTION_COMPLETE_SCHEMA_VERSION,
        DEFAULT_BAM_OVERLAP_CORRECTION_COMPLETE_PATH,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn render_bam_overlap_correction_complete_reports_governed_metrics() {
        let root = repo_root();
        let report = render_bam_overlap_correction_complete(
            &root,
            PathBuf::from(DEFAULT_BAM_OVERLAP_CORRECTION_COMPLETE_PATH),
        )
        .expect("render BAM overlap-correction completion report");

        assert_eq!(report.schema_version, BAM_OVERLAP_CORRECTION_COMPLETE_SCHEMA_VERSION);
        assert_eq!(report.active_row_count, 1);
        assert_eq!(report.complete_row_count, 1);
        assert_eq!(report.incomplete_row_count, 0);
        assert_eq!(report.checked_surface_count, 11);
        assert_eq!(report.violation_count, 0);
        assert!(report.ok);

        let row = report.rows.first().expect("overlap-correction row");
        assert_eq!(row.stage_id, "bam.overlap_correction");
        assert_eq!(row.tool_id, "bamutil");
        assert_eq!(row.summary_schema_version, "bijux.bam.overlap_correction.v1");
        assert_eq!(
            row.normalized_metrics_schema_version,
            "bijux.bam.overlap_correction.local_smoke.metrics.v1"
        );
        assert_eq!(row.summary_pair_count, 2);
        assert_eq!(row.summary_corrected_pairs, 1);
        assert_eq!(row.summary_corrected_overlap_bases, 7);
        assert_eq!(row.normalized_metrics.expected_pair_count, 2);
        assert_eq!(row.normalized_metrics.pair_count, 2);
        assert_eq!(row.normalized_metrics.expected_corrected_pairs, 1);
        assert_eq!(row.normalized_metrics.corrected_pairs, 1);
        assert_eq!(row.normalized_metrics.expected_corrected_overlap_bases, 7);
        assert_eq!(row.normalized_metrics.corrected_overlap_bases, 7);
        assert!(row.local_smoke_expectation_matched);
        assert!(row.summary_ready);
        assert!(row.normalized_metrics_ready);
        assert!(row.corrected_overlap_metrics_ready);
        assert_eq!(row.coverage_status, "complete");
    }
}
