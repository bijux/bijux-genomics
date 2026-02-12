use bijux_dna_runtime::{FailureCode, TelemetryEventName, TelemetryEventV1};

#[test]
fn telemetry_jsonl_golden_for_toy_run_is_stable() -> anyhow::Result<()> {
    let start = TelemetryEventV1 {
        schema_version: "bijux.telemetry.v1".to_string(),
        run_id: "toy-run".to_string(),
        stage_id: "fastq.trim".to_string(),
        tool_id: "fastp".to_string(),
        event_name: TelemetryEventName::StageStart,
        timestamp: chrono::DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")?
            .with_timezone(&chrono::Utc),
        duration_ms: None,
        status: "running".to_string(),
        trace_id: "trace-fastq.trim".to_string(),
        span_id: "span-fastp".to_string(),
        attrs: std::collections::BTreeMap::new(),
        failure_code: None,
    };
    let end = TelemetryEventV1 {
        schema_version: "bijux.telemetry.v1".to_string(),
        run_id: "toy-run".to_string(),
        stage_id: "fastq.trim".to_string(),
        tool_id: "fastp".to_string(),
        event_name: TelemetryEventName::StageEnd,
        timestamp: chrono::DateTime::parse_from_rfc3339("2026-01-01T00:00:05Z")?
            .with_timezone(&chrono::Utc),
        duration_ms: Some(5000),
        status: "ok".to_string(),
        trace_id: "trace-fastq.trim".to_string(),
        span_id: "span-fastp".to_string(),
        attrs: std::collections::BTreeMap::from([(
            "bytes_written".to_string(),
            bijux_dna_runtime::AttrValue::Int(1024),
        )]),
        failure_code: Some(FailureCode::ToolFailed),
    };

    let rendered = [serde_json::to_string(&start)?, serde_json::to_string(&end)?].join("\n") + "\n";
    let expected = include_str!(
        "../fixtures/runtime_schema/telemetry_toy_run/telemetry_toy_run.jsonl"
    );
    assert_eq!(rendered, expected);
    Ok(())
}
