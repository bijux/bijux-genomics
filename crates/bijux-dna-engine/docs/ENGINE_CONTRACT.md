# Engine Contract

## Purpose
The engine executes a fully-formed `ExecutionGraph` and emits the per-step truth set.

## Inputs
- `ExecutionGraph` (bijux-dna-core contract)
- `Runner` implementation
- Run layout (`bijux-dna-runtime`)

## Outputs (truth set per step)
Written into `run_artifacts/` for each step:
- `effective_config.json`
- `tool_invocation.json`
- `metrics.json`
- `stage_report.json`
- `execution_record.json`
- `metrics_envelope.json` when `ExecutionStep::metrics_schema_ids` is non-empty

## Non-goals
- Planning or tool selection
- Process spawning or runtime backend logic
- Domain semantics (owned by planners and domains)

## Contract validation
The engine verifies that required run artifacts are non-empty parseable JSON, declared JSON outputs
are parseable JSON, expected artifact IDs are declared as outputs, and metrics envelopes match one
of the step's declared metrics schema IDs.
