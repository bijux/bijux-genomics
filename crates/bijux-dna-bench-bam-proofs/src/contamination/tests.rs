#![cfg(test)]
use super::contracts::required_stage_metrics_ready;
use super::declarations::EXPECTED_STAGE_METRICS_SCHEMA_VERSION;
#[test]
fn keeps_ready_and_insufficient_prerequisite_evidence_distinct() {
    let mut payload = serde_json::json!({"schema_version":EXPECTED_STAGE_METRICS_SCHEMA_VERSION,"proof_case":"ready","expected_prerequisites_passed":true,"prerequisites_passed":true,"expectation_matched":true});
    assert!(required_stage_metrics_ready(&payload, "ready", true, true));
    assert!(!required_stage_metrics_ready(&payload, "insufficient", false, false));
    payload["prerequisites_passed"] = serde_json::json!(false);
    assert!(!required_stage_metrics_ready(&payload, "ready", true, true));
    payload["proof_case"] = serde_json::json!("insufficient");
    payload["expected_prerequisites_passed"] = serde_json::json!(false);
    assert!(required_stage_metrics_ready(&payload, "insufficient", false, false));
    payload["expectation_matched"] = serde_json::json!(false);
    assert!(!required_stage_metrics_ready(&payload, "insufficient", false, false));
}
