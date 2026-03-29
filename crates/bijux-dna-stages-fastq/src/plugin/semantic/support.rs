use super::*;

pub(crate) fn validate_semantic_metrics(
    artifacts: &[ArtifactRef],
) -> Option<serde_json::Value> {
    let report = artifacts
        .iter()
        .find(|artifact| artifact.name.as_str() == "validation_report")
        .map(|artifact| artifact.path.as_path())
        .and_then(|report_path| {
            fs::read_to_string(report_path)
                .ok()
                .and_then(|raw_report| parse_validation_report(&raw_report).ok())
        });
    let manifest = artifacts
        .iter()
        .find(|artifact| artifact.name.as_str() == "validated_reads_manifest")
        .map(|artifact| artifact.path.as_path())
        .and_then(|manifest_path| {
            fs::read_to_string(manifest_path)
                .ok()
                .and_then(|raw_manifest| parse_validated_reads_manifest(&raw_manifest).ok())
        });
    if report.is_none() && manifest.is_none() {
        return None;
    }
    Some(serde_json::json!({
        "tool_id": report.as_ref().map(|value| value.tool_id.clone()).or_else(|| manifest.as_ref().map(|value| value.tool_id.clone())),
        "validation_mode": report.as_ref().and_then(|value| serde_json::to_value(&value.validation_mode).ok()).unwrap_or(serde_json::Value::Null),
        "pair_sync_policy": report.as_ref().and_then(|value| serde_json::to_value(&value.pair_sync_policy).ok()).unwrap_or(serde_json::Value::Null),
        "failure_class": report.as_ref().and_then(|value| serde_json::to_value(&value.failure_class).ok()).unwrap_or(serde_json::Value::Null),
        "strict_pass": report.as_ref().map(|value| serde_json::json!(value.strict_pass)).unwrap_or(serde_json::Value::Null),
        "exit_code": report.as_ref().map(|value| serde_json::json!(value.exit_code)).unwrap_or(serde_json::Value::Null),
        "validated_inputs": report.as_ref().map(|value| serde_json::json!(value.validated_inputs)).unwrap_or(serde_json::Value::Null),
        "validated_reads_r1": report.as_ref().map(|value| serde_json::json!(value.validated_reads_r1)).unwrap_or(serde_json::Value::Null),
        "validated_reads_r2": report.as_ref().and_then(|value| serde_json::to_value(value.validated_reads_r2).ok()).unwrap_or(serde_json::Value::Null),
        "validated_pairs": report.as_ref().and_then(|value| serde_json::to_value(value.validated_pairs).ok()).unwrap_or(serde_json::Value::Null),
        "status_r1": report.as_ref().map(|value| serde_json::json!(value.status_r1)).unwrap_or(serde_json::Value::Null),
        "status_r2": report.as_ref().map(|value| serde_json::json!(value.status_r2)).unwrap_or(serde_json::Value::Null),
        "pair_sync_checked": report.as_ref().map(|value| serde_json::json!(value.pair_sync_checked)).or_else(|| manifest.as_ref().map(|value| serde_json::json!(value.pair_sync_checked))).unwrap_or(serde_json::Value::Null),
        "pair_sync_pass": report.as_ref().and_then(|value| serde_json::to_value(value.pair_sync_pass).ok()).or_else(|| manifest.as_ref().and_then(|value| serde_json::to_value(value.pair_sync_pass).ok())).unwrap_or(serde_json::Value::Null),
        "pair_count_match": report.as_ref().and_then(|value| serde_json::to_value(value.pair_count_match).ok()).unwrap_or(serde_json::Value::Null),
        "paired_mode": manifest.as_ref().and_then(|value| serde_json::to_value(value.paired_mode).ok()).unwrap_or(serde_json::Value::Null),
        "validated_stream_ids": manifest.as_ref().map(|value| serde_json::json!(value.validated_stream_ids)).unwrap_or(serde_json::Value::Null),
        "validation_report": manifest.as_ref().map(|value| serde_json::json!(value.validation_report)).unwrap_or(serde_json::Value::Null),
    }))
}

pub(super) fn parse_qc_contributor_identity(name: &str) -> Option<(String, String)> {
    let mut parts = name.split('.');
    let domain = parts.next()?;
    let stage = parts.next()?;
    let tool = parts.next()?;
    Some((format!("{domain}.{stage}"), tool.to_string()))
}
