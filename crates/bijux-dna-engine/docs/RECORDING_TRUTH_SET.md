# RECORDING_TRUTH_SET

## Required per-step artifacts
Every executed step must emit:
- `effective_config.json`
- `tool_invocation.json`
- `execution_record.json`
- `metrics.json`
- `stage_report.json`

Steps with declared `metrics_schema_ids` must also emit:
- `metrics_envelope.json`

## Minimal example
```
run_123/
  stage_0/
    effective_config.json
    tool_invocation.json
    execution_record.json
    metrics.json
    stage_report.json
    metrics_envelope.json
```

## Field meanings
- `tool_invocation.json`: tool id/version/image/params/input hashes
- `effective_config.json`: merged params and defaults
- `execution_record.json`: timing, exit status, resource summary
- `metrics_envelope.json`: typed metrics payload whose schema must be declared by the step

Enforced by `tests/contracts/recording/recording_completeness.rs`.
