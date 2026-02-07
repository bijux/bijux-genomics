# RECORDING_TRUTH_SET

## Required per-step artifacts
Every executed step must emit:
- `effective_config.json`
- `tool_invocation.json`
- `execution_record.json`
- `metrics.json` (when metrics required)
- `stage_report.json` (when required)

## Minimal example
```
run_123/
  stage_0/
    effective_config.json
    tool_invocation.json
    execution_record.json
    metrics.json
    stage_report.json
```

## Field meanings
- `tool_invocation.json`: tool id/version/image/params/input hashes
- `effective_config.json`: merged params and defaults
- `execution_record.json`: timing, exit status, resource summary

Enforced by `tests/recording/recording_completeness.rs`.
