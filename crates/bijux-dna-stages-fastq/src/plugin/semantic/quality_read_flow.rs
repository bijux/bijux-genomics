use std::fs;

use bijux_dna_stage_contract::{ArtifactRef, StagePlanV1};

use crate::observer::{parse_merge_pairs_report, parse_normalize_primers_report};

pub(super) fn observed_quality_read_flow_metrics(
    plan: &StagePlanV1,
    artifacts: &[ArtifactRef],
) -> Option<serde_json::Value> {
    if plan.stage_id.as_str() == "fastq.merge_pairs" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_merge_pairs_report(&raw_report) {
                    return Some(serde_json::json!({
                        "paired_mode": report.paired_mode,
                        "merge_engine": report.merge_engine,
                        "threads": report.threads,
                        "merge_overlap": report.merge_overlap,
                        "min_length": report.min_len,
                        "unmerged_read_policy": report.unmerged_read_policy,
                        "reads_r1": report.reads_r1,
                        "reads_r2": report.reads_r2,
                        "reads_merged": report.reads_merged,
                        "reads_unmerged": report.reads_unmerged,
                        "merge_rate": report.merge_rate,
                        "merged_reads": report.merged_reads,
                        "unmerged_reads_r1": report.unmerged_reads_r1,
                        "unmerged_reads_r2": report.unmerged_reads_r2,
                        "raw_backend_report": report.raw_backend_report,
                        "raw_backend_report_format": report.raw_backend_report_format,
                    }));
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.normalize_primers" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_normalize_primers_report(&raw_report) {
                    return Some(serde_json::json!({
                        "paired_mode": report.paired_mode,
                        "primer_set_id": report.primer_set_id,
                        "marker_id": report.marker_id,
                        "orientation_policy": report.orientation_policy,
                        "max_mismatch_rate": report.max_mismatch_rate,
                        "min_overlap_bp": report.min_overlap_bp,
                        "reads_in": report.reads_in,
                        "reads_out": report.reads_out,
                        "primer_trimmed_reads": report.primer_trimmed_reads,
                        "primer_trimmed_fraction": report.primer_trimmed_fraction,
                        "orientation_forward_fraction": report.orientation_forward_fraction,
                        "primer_orientation_report": report.primer_orientation_report,
                        "primer_stats_json": report.primer_stats_json,
                        "raw_backend_report": report.raw_backend_report,
                        "raw_backend_report_format": report.raw_backend_report_format,
                    }));
                }
            }
        }
    }
    None
}
