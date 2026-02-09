use bijux_dna_runtime::{TelemetryEventName, TelemetryEventV1};

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
