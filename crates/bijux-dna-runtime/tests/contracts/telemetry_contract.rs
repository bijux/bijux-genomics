use bijux_dna_runtime::{
    redacted_attrs, validate_stage_telemetry, AttrMap, AttrValue, FailureCode, TelemetryEventName,
    TelemetryEventV1,
};

#[test]
fn telemetry_event_taxonomy_rejects_unknown_event_names() {
    let raw = serde_json::json!({
        "schema_version": "bijux.telemetry.v1",
        "run_id": "run-1",
        "stage_id": "fastq.preprocess",
        "tool_id": "planner",
        "event_name": "not_allowed",
        "timestamp": "2026-01-01T00:00:00Z",
        "duration_ms": null,
        "status": "ok",
        "trace_id": "trace-1",
        "span_id": "span-1",
        "attrs": {}
    });
    let parsed = serde_json::from_value::<TelemetryEventV1>(raw);
    assert!(
        parsed.is_err(),
        "unknown telemetry event_name must fail deserialization"
    );
}

#[test]
fn telemetry_event_serializes_typed_timestamp_and_event_name() -> anyhow::Result<()> {
    let event = TelemetryEventV1 {
        schema_version: "bijux.telemetry.v1".to_string(),
        run_id: "run-1".to_string(),
        stage_id: "fastq.preprocess".to_string(),
        tool_id: "planner".to_string(),
        event_name: TelemetryEventName::RunStarted,
        timestamp: chrono::DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")?
            .with_timezone(&chrono::Utc),
        duration_ms: None,
        status: "ok".to_string(),
        trace_id: "trace-1".to_string(),
        span_id: "span-1".to_string(),
        attrs: std::collections::BTreeMap::default(),
        failure_code: None,
    };
    let value = serde_json::to_value(&event)?;
    assert_eq!(
        value.get("event_name").and_then(|v| v.as_str()),
        Some("run_started")
    );
    assert_eq!(
        value.get("timestamp").and_then(|v| v.as_str()),
        Some("2026-01-01T00:00:00Z")
    );
    Ok(())
}

#[test]
fn telemetry_attrs_redact_sensitive_keys() {
    let mut attrs = AttrMap::new();
    attrs.insert("api_token".to_string(), AttrValue::Str("abc".to_string()));
    attrs.insert("safe".to_string(), AttrValue::Str("ok".to_string()));
    let redacted = redacted_attrs(&attrs);
    assert_eq!(
        redacted.get("api_token"),
        Some(&AttrValue::Str("[REDACTED]".to_string()))
    );
    assert_eq!(
        redacted.get("safe"),
        Some(&AttrValue::Str("ok".to_string()))
    );
}

#[test]
fn telemetry_failure_code_is_snake_case() -> anyhow::Result<()> {
    let event = TelemetryEventV1 {
        schema_version: "bijux.telemetry.v1".to_string(),
        run_id: "run-1".to_string(),
        stage_id: "fastq.preprocess".to_string(),
        tool_id: "planner".to_string(),
        event_name: TelemetryEventName::RunFailed,
        timestamp: chrono::DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")?
            .with_timezone(&chrono::Utc),
        duration_ms: Some(10),
        status: "error".to_string(),
        trace_id: "trace-1".to_string(),
        span_id: "span-1".to_string(),
        attrs: std::collections::BTreeMap::default(),
        failure_code: Some(FailureCode::ToolFailed),
    };
    let value = serde_json::to_value(&event)?;
    assert_eq!(
        value.get("failure_code").and_then(|v| v.as_str()),
        Some("tool_failed")
    );
    Ok(())
}

#[test]
fn telemetry_contract_requires_stage_start_end_and_artifact_refs() -> anyhow::Result<()> {
    let base = TelemetryEventV1 {
        schema_version: "bijux.telemetry.v1".to_string(),
        run_id: "run-1".to_string(),
        stage_id: "fastq.trim".to_string(),
        tool_id: "fastp".to_string(),
        event_name: TelemetryEventName::StageStart,
        timestamp: chrono::DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")?
            .with_timezone(&chrono::Utc),
        duration_ms: None,
        status: "ok".to_string(),
        trace_id: "trace-1".to_string(),
        span_id: "span-1".to_string(),
        attrs: std::collections::BTreeMap::default(),
        failure_code: None,
    };
    let mut artifact = base.clone();
    artifact.event_name = TelemetryEventName::ArtifactWritten;
    artifact.attrs = std::collections::BTreeMap::from([(
        "artifact_path".to_string(),
        AttrValue::Str("trimmed.fastq.gz".to_string()),
    )]);
    let mut end = base.clone();
    end.event_name = TelemetryEventName::StageEnd;
    let events = vec![base, artifact, end];
    assert!(validate_stage_telemetry(&events).is_empty());
    Ok(())
}
