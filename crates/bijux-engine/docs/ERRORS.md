# Errors

## Taxonomy
Engine errors are categorized as:
- Planning
- Execution
- Observation
- Validation
- Contract (contract enforcement failures)

## Guaranteed context
The engine guarantees the following context fields when present:
- `stage_id` (contract errors)
- `artifact_id` (contract errors)
- `path` (contract errors)

Runner backend context:
- The engine does not attach runner backend metadata to error variants today.
- The canonical runner backend is recorded in `tool_invocation.json` as part of the truth set.

## Contract errors
Contract errors are emitted when required artifacts are missing or invalid.
Guaranteed fields:
- `stage_id`
- `artifact_id`
- `path`

## Execution errors
Execution errors wrap runner failures and orchestration failures.
Guaranteed fields:
- none beyond the error message; consult `tool_invocation.json` for runner backend context.
