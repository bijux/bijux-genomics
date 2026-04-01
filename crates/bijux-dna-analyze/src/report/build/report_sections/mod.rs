mod pipeline_overview;

pub(super) use pipeline_overview::pipeline_overview_section;

pub(super) fn key_findings_section(
    missing_metrics: &[String],
    missing_reports: &[String],
    rows: &[FactsRowV1],
    stage_confidence: &serde_json::Value,
) -> serde_json::Value {
    let mut findings = Vec::new();
    if !missing_metrics.is_empty() {
        findings.push(serde_json::json!({
            "kind": "missing_metrics",
            "count": missing_metrics.len(),
            "items": missing_metrics,
        }));
    }
    if !missing_reports.is_empty() {
        findings.push(serde_json::json!({
            "kind": "missing_reports",
            "count": missing_reports.len(),
            "items": missing_reports,
        }));
    }
    let failed: Vec<String> = rows
        .iter()
        .filter(|row| row.exit_code != 0)
        .map(|row| format!("{}:{}", row.stage_id, row.tool_id))
        .collect();
    if !failed.is_empty() {
        findings.push(serde_json::json!({
            "kind": "tool_failures",
            "count": failed.len(),
            "items": failed,
        }));
    }
    if let Some(entries) = stage_confidence
        .get("entries")
        .and_then(serde_json::Value::as_array)
    {
        let fragile: Vec<serde_json::Value> = entries
            .iter()
            .filter(|entry| {
                entry
                    .get("score")
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or(1.0)
                    < 0.6
            })
            .cloned()
            .collect();
        if !fragile.is_empty() {
            findings.push(serde_json::json!({
                "kind": "low_confidence_stages",
                "count": fragile.len(),
                "items": fragile,
            }));
        }
    }
    serde_json::Value::Array(findings)
}

pub(super) fn data_contract_validation_section(
    completeness: &bijux_dna_runtime::ReportCompletenessV1,
) -> serde_json::Value {
    serde_json::json!({
        "status": completeness.status,
        "missing_metrics": completeness.missing_metrics,
        "missing_reports": completeness.missing_reports,
    })
}
