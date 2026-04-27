use std::fs;

use bijux_dna_stage_contract::{ArtifactRef, StagePlanV1};

use crate::observer::{parse_detect_adapters_report, parse_extract_umis_report};

pub(super) fn observed_read_preparation_metrics(
    plan: &StagePlanV1,
    artifacts: &[ArtifactRef],
) -> Option<serde_json::Value> {
    if plan.stage_id.as_str() == "fastq.detect_adapters" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_detect_adapters_report(&raw_report) {
                    return Some(serde_json::json!({
                        "paired_mode": report.paired_mode,
                        "threads": report.threads,
                        "inspection_mode": report.inspection_mode,
                        "report_only": report.report_only,
                        "evidence_engine": report.evidence_engine,
                        "evidence_scope": report.evidence_scope,
                        "evidence_format": report.evidence_format,
                        "candidate_adapter_count": report.candidate_adapter_count,
                        "adapter_trimmed_fraction": report.adapter_trimmed_fraction,
                        "adapter_content_max": report.adapter_content_max,
                        "adapter_content_mean": report.adapter_content_mean,
                        "duplication_rate": report.duplication_rate,
                        "n_rate": report.n_rate,
                        "kmer_warning_count": report.kmer_warning_count,
                        "overrepresented_sequence_count": report.overrepresented_sequence_count,
                        "adapter_evidence_dir": report.adapter_evidence_dir,
                    }));
                }
            }
        }
    }
    if plan.stage_id.as_str() == "fastq.extract_umis" {
        if let Some(report_path) = artifacts
            .iter()
            .find(|artifact| artifact.name.as_str() == "report_json")
            .map(|artifact| artifact.path.as_path())
        {
            if let Ok(raw_report) = fs::read_to_string(report_path) {
                if let Ok(report) = parse_extract_umis_report(&raw_report) {
                    let mut semantics = serde_json::Map::from_iter([
                        ("paired_mode".to_string(), serde_json::json!(report.paired_mode)),
                        ("threads".to_string(), serde_json::json!(report.threads)),
                        ("umi_pattern".to_string(), serde_json::json!(report.umi_pattern)),
                        ("reads_in".to_string(), serde_json::json!(report.reads_in)),
                        ("reads_out".to_string(), serde_json::json!(report.reads_out)),
                        ("bases_in".to_string(), serde_json::json!(report.bases_in)),
                        ("bases_out".to_string(), serde_json::json!(report.bases_out)),
                        ("pairs_in".to_string(), serde_json::json!(report.pairs_in)),
                        ("pairs_out".to_string(), serde_json::json!(report.pairs_out)),
                        ("reads_with_umi".to_string(), serde_json::json!(report.reads_with_umi)),
                        ("mean_q_before".to_string(), serde_json::json!(report.mean_q_before)),
                        ("mean_q_after".to_string(), serde_json::json!(report.mean_q_after)),
                        (
                            "raw_backend_report_format".to_string(),
                            serde_json::json!(report.raw_backend_report_format),
                        ),
                    ]);
                    if let Some(backend_metrics) =
                        report.backend_metrics.as_ref().and_then(serde_json::Value::as_object)
                    {
                        for (metric_name, metric_value) in backend_metrics {
                            semantics.insert(metric_name.clone(), metric_value.clone());
                        }
                    }
                    return Some(serde_json::Value::Object(semantics));
                }
            }
        }
    }
    None
}
