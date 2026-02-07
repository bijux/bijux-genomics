# ERROR_TAXONOMY

## ContractError
When contract enforcement fails, errors must include:

- `step_id`
- `artifact_id` (when applicable)
- `path`
- `reason`

## ToolError
Used when a tool exits non-zero or emits invalid metrics.

## ValidationError
Used when the input graph or manifest fails preconditions.
