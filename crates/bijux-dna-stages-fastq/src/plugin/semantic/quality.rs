use super::support::{parse_qc_contributor_identity, validate_semantic_metrics};
use super::*;

pub(super) fn observed_quality_metrics(
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
    if plan.stage_id.as_str() == "fastq.report_qc" {
        let multiqc_metrics = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "multiqc_data")
            .map(|artifact| artifact.path.join("multiqc_general_stats.json"))
            .filter(|path| path.exists())
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|raw| parse_multiqc_general_stats_metrics(&raw).ok());
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_report_qc_report(&raw_report) {
                    return Some(serde_json::json!({
                        "aggregation_engine": report.aggregation_engine,
                        "aggregation_scope": report.aggregation_scope,
                        "paired_mode": report.paired_mode,
                        "lineage_hash": report.governed_qc_lineage_hash,
                        "contributor_artifact_count": report.governed_qc_input_count,
                        "contributor_stage_ids": report.governed_qc_contributor_stage_ids,
                        "contributor_tool_ids": report.governed_qc_contributor_tool_ids,
                        "raw_fastqc_dir": report.raw_fastqc_dir,
                        "trimmed_fastqc_dir": report.trimmed_fastqc_dir,
                        "multiqc_report": report.multiqc_report,
                        "multiqc_data": report.multiqc_data,
                        "multiqc_sample_count": report.multiqc_sample_count.or_else(|| multiqc_metrics.as_ref().map(|metrics| metrics.sample_count)),
                        "multiqc_module_count": report.multiqc_module_count.or_else(|| multiqc_metrics.as_ref().map(|metrics| metrics.module_count)),
                    }));
                }
            }
        }
        if let Some(manifest_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "governed_qc_inputs_manifest")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_manifest) = fs::read_to_string(manifest_path) {
                if let Ok(manifest) = serde_json::from_str::<serde_json::Value>(&raw_manifest) {
                    let contributor_entries = manifest
                        .get("contributors")
                        .and_then(serde_json::Value::as_array)
                        .cloned()
                        .unwrap_or_default();
                    let mut contributor_stage_ids = if contributor_entries.is_empty() {
                        manifest
                            .get("qc_inputs")
                            .and_then(serde_json::Value::as_array)
                            .into_iter()
                            .flatten()
                            .filter_map(|entry| {
                                entry.get("name").and_then(serde_json::Value::as_str)
                            })
                            .filter_map(parse_qc_contributor_identity)
                            .map(|(stage_id, _tool_id)| stage_id)
                            .collect::<Vec<_>>()
                    } else {
                        contributor_entries
                            .iter()
                            .filter_map(|entry| {
                                entry
                                    .get("stage_id")
                                    .and_then(serde_json::Value::as_str)
                                    .map(ToString::to_string)
                            })
                            .collect::<Vec<_>>()
                    };
                    contributor_stage_ids.sort();
                    contributor_stage_ids.dedup();
                    let mut contributor_tool_ids = if contributor_entries.is_empty() {
                        manifest
                            .get("qc_inputs")
                            .and_then(serde_json::Value::as_array)
                            .into_iter()
                            .flatten()
                            .filter_map(|entry| {
                                entry.get("name").and_then(serde_json::Value::as_str)
                            })
                            .filter_map(parse_qc_contributor_identity)
                            .map(|(_stage_id, tool_id)| tool_id)
                            .collect::<Vec<_>>()
                    } else {
                        contributor_entries
                            .iter()
                            .filter_map(|entry| {
                                entry
                                    .get("contributor_id")
                                    .and_then(serde_json::Value::as_str)
                                    .and_then(|contributor_id| {
                                        contributor_id
                                            .rsplit_once('.')
                                            .map(|(_, tool_id)| tool_id.to_string())
                                    })
                            })
                            .collect::<Vec<_>>()
                    };
                    contributor_tool_ids.sort();
                    contributor_tool_ids.dedup();
                    let contributor_count = if contributor_entries.is_empty() {
                        manifest
                            .get("qc_inputs")
                            .and_then(serde_json::Value::as_array)
                            .map_or(0, std::vec::Vec::len)
                    } else {
                        contributor_entries.len()
                    };
                    return Some(serde_json::json!({
                        "lineage_hash": manifest.get("lineage_hash").cloned().unwrap_or(serde_json::Value::Null),
                        "contributor_artifact_count": contributor_count,
                        "contributor_stage_ids": contributor_stage_ids,
                        "contributor_tool_ids": contributor_tool_ids,
                        "raw_fastqc_dir": manifest.get("raw_fastqc_dir").cloned().unwrap_or(serde_json::Value::Null),
                        "multiqc_sample_count": multiqc_metrics.as_ref().map(|metrics| metrics.sample_count),
                        "multiqc_module_count": multiqc_metrics.as_ref().map(|metrics| metrics.module_count),
                    }));
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.validate_reads" {
        if let Some(semantics) = validate_semantic_metrics(artifacts) {
            return Some(semantics);
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
