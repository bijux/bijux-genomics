//! Owner: admit report-level insufficiency markers from legacy evidence payloads.
use super::acquisition::ResultEvidence;
use anyhow::Result;
use bijux_dna_bench_model::{MicroBenchmarkInsufficientDataRow, MicroBenchmarkResultRow};
use serde_json::Value;
pub(super) fn detect_insufficient_data_row(
    evidence: &mut ResultEvidence<'_>,
    row: &MicroBenchmarkResultRow,
) -> Result<Option<MicroBenchmarkInsufficientDataRow>> {
    let reason_lower = row.reason.to_ascii_lowercase();
    if reason_lower.contains("insufficient") {
        return Ok(Some(MicroBenchmarkInsufficientDataRow {
            execution_id: row.execution_id.clone(),
            component_id: row.component_id.clone(),
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            source_report_path: row.source_report_path.clone(),
            evidence_path: row.evidence_path.clone(),
            detail: row.reason.clone(),
        }));
    }
    let Some(evidence_path) = &row.evidence_path else {
        return Ok(None);
    };
    if !evidence_path.ends_with(".json") {
        return Ok(None);
    }
    let payload = evidence.json()?;
    if json_contains_insufficient_marker(payload) {
        return Ok(Some(MicroBenchmarkInsufficientDataRow {
            execution_id: row.execution_id.clone(),
            component_id: row.component_id.clone(),
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            source_report_path: row.source_report_path.clone(),
            evidence_path: row.evidence_path.clone(),
            detail: "evidence report retains an insufficient-data marker".to_string(),
        }));
    }
    Ok(None)
}

fn json_contains_insufficient_marker(value: &Value) -> bool {
    match value {
        Value::Object(map) => {
            const STATUS_KEYS: &[&str] = &["status", "state", "result_status", "summary_status"];

            STATUS_KEYS.iter().any(|key| {
                map.get(*key).and_then(Value::as_str).is_some_and(text_contains_insufficient_marker)
            }) || map.get("summary").is_some_and(json_contains_insufficient_marker)
        }
        _ => false,
    }
}

fn text_contains_insufficient_marker(text: &str) -> bool {
    let text = text.to_ascii_lowercase();
    text.contains("insufficient_data")
        || text.contains("insufficient-data")
        || text.contains("insufficient_overlap")
        || text.contains("insufficient-overlap")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn insufficient_marker_detector_matches_nested_json() {
        let payload = json!({
            "summary": {
                "status": "insufficient_data"
            }
        });
        assert!(json_contains_insufficient_marker(&payload));
    }

    #[test]
    fn insufficient_marker_detector_ignores_expected_case_level_insufficiency() {
        let payload = json!({
            "all_cases_matched": true,
            "cases": [
                {
                    "status": "insufficient",
                    "insufficiency_reason": "insufficient_overlap_snps"
                },
                {
                    "status": "ok"
                }
            ]
        });
        assert!(!json_contains_insufficient_marker(&payload));
    }
}
