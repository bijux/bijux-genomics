use super::declarations::EXPECTED_INSUFFICIENT_STATUS;
use super::declarations::EXPECTED_POPULATION_SCOPE;
use super::declarations::EXPECTED_READY_STATUS;
use super::declarations::EXPECTED_REFERENCE_BUILD;
use super::declarations::EXPECTED_REFERENCE_PANEL_ID;
use super::declarations::EXPECTED_REPORT_SCHEMA_VERSION;
use super::declarations::EXPECTED_SAMPLE_ID;
use super::declarations::EXPECTED_STAGE_ID;
use super::declarations::EXPECTED_STAGE_METRICS_SCHEMA_VERSION;
use super::declarations::EXPECTED_TOOL_IDS;
use super::declarations::REQUIRED_OUTPUT_IDS;
use super::observations::LocalHaplogroupsSmokeCaseReport;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use std::fs;
use std::path::Path;
pub(super) fn required_outputs_present(case: &LocalHaplogroupsSmokeCaseReport) -> bool {
    REQUIRED_OUTPUT_IDS
        .iter()
        .all(|output_id| case.declared_output_ids.iter().any(|candidate| candidate == output_id))
}

pub(super) fn case_artifacts_present(case: &LocalHaplogroupsSmokeCaseReport) -> bool {
    case.artifact_paths.iter().any(|path| path == &case.haplogroups_report)
        && case.artifact_paths.iter().any(|path| path == &case.haplogroups_summary)
        && case.artifact_paths.iter().any(|path| path == &case.stage_metrics)
        && case.artifact_paths.iter().any(|path| path == &case.haplogroup_report)
}

pub(super) fn case_parser_contract_ready(case: &LocalHaplogroupsSmokeCaseReport) -> bool {
    case.artifact_paths.iter().any(|path| path == &case.haplogroups_report)
        && case.artifact_paths.iter().any(|path| path == &case.haplogroups_summary)
        && case.artifact_paths.iter().any(|path| path == &case.haplogroup_report)
        && case.artifact_paths.iter().any(|path| path == &case.stage_metrics)
}

pub(super) fn report_contract_ready(
    payload: &serde_json::Value,
    expected: &ReportExpectation<'_>,
) -> bool {
    let ReportExpectation {
        ready: expected_ready,
        call: expected_call,
        confidence: expected_confidence,
        status: expected_status,
        minimum_coverage: expected_minimum_coverage,
        observed_mean_coverage: expected_observed_mean_coverage,
        markers_total: expected_markers_total,
        markers_supported: expected_markers_supported,
    } = *expected;
    payload.get("schema_version").and_then(serde_json::Value::as_str)
        == Some(EXPECTED_REPORT_SCHEMA_VERSION)
        && payload.get("stage_id").and_then(serde_json::Value::as_str) == Some(EXPECTED_STAGE_ID)
        && payload.get("tool_id").and_then(serde_json::Value::as_str) == Some(EXPECTED_TOOL_IDS[0])
        && payload.get("sample_id").and_then(serde_json::Value::as_str) == Some(EXPECTED_SAMPLE_ID)
        && payload.get("reference_panel_id").and_then(serde_json::Value::as_str)
            == Some(EXPECTED_REFERENCE_PANEL_ID)
        && payload.get("reference_build").and_then(serde_json::Value::as_str)
            == Some(EXPECTED_REFERENCE_BUILD)
        && payload.get("population_scope").and_then(serde_json::Value::as_str)
            == Some(EXPECTED_POPULATION_SCOPE)
        && optional_string_field(payload, "haplogroup_call") == expected_call
        && payload.get("confidence").and_then(serde_json::Value::as_f64)
            == Some(expected_confidence)
        && payload.get("status").and_then(serde_json::Value::as_str) == Some(expected_status)
        && payload.get("markers_total").and_then(serde_json::Value::as_u64)
            == Some(expected_markers_total as u64)
        && payload.get("markers_supported").and_then(serde_json::Value::as_u64)
            == Some(expected_markers_supported as u64)
        && payload
            .get("coverage_gate")
            .and_then(|value| value.get("min_coverage"))
            .and_then(serde_json::Value::as_f64)
            == Some(expected_minimum_coverage)
        && payload
            .get("coverage_gate")
            .and_then(|value| value.get("observed_mean_coverage"))
            .and_then(serde_json::Value::as_f64)
            == Some(expected_observed_mean_coverage)
        && (expected_ready
            || payload.get("haplogroup_call").is_some_and(serde_json::Value::is_null))
}

pub(super) fn insufficient_report_contract_ready(
    payload: &serde_json::Value,
    expected: &ReportExpectation<'_>,
) -> bool {
    let ReportExpectation {
        ready: expected_ready,
        call: expected_call,
        confidence: expected_confidence,
        status: expected_status,
        minimum_coverage: expected_minimum_coverage,
        observed_mean_coverage: expected_observed_mean_coverage,
        markers_total: expected_markers_total,
        markers_supported: expected_markers_supported,
    } = *expected;
    report_contract_ready(
        payload,
        &ReportExpectation {
            ready: expected_ready,
            call: expected_call,
            confidence: expected_confidence,
            status: expected_status,
            minimum_coverage: expected_minimum_coverage,
            observed_mean_coverage: expected_observed_mean_coverage,
            markers_total: expected_markers_total,
            markers_supported: expected_markers_supported,
        },
    )
}

pub(super) fn stage_metrics_ready(
    payload: &serde_json::Value,
    proof_case: &str,
    expected_ready: bool,
    expected_minimum_coverage: f64,
    expected_observed_mean_coverage: f64,
    expected_status: &str,
) -> bool {
    payload.get("schema_version").and_then(serde_json::Value::as_str)
        == Some(EXPECTED_STAGE_METRICS_SCHEMA_VERSION)
        && payload.get("stage_id").and_then(serde_json::Value::as_str) == Some(EXPECTED_STAGE_ID)
        && payload.get("tool_id").and_then(serde_json::Value::as_str) == Some(EXPECTED_TOOL_IDS[0])
        && payload.get("sample_id").and_then(serde_json::Value::as_str) == Some(EXPECTED_SAMPLE_ID)
        && payload.get("proof_case").and_then(serde_json::Value::as_str) == Some(proof_case)
        && payload.get("expected_ready").and_then(serde_json::Value::as_bool)
            == Some(expected_ready)
        && payload.get("ready").and_then(serde_json::Value::as_bool) == Some(expected_ready)
        && payload.get("minimum_coverage").and_then(serde_json::Value::as_f64)
            == Some(expected_minimum_coverage)
        && payload.get("observed_mean_coverage").and_then(serde_json::Value::as_f64)
            == Some(expected_observed_mean_coverage)
        && payload.get("status").and_then(serde_json::Value::as_str) == Some(expected_status)
        && payload.get("expectation_matched").and_then(serde_json::Value::as_bool) == Some(true)
}

pub(super) fn optional_string_field<'a>(
    payload: &'a serde_json::Value,
    key: &str,
) -> Option<&'a str> {
    payload.get(key).and_then(serde_json::Value::as_str)
}

pub(super) fn tool_specific_artifact_ready(
    ready_path: &Path,
    insufficient_path: &Path,
    ready_case: &LocalHaplogroupsSmokeCaseReport,
    insufficient_case: &LocalHaplogroupsSmokeCaseReport,
) -> Result<bool> {
    let ready_payload: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(ready_path)
            .with_context(|| format!("read {}", ready_path.display()))?,
    )
    .with_context(|| format!("parse {}", ready_path.display()))?;
    let insufficient_payload: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(insufficient_path)
            .with_context(|| format!("read {}", insufficient_path.display()))?,
    )
    .with_context(|| format!("parse {}", insufficient_path.display()))?;

    let ready_ok = ready_payload.get("artifact_id").and_then(serde_json::Value::as_str)
        == Some("haplogroup_report")
        && ready_payload.get("stage_id").and_then(serde_json::Value::as_str)
            == Some(EXPECTED_STAGE_ID)
        && ready_payload.get("tool_id").and_then(serde_json::Value::as_str)
            == Some(EXPECTED_TOOL_IDS[0])
        && ready_payload.get("proof_case").and_then(serde_json::Value::as_str) == Some("ready")
        && optional_string_field(&ready_payload, "haplogroup")
            == ready_case.haplogroup_call.as_deref()
        && ready_payload.get("confidence").and_then(serde_json::Value::as_f64)
            == Some(ready_case.confidence)
        && ready_payload.get("status").and_then(serde_json::Value::as_str)
            == Some(EXPECTED_READY_STATUS)
        && ready_payload.get("reference_panel_id").and_then(serde_json::Value::as_str)
            == Some(EXPECTED_REFERENCE_PANEL_ID)
        && ready_payload.get("markers_total").and_then(serde_json::Value::as_u64)
            == Some(ready_case.markers_total as u64)
        && ready_payload.get("markers_supported").and_then(serde_json::Value::as_u64)
            == Some(ready_case.markers_supported as u64);
    let insufficient_ok =
        insufficient_payload.get("artifact_id").and_then(serde_json::Value::as_str)
            == Some("haplogroup_report")
            && insufficient_payload.get("stage_id").and_then(serde_json::Value::as_str)
                == Some(EXPECTED_STAGE_ID)
            && insufficient_payload.get("tool_id").and_then(serde_json::Value::as_str)
                == Some(EXPECTED_TOOL_IDS[0])
            && insufficient_payload.get("proof_case").and_then(serde_json::Value::as_str)
                == Some("insufficient")
            && insufficient_payload.get("haplogroup").is_some_and(serde_json::Value::is_null)
            && insufficient_payload.get("confidence").and_then(serde_json::Value::as_f64)
                == Some(insufficient_case.confidence)
            && insufficient_payload.get("status").and_then(serde_json::Value::as_str)
                == Some(EXPECTED_INSUFFICIENT_STATUS)
            && insufficient_payload.get("reference_panel_id").and_then(serde_json::Value::as_str)
                == Some(EXPECTED_REFERENCE_PANEL_ID)
            && insufficient_payload.get("markers_total").and_then(serde_json::Value::as_u64)
                == Some(insufficient_case.markers_total as u64)
            && insufficient_payload.get("markers_supported").and_then(serde_json::Value::as_u64)
                == Some(insufficient_case.markers_supported as u64);
    Ok(ready_ok && insufficient_ok)
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

#[derive(Clone, Copy)]
pub(super) struct ReportExpectation<'a> {
    pub ready: bool,
    pub call: Option<&'a str>,
    pub confidence: f64,
    pub status: &'a str,
    pub minimum_coverage: f64,
    pub observed_mean_coverage: f64,
    pub markers_total: usize,
    pub markers_supported: usize,
}
