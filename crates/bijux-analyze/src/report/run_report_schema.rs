use std::collections::BTreeMap;

use bijux_core::{MetricSemanticsV1, ReportCompletenessV1, ReportContractV1, ReportSchemaV1};

use crate::decision::effect::default_thresholds;

pub(super) fn report_contract() -> ReportContractV1 {
    ReportContractV1 {
        schema_version: "bijux.report_contract.v1".to_string(),
        required_sections: vec![
            "contract".to_string(),
            "completeness".to_string(),
            "stages".to_string(),
            "provenance".to_string(),
            "retention_definition".to_string(),
            "retention_context".to_string(),
            "assets_provenance".to_string(),
            "metric_semantics".to_string(),
            "telemetry".to_string(),
            "qc_improvement".to_string(),
            "final_qc_summary".to_string(),
            "filter_interpretation".to_string(),
            "adapter_inference".to_string(),
        ],
        required_provenance_fields: vec![
            "tool_id".to_string(),
            "tool_version".to_string(),
            "image_digest".to_string(),
            "trace_id".to_string(),
            "span_id".to_string(),
            "params_hash".to_string(),
            "bank_hashes".to_string(),
        ],
    }
}

pub(super) fn build_report_sections(
    report: &ReportSchemaV1,
) -> BTreeMap<String, serde_json::Value> {
    let mut sections = BTreeMap::new();
    sections.insert("qc".to_string(), report.qc_improvement.clone());
    sections.insert(
        "final_qc_summary".to_string(),
        report.final_qc_summary.clone(),
    );
    sections.insert(
        "trimming".to_string(),
        serde_json::json!({
            "retention_definition": report.retention_definition.clone(),
            "retention_context": report.retention_context.clone(),
        }),
    );
    sections.insert(
        "filtering".to_string(),
        report.filter_interpretation.clone(),
    );
    sections.insert(
        "contamination".to_string(),
        serde_json::json!({
            "assets": report.assets_provenance.clone(),
        }),
    );
    sections.insert(
        "retention".to_string(),
        serde_json::json!({
            "definitions": report.retention_definition.clone(),
            "contexts": report.retention_context.clone(),
        }),
    );
    sections.insert(
        "failures".to_string(),
        serde_json::json!({
            "completeness": report.completeness,
        }),
    );
    sections.insert(
        "method_assumptions".to_string(),
        method_assumptions_section(report),
    );
    sections.insert("stage_completeness".to_string(), serde_json::json!([]));
    sections.insert("failure_hints".to_string(), serde_json::json!([]));
    sections.insert("metric_provenance".to_string(), serde_json::json!({}));
    sections.insert("bench_summary".to_string(), serde_json::json!({}));
    sections.insert("pipeline_overview".to_string(), serde_json::json!({}));
    sections.insert("key_findings".to_string(), serde_json::json!([]));
    sections.insert("stage_plots".to_string(), serde_json::json!([]));
    sections.insert("reproducibility".to_string(), serde_json::json!({}));
    sections.insert(
        "data_contract_validation".to_string(),
        serde_json::json!({}),
    );
    sections.insert("qc_delta".to_string(), serde_json::json!({}));
    sections.insert("contaminant_summary".to_string(), serde_json::json!({}));
    sections
}

fn method_assumptions_section(report: &ReportSchemaV1) -> serde_json::Value {
    let thresholds = default_thresholds();
    let tools: Vec<serde_json::Value> = report
        .provenance
        .iter()
        .map(|entry| {
            serde_json::json!({
                "stage_id": entry.stage_id,
                "tool_id": entry.tool_id,
                "tool_version": entry.tool_version,
                "params_hash": entry.params_hash,
                "image_digest": entry.image_digest,
            })
        })
        .collect();
    let presets: Vec<serde_json::Value> = report
        .stages
        .iter()
        .map(|stage| {
            serde_json::json!({
                "stage_id": stage.stage_id,
                "tool_id": stage.tool_id,
                "tool_version": stage.tool_version,
                "params_hash": stage.params_hash,
                "effective_config_path": stage.effective_config_path,
            })
        })
        .collect();
    serde_json::json!({
        "compare_objective": "balanced_default",
        "effect_thresholds": {
            "absolute": thresholds.absolute,
            "relative": thresholds.relative,
        },
        "tools": tools,
        "presets": presets,
        "assumptions": [
            "compare_objective defaults to balanced when not specified",
            "effect_size uses absolute/relative thresholds for practical significance",
        ],
    })
}

pub(super) fn report_completeness(
    missing_metrics: &[String],
    missing_reports: &[String],
) -> ReportCompletenessV1 {
    let status = if missing_metrics.is_empty() && missing_reports.is_empty() {
        "complete"
    } else {
        "incomplete"
    };
    ReportCompletenessV1 {
        status: status.to_string(),
        missing_metrics: missing_metrics.to_vec(),
        missing_reports: missing_reports.to_vec(),
    }
}

pub(super) fn report_metric_semantics() -> Vec<MetricSemanticsV1> {
    let metric_ids = [
        "runtime_s",
        "memory_mb",
        "read_retention",
        "base_retention",
        "merge_rate",
        "error_reduction_proxy",
    ];
    metric_ids
        .iter()
        .filter_map(|metric_id| {
            bijux_core::metric_semantics(metric_id).map(|spec| MetricSemanticsV1 {
                metric_id: spec.metric_id.to_string(),
                direction: format!("{:?}", spec.direction),
                units: spec.units.to_string(),
                range: spec.range.to_string(),
                missing_data_policy: spec.missing_data_policy.to_string(),
            })
        })
        .collect()
}
