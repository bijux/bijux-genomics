use super::declarations::EXPECTED_INSUFFICIENCY_REASON;
use super::declarations::EXPECTED_REPORT_SCHEMA_VERSION;
use super::declarations::EXPECTED_STAGE_ID;
use super::declarations::EXPECTED_STAGE_METRICS_SCHEMA_VERSION;
use super::declarations::REQUIRED_OUTPUT_IDS;
use super::observations::LocalSexToolSmokeCaseReport;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use std::fs;
use std::path::Path;
pub(super) fn required_outputs_present(case: &LocalSexToolSmokeCaseReport) -> bool {
    REQUIRED_OUTPUT_IDS
        .iter()
        .all(|output_id| case.declared_output_ids.iter().any(|candidate| candidate == output_id))
}

pub(super) fn case_artifacts_present(case: &LocalSexToolSmokeCaseReport) -> bool {
    let tool_specific_output_path = tool_specific_output_path(case);
    case.artifact_paths.iter().any(|path| path == &case.sex_report)
        && case.artifact_paths.iter().any(|path| path == &case.sex_summary)
        && case.artifact_paths.iter().any(|path| path == &case.stage_metrics)
        && case.artifact_paths.iter().any(|path| path == tool_specific_output_path)
}

pub(super) fn tool_specific_output_path(case: &LocalSexToolSmokeCaseReport) -> &str {
    match case.tool_id.as_str() {
        "angsd" => &case.population_metrics,
        "yleaf" => &case.haplogroup_report,
        _ => &case.sex_estimate,
    }
}

pub(super) fn ready_report_contract_ready(payload: &serde_json::Value, tool_id: &str) -> bool {
    payload.get("schema_version").and_then(serde_json::Value::as_str)
        == Some(EXPECTED_REPORT_SCHEMA_VERSION)
        && payload.get("method").and_then(serde_json::Value::as_str) == Some(tool_id)
        && payload.get("call").and_then(serde_json::Value::as_str).is_some()
        && payload.get("classification").and_then(serde_json::Value::as_str).is_some()
        && payload.get("confidence").and_then(serde_json::Value::as_f64).is_some()
        && payload.get("status").and_then(serde_json::Value::as_str) == Some("ok")
        && payload.get("insufficiency_reason").is_some_and(serde_json::Value::is_null)
}

pub(super) fn insufficient_report_contract_ready(
    payload: &serde_json::Value,
    tool_id: &str,
    expected_reason: &str,
) -> bool {
    payload.get("schema_version").and_then(serde_json::Value::as_str)
        == Some(EXPECTED_REPORT_SCHEMA_VERSION)
        && payload.get("method").and_then(serde_json::Value::as_str) == Some(tool_id)
        && payload.get("call").and_then(serde_json::Value::as_str) == Some("insufficient")
        && payload.get("classification").and_then(serde_json::Value::as_str) == Some("insufficient")
        && payload.get("confidence").and_then(serde_json::Value::as_f64) == Some(0.0)
        && payload.get("status").and_then(serde_json::Value::as_str) == Some(expected_reason)
        && payload.get("insufficiency_reason").and_then(serde_json::Value::as_str)
            == Some(expected_reason)
}

pub(super) fn stage_metrics_ready(
    payload: &serde_json::Value,
    proof_case: &str,
    expected_insufficiency_reason: Option<&str>,
    expected_call: &str,
    expected_confidence: f64,
    expected_status: &str,
) -> bool {
    payload.get("schema_version").and_then(serde_json::Value::as_str)
        == Some(EXPECTED_STAGE_METRICS_SCHEMA_VERSION)
        && payload.get("stage_id").and_then(serde_json::Value::as_str) == Some(EXPECTED_STAGE_ID)
        && payload.get("proof_case").and_then(serde_json::Value::as_str) == Some(proof_case)
        && payload.get("expected_call").and_then(serde_json::Value::as_str) == Some(expected_call)
        && payload.get("call").and_then(serde_json::Value::as_str) == Some(expected_call)
        && payload.get("expected_confidence").and_then(serde_json::Value::as_f64)
            == Some(expected_confidence)
        && payload.get("confidence").and_then(serde_json::Value::as_f64)
            == Some(expected_confidence)
        && payload.get("expected_status").and_then(serde_json::Value::as_str)
            == Some(expected_status)
        && payload.get("status").and_then(serde_json::Value::as_str) == Some(expected_status)
        && optional_string_field(payload, "expected_insufficiency_reason")
            == expected_insufficiency_reason
        && optional_string_field(payload, "insufficiency_reason") == expected_insufficiency_reason
        && payload.get("expectation_matched").and_then(serde_json::Value::as_bool) == Some(true)
}

pub(super) fn optional_string_field<'a>(
    payload: &'a serde_json::Value,
    key: &str,
) -> Option<&'a str> {
    payload.get(key).and_then(serde_json::Value::as_str)
}

pub(super) fn sex_call_name(
    call: bijux_dna_domain_bam::metrics::SexConfidenceClass,
) -> &'static str {
    match call {
        bijux_dna_domain_bam::metrics::SexConfidenceClass::Male => "male",
        bijux_dna_domain_bam::metrics::SexConfidenceClass::Female => "female",
        bijux_dna_domain_bam::metrics::SexConfidenceClass::Ambiguous => "ambiguous",
        bijux_dna_domain_bam::metrics::SexConfidenceClass::Insufficient => "insufficient",
    }
}

pub(super) fn tool_specific_artifact_ready(
    tool_id: &str,
    ready_path: &Path,
    insufficient_path: &Path,
    ready_case: &LocalSexToolSmokeCaseReport,
    insufficient_case: &LocalSexToolSmokeCaseReport,
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

    let ready_common = ready_payload.get("stage_id").and_then(serde_json::Value::as_str)
        == Some(EXPECTED_STAGE_ID)
        && ready_payload.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
        && ready_payload.get("proof_case").and_then(serde_json::Value::as_str) == Some("ready");
    let insufficient_common = insufficient_payload
        .get("stage_id")
        .and_then(serde_json::Value::as_str)
        == Some(EXPECTED_STAGE_ID)
        && insufficient_payload.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
        && insufficient_payload.get("proof_case").and_then(serde_json::Value::as_str)
            == Some("insufficient");

    let ready_ok = match tool_id {
        "angsd" => {
            ready_common
                && ready_payload.get("artifact_id").and_then(serde_json::Value::as_str)
                    == Some("population_metrics")
                && ready_payload.get("chromosome_system").and_then(serde_json::Value::as_str)
                    == ready_case.chromosome_system.as_deref()
                && ready_payload.get("x_coverage").and_then(serde_json::Value::as_f64)
                    == Some(ready_case.x_coverage)
                && ready_payload.get("y_coverage").and_then(serde_json::Value::as_f64)
                    == Some(ready_case.y_coverage)
                && ready_payload.get("autosomal_coverage").and_then(serde_json::Value::as_f64)
                    == Some(ready_case.autosomal_coverage)
        }
        "rxy" => {
            ready_common
                && ready_payload.get("artifact_id").and_then(serde_json::Value::as_str)
                    == Some("sex_estimate")
                && ready_payload.get("call").and_then(serde_json::Value::as_str)
                    == Some(sex_call_name(ready_case.call))
                && ready_payload.get("confidence").and_then(serde_json::Value::as_f64)
                    == Some(ready_case.confidence)
                && ready_payload.get("status").and_then(serde_json::Value::as_str)
                    == Some(ready_case.status.as_str())
        }
        "yleaf" => {
            ready_common
                && ready_payload.get("artifact_id").and_then(serde_json::Value::as_str)
                    == Some("haplogroup_report")
                && ready_payload.get("status").and_then(serde_json::Value::as_str)
                    == Some("not_applicable_for_sex_inference")
        }
        _ => false,
    };

    let insufficient_ok = match tool_id {
        "angsd" => {
            insufficient_common
                && insufficient_payload.get("artifact_id").and_then(serde_json::Value::as_str)
                    == Some("population_metrics")
                && insufficient_payload.get("x_coverage").and_then(serde_json::Value::as_f64)
                    == Some(insufficient_case.x_coverage)
                && insufficient_payload
                    .get("autosomal_coverage")
                    .and_then(serde_json::Value::as_f64)
                    == Some(insufficient_case.autosomal_coverage)
        }
        "rxy" => {
            insufficient_common
                && insufficient_payload.get("artifact_id").and_then(serde_json::Value::as_str)
                    == Some("sex_estimate")
                && insufficient_payload.get("call").and_then(serde_json::Value::as_str)
                    == Some("insufficient")
                && insufficient_payload
                    .get("insufficiency_reason")
                    .and_then(serde_json::Value::as_str)
                    == Some(EXPECTED_INSUFFICIENCY_REASON)
        }
        "yleaf" => {
            insufficient_common
                && insufficient_payload.get("artifact_id").and_then(serde_json::Value::as_str)
                    == Some("haplogroup_report")
                && insufficient_payload.get("status").and_then(serde_json::Value::as_str)
                    == Some("not_applicable_due_to_insufficient_chromosomes")
        }
        _ => false,
    };

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
