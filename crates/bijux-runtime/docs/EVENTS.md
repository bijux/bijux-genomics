# EVENTS

## Emitted events
- `step_start`
- `step_end`
- `artifact_verified`
- `cache_hit`

## Telemetry minimal contract
Stable fields (do not change without versioning):
- `schema_version`
- `run_id`
- `stage_id`
- `tool_id`
- `event_name`
- `status`
- `trace_id`
- `span_id`
- `attrs`

Intentionally unstable fields:
- `timestamp` (wall clock; not stable across replays)
- `duration_ms` (computed; may change with runtime conditions)

## Required fields
- `event_name`
- `stage_id`
- `timestamp`

## Stability
Additive fields are backward compatible.

## Privacy
Do not emit PII. Redact sensitive values.
