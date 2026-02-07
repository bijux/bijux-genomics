# Engine Contract

## Purpose
The engine executes a fully-formed `ExecutionGraph` and emits the per-step truth set.

## Inputs
- `ExecutionGraph` (bijux-core contract)
- `Runner` implementation
- Run layout (`bijux-runtime`)

## Outputs (truth set per step)
Written into `run_artifacts/` for each step:
- `effective_config.json`
- `tool_invocation.json`
- `metrics.json`
- `stage_report.json`
- `execution_record.json`

## Non-goals
- Planning or tool selection
- Process spawning or runtime backend logic
- Domain semantics (owned by planners and domains)
