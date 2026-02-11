# OBSERVABILITY

## Event schema
Telemetry uses `bijux.telemetry.v1` JSONL events with:
- `event_name`: `run_started`, `stage_start`, `tool_invocation`, `stdout_summary`,
  `stderr_summary`, `metrics_emitted`, `invariant_result`, `artifact_written`,
  `stage_end`, `run_finished`, `run_failed`, and scientific decision events.
- `failure_code`: finite taxonomy (`tool_failed`, `missing_artifact`, `invalid_params`,
  `invariant_violation`, `io_error`, `timeout`, `parse_error`, `unknown`).
- `attrs`: redacted key-value attributes (`*token*`, `*secret*`, etc. are masked).

## Files emitted per run
- `run_artifacts/telemetry.jsonl`: append-only event stream.
- `run_artifacts/run_summary.json`: compact runtime summary.

## Interpretation
- A valid stage execution should emit `stage_start` and `stage_end`.
- `artifact_written` events must include an artifact reference (`artifact_id` or `artifact_path`).
- `run_failed` must include a `failure_code`.

## Enforced by
- `crates/bijux-dna-runtime/tests/contracts/telemetry_contract.rs`
- `crates/bijux-dna-runtime/tests/contracts/telemetry_golden.rs`
