use super::declarations::EXPECTED_STAGE_METRICS_SCHEMA_VERSION;
use super::declarations::REQUIRED_OUTPUT_IDS;
use super::observations::LocalContaminationSmokeCaseReport;
use anyhow::anyhow;
use anyhow::Result;
use std::path::Path;
pub(super) fn required_outputs_present(case: &LocalContaminationSmokeCaseReport) -> bool {
    REQUIRED_OUTPUT_IDS
        .iter()
        .all(|output_id| case.declared_output_ids.iter().any(|candidate| candidate == output_id))
}

pub(super) fn tool_specific_output_path(
    case: &LocalContaminationSmokeCaseReport,
) -> Option<String> {
    match case.tool_id.as_str() {
        "contammix" => case.contammix_report.clone(),
        "schmutzi" => case.mt_consensus.clone(),
        "verifybamid2" => case.contamination_estimate.clone(),
        _ => None,
    }
}

pub(super) fn required_stage_metrics_ready(
    payload: &serde_json::Value,
    proof_case: &str,
    expected_prerequisites_passed: bool,
    actual_prerequisites_passed: bool,
) -> bool {
    payload.get("schema_version").and_then(serde_json::Value::as_str)
        == Some(EXPECTED_STAGE_METRICS_SCHEMA_VERSION)
        && payload.get("proof_case").and_then(serde_json::Value::as_str) == Some(proof_case)
        && payload.get("expected_prerequisites_passed").and_then(serde_json::Value::as_bool)
            == Some(expected_prerequisites_passed)
        && payload.get("prerequisites_passed").and_then(serde_json::Value::as_bool)
            == Some(actual_prerequisites_passed)
        && payload.get("expectation_matched").and_then(serde_json::Value::as_bool) == Some(true)
}

pub(super) fn required_string_field(
    payload: &serde_json::Value,
    key: &str,
    path: &Path,
) -> Result<String> {
    payload
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("{} is missing string `{key}`", path.display()))
}

pub(super) fn float_matches(left: f64, right: f64) -> bool {
    (left - right).abs() <= 1e-9
}
