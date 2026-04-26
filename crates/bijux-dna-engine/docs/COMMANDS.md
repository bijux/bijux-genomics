# Commands

This file is the SSOT for callable operations managed by `bijux-dna-engine`.
The engine owns Rust operations, not CLI commands. CLI parsing and user-facing
command routing belong outside this crate.

## Managed Engine Operations

| Operation | Rust entrypoint | Purpose |
| --- | --- | --- |
| `create-engine` | `Engine::new` | Build an engine with explicit execution policy. |
| `execute-graph` | `Engine::execute` | Validate engine policy and run layout, execute a planned graph through a caller-provided runner, and return a run record. |
| `validate-engine-config` | `EngineConfig::validate` | Reject unsupported execution policy such as parallelism greater than one. |
| `cancel-execution` | `CancellationToken::cancel` | Request cooperative cancellation. |
| `check-cancellation` | `CancellationToken::is_cancelled` | Observe cancellation state before or during execution. |
| `observe-engine-event` | `EngineHooks::on_event` | Receive engine events without coupling to executor internals. |
| `prepare-execution-graph` | `executor::graph::normalize_for_execution` | Normalize and order graph steps for execution. Internal operation. |
| `execute-ordered-steps` | `executor::step_execution::execute_ordered_steps` | Run ordered steps sequentially. Internal operation. |
| `record-execution` | `executor::recording::record_execution` | Persist per-step `execution_record.json`. Internal operation. |
| `enforce-output-contract` | `executor::contracts::outputs::verify_outputs` | Verify declared outputs and JSON output roles. Internal operation. |
| `enforce-run-artifacts` | `executor::contracts::run_artifacts::verify_required_run_artifacts` | Verify required per-step run artifacts. Internal operation. |
| `enforce-metrics-envelope` | `executor::contracts::metrics::verify_metrics_envelope` | Verify declared metrics envelope schema IDs. Internal operation. |

## Local Verification Commands

Run from the `bijux-genomics` repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo check -p bijux-dna-engine --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-engine --test boundaries --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-engine --test contracts --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-engine --test determinism --no-default-features
```
