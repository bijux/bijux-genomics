# configs/logging

## What
Configuration files for the logging domain.

## Philosophy
Keep logging configuration scoped to this directory so ownership is explicit and drift is easy to detect.

## Notes
Logging presets and format knobs belong here.

## Runtime Knobs
- `configs/logging/runtime.toml`
- Environment variables:
- `RUST_LOG`: base log level/filter policy.
- `BIJUX_LOG_FORMAT`: `json` or `text`.
- `BIJUX_LOG_INCLUDE_FIELDS`: comma-separated list of JSON fields to emit.

## JSON Fields Contract
- Required fields for JSON logs: `timestamp`, `level`, `target`, `message`, `trace_id`, `run_id`.
