use std::fs;

use bijux_dna_stage_contract::{ArtifactRef, StagePlanV1};

use crate::observer::{
    parse_profile_overrepresented_report, parse_profile_read_lengths_report,
    parse_profile_reads_report,
};

pub(super) fn observed_profiling_metrics(
    plan: &StagePlanV1,
    artifacts: &[ArtifactRef],
) -> Option<serde_json::Value> {
    if plan.stage_id.as_str() == "fastq.profile_reads" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "qc_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_profile_reads_report(&raw_report) {
                    return Some(serde_json::json!({
                        "paired_mode": report.paired_mode,
                        "threads": report.threads,
                        "reads_total": report.reads_total,
                        "bases_total": report.bases_total,
                        "mean_q": report.mean_q,
                        "gc_percent": report.gc_percent,
                        "length_histogram_source": report.length_histogram_source,
                        "length_histogram_bins": report.length_histogram.len(),
                        "mate_summary_count": report.mate_summaries.len(),
                        "mate_summaries": report.mate_summaries,
                        "qc_tsv": report.qc_tsv,
                        "qc_plots_dir": report.qc_plots_dir,
                        "raw_backend_report": report.raw_backend_report,
                        "raw_backend_report_format": report.raw_backend_report_format,
                    }));
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.profile_read_lengths" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_profile_read_lengths_report(&raw_report) {
                    return Some(serde_json::json!({
                        "paired_mode": report.paired_mode,
                        "threads": report.threads,
                        "histogram_bins": report.histogram_bins,
                        "read_count": report.read_count,
                        "mean_read_length": report.mean_read_length,
                        "max_read_length": report.max_read_length,
                        "distinct_lengths": report.distinct_lengths,
                        "histogram_entry_count": report.histogram.len(),
                        "length_distribution_tsv": report.length_distribution_tsv,
                        "length_distribution_json": report.length_distribution_json,
                        "raw_backend_report": report.raw_backend_report,
                        "raw_backend_report_format": report.raw_backend_report_format,
                    }));
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.profile_overrepresented_sequences" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_profile_overrepresented_report(&raw_report) {
                    return Some(serde_json::json!({
                        "paired_mode": report.paired_mode,
                        "threads": report.threads,
                        "top_k": report.top_k,
                        "sequence_count": report.sequence_count,
                        "flagged_sequences": report.flagged_sequences,
                        "top_fraction": report.top_fraction,
                        "row_count": report.rows.len(),
                        "overrepresented_sequences_tsv": report.overrepresented_sequences_tsv,
                        "overrepresented_sequences_json": report.overrepresented_sequences_json,
                        "raw_backend_report": report.raw_backend_report,
                        "raw_backend_report_format": report.raw_backend_report_format,
                    }));
                }
            }
        }
    }
    None
}
