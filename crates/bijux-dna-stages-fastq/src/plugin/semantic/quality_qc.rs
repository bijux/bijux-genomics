use super::validation_semantics::{parse_qc_contributor_identity, validate_semantic_metrics};
use std::fs;

use bijux_dna_stage_contract::{ArtifactRef, StagePlanV1};

use crate::observer::{parse_multiqc_general_stats_metrics, parse_report_qc_report};

pub(super) fn observed_quality_qc_metrics(
    plan: &StagePlanV1,
    artifacts: &[ArtifactRef],
) -> Option<serde_json::Value> {
    match plan.stage_id.as_str() {
        "fastq.report_qc" => report_qc_metrics(artifacts),
        "fastq.validate_reads" => validate_semantic_metrics(artifacts),
        _ => None,
    }
}

fn report_qc_metrics(artifacts: &[ArtifactRef]) -> Option<serde_json::Value> {
    let multiqc_metrics = multiqc_metrics(artifacts);
    report_qc_report_metrics(artifacts, multiqc_metrics.as_ref())
        .or_else(|| report_qc_manifest_metrics(artifacts, multiqc_metrics.as_ref()))
}

fn multiqc_metrics(
    artifacts: &[ArtifactRef],
) -> Option<bijux_dna_domain_fastq::metrics::MultiqcToolMetricsV1> {
    artifacts
        .iter()
        .find(|artifact| artifact.name.as_str() == "multiqc_data")
        .map(|artifact| artifact.path.join("multiqc_general_stats.json"))
        .filter(|path| path.exists())
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|raw| parse_multiqc_general_stats_metrics(&raw).ok())
}

fn report_qc_report_metrics(
    artifacts: &[ArtifactRef],
    multiqc_metrics: Option<&bijux_dna_domain_fastq::metrics::MultiqcToolMetricsV1>,
) -> Option<serde_json::Value> {
    let report = artifacts
        .iter()
        .find(|artifact| artifact.name.as_str() == "report_json")
        .and_then(|artifact| fs::read_to_string(&artifact.path).ok())
        .and_then(|raw| parse_report_qc_report(&raw).ok())?;
    Some(serde_json::json!({
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
        "multiqc_sample_count": report
            .multiqc_sample_count
            .or_else(|| multiqc_metrics.map(|metrics| metrics.sample_count)),
        "multiqc_module_count": report
            .multiqc_module_count
            .or_else(|| multiqc_metrics.map(|metrics| metrics.module_count)),
    }))
}

fn report_qc_manifest_metrics(
    artifacts: &[ArtifactRef],
    multiqc_metrics: Option<&bijux_dna_domain_fastq::metrics::MultiqcToolMetricsV1>,
) -> Option<serde_json::Value> {
    let manifest = artifacts
        .iter()
        .find(|artifact| artifact.name.as_str() == "governed_qc_inputs_manifest")
        .and_then(|artifact| fs::read_to_string(&artifact.path).ok())
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())?;
    let contributor_entries = manifest
        .get("contributors")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let contributor_stage_ids = contributor_stage_ids(&manifest, &contributor_entries);
    let contributor_tool_ids = contributor_tool_ids(&manifest, &contributor_entries);
    let contributor_count = contributor_count(&manifest, &contributor_entries);
    Some(serde_json::json!({
        "lineage_hash": manifest.get("lineage_hash").cloned().unwrap_or(serde_json::Value::Null),
        "contributor_artifact_count": contributor_count,
        "contributor_stage_ids": contributor_stage_ids,
        "contributor_tool_ids": contributor_tool_ids,
        "raw_fastqc_dir": manifest.get("raw_fastqc_dir").cloned().unwrap_or(serde_json::Value::Null),
        "multiqc_sample_count": multiqc_metrics.map(|metrics| metrics.sample_count),
        "multiqc_module_count": multiqc_metrics.map(|metrics| metrics.module_count),
    }))
}

fn contributor_stage_ids(
    manifest: &serde_json::Value,
    contributor_entries: &[serde_json::Value],
) -> Vec<String> {
    let mut ids = if contributor_entries.is_empty() {
        contributor_identities_from_legacy_inputs(manifest, |stage_id, _tool_id| stage_id)
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
    ids.sort();
    ids.dedup();
    ids
}

fn contributor_tool_ids(
    manifest: &serde_json::Value,
    contributor_entries: &[serde_json::Value],
) -> Vec<String> {
    let mut ids = if contributor_entries.is_empty() {
        contributor_identities_from_legacy_inputs(manifest, |_stage_id, tool_id| tool_id)
    } else {
        contributor_entries
            .iter()
            .filter_map(|entry| {
                entry
                    .get("contributor_id")
                    .and_then(serde_json::Value::as_str)
                    .and_then(|contributor_id| {
                        contributor_id.rsplit_once('.').map(|(_, tool_id)| tool_id.to_string())
                    })
            })
            .collect::<Vec<_>>()
    };
    ids.sort();
    ids.dedup();
    ids
}

fn contributor_identities_from_legacy_inputs(
    manifest: &serde_json::Value,
    select: impl Fn(String, String) -> String,
) -> Vec<String> {
    manifest
        .get("qc_inputs")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.get("name").and_then(serde_json::Value::as_str))
        .filter_map(parse_qc_contributor_identity)
        .map(|(stage_id, tool_id)| select(stage_id, tool_id))
        .collect()
}

fn contributor_count(
    manifest: &serde_json::Value,
    contributor_entries: &[serde_json::Value],
) -> usize {
    if contributor_entries.is_empty() {
        manifest.get("qc_inputs").and_then(serde_json::Value::as_array).map_or(0, Vec::len)
    } else {
        contributor_entries.len()
    }
}
