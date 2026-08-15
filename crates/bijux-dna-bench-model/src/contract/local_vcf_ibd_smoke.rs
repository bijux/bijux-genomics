//! Owner: bijux-dna-bench-model
//! Semantic admission for persisted local VCF identity-by-descent smoke evidence.

use std::collections::BTreeSet;

use anyhow::{bail, Result};

use crate::model::local_vcf_ibd_smoke::{
    BenchmarkLocalVcfIbdSmokeReport, BENCHMARK_LOCAL_VCF_IBD_SMOKE_SCHEMA_V1,
};

const STAGE_ID: &str = "vcf.ibd";
const COMPLETE: &str = "complete";

/// Validates the semantic closure of a local VCF identity-by-descent smoke record.
///
/// # Errors
///
/// Returns an error when identities, paths, timing, pair summaries, backend state, or the
/// insufficient-overlap downstream probe do not satisfy the persisted contract.
pub fn validate_benchmark_local_vcf_ibd_smoke_report(
    report: &BenchmarkLocalVcfIbdSmokeReport,
) -> Result<()> {
    if report.schema_version != BENCHMARK_LOCAL_VCF_IBD_SMOKE_SCHEMA_V1 {
        bail!("unsupported local VCF IBD smoke schema");
    }
    if report.stage_id != STAGE_ID {
        bail!("local VCF IBD report has the wrong stage identity");
    }
    if !matches!(report.tool_id.as_str(), "germline" | "ibdseq" | "ibdhap") {
        bail!("local VCF IBD report has an unsupported tool identity");
    }
    for (name, value) in [
        ("command", report.command.as_str()),
        ("corpus_id", report.corpus_id.as_str()),
        ("input_fixture_id", report.input_fixture_id.as_str()),
        ("fixture_manifest_path", report.fixture_manifest_path.as_str()),
        ("input_vcf_path", report.input_vcf_path.as_str()),
        ("sample_metadata_path", report.sample_metadata_path.as_str()),
        ("output_root", report.output_root.as_str()),
        ("ibd_tsv_path", report.ibd_tsv_path.as_str()),
        ("ibd_json_path", report.ibd_json_path.as_str()),
        ("source_ibd_input_path", report.source_ibd_input_path.as_str()),
        ("source_ibd_segments_path", report.source_ibd_segments_path.as_str()),
        ("source_ibd_merged_segments_path", report.source_ibd_merged_segments_path.as_str()),
        ("source_ibd_filtered_segments_path", report.source_ibd_filtered_segments_path.as_str()),
        ("source_ibd_summary_path", report.source_ibd_summary_path.as_str()),
        ("source_ibd_metrics_path", report.source_ibd_metrics_path.as_str()),
        ("source_logs_path", report.source_logs_path.as_str()),
        ("stage_result_manifest_path", report.stage_result_manifest_path.as_str()),
        ("started_at", report.started_at.as_str()),
        ("finished_at", report.finished_at.as_str()),
    ] {
        if value.trim().is_empty() {
            bail!("local VCF IBD report has empty {name}");
        }
    }
    if !report.command.ends_with(&format!("--tool-id {}", report.tool_id)) {
        bail!("local VCF IBD command does not bind the reported tool");
    }
    if !report.elapsed_seconds.is_finite() || report.elapsed_seconds < 0.0 {
        bail!("local VCF IBD report has invalid elapsed time");
    }
    if report.exit_code != 0 || report.status != COMPLETE {
        bail!("successful local VCF IBD evidence requires complete status and exit code zero");
    }
    if !matches!(report.execution_mode.as_str(), "fallback_proxy" | "real_tool") {
        bail!("local VCF IBD report has an unsupported execution mode");
    }
    let selected_tool_ok = match report.tool_id.as_str() {
        "germline" => report.germline_tool_ok,
        "ibdseq" => report.ibdseq_tool_ok,
        "ibdhap" => report.ibdhap_tool_ok,
        _ => false,
    };
    if report.execution_mode == "real_tool" && !selected_tool_ok {
        bail!("real-tool IBD execution requires a successful selected backend");
    }
    let row_count = u64::try_from(report.rows.len())?;
    if report.pair_count == 0 || report.pair_count != row_count {
        bail!("local VCF IBD pair count disagrees with its row closure");
    }
    let mut pairs = BTreeSet::new();
    for row in &report.rows {
        if row.sample_a.trim().is_empty()
            || row.sample_b.trim().is_empty()
            || row.sample_a >= row.sample_b
        {
            bail!("local VCF IBD row has a non-canonical sample pair");
        }
        if !pairs.insert((row.sample_a.as_str(), row.sample_b.as_str())) {
            bail!("local VCF IBD report repeats a sample pair");
        }
        if row.segment_count == 0
            || row.overlap_marker_count == 0
            || !row.total_length.is_finite()
            || row.total_length <= 0.0
            || row.status != COMPLETE
        {
            bail!("local VCF IBD row has invalid complete-evidence metrics");
        }
    }
    validate_insufficient_overlap_probe(report)?;
    Ok(())
}

fn validate_insufficient_overlap_probe(report: &BenchmarkLocalVcfIbdSmokeReport) -> Result<()> {
    let probe = &report.insufficient_overlap_probe;
    for (name, value) in [
        ("probe input_vcf_path", probe.input_vcf_path.as_str()),
        ("probe source_ibd_summary_path", probe.source_ibd_summary_path.as_str()),
        (
            "probe source_ibd_filtered_segments_path",
            probe.source_ibd_filtered_segments_path.as_str(),
        ),
        ("probe source_roh_summary_path", probe.source_roh_summary_path.as_str()),
        ("probe source_roh_segments_path", probe.source_roh_segments_path.as_str()),
    ] {
        if value.trim().is_empty() {
            bail!("local VCF IBD report has empty {name}");
        }
    }
    if probe.ibd_status != "insufficient_marker_overlap"
        || probe.insufficient_data_reason != "no_pairs_met_min_marker_or_length_threshold"
        || probe.overlap_marker_count == 0
        || probe.filtered_segment_count != 0
    {
        bail!("local VCF IBD insufficient-overlap probe drifted");
    }
    if probe.unrelated_stage_id != "vcf.roh"
        || probe.unrelated_stage_status != COMPLETE
        || probe.unrelated_stage_segment_count == 0
    {
        bail!("local VCF IBD probe did not localize failure away from the unrelated stage");
    }
    Ok(())
}
