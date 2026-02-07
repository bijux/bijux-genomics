# ERROR_TAXONOMY

## ContractError
- User action: fix missing/empty artifacts or invalid metrics.
- Developer action: ensure planner declares outputs correctly.
- Inspect: step directory, manifest, stage_report.

## ToolError
- User action: re-run or inspect tool stderr.
- Developer action: review tool adapter and params.
- Inspect: execution_record, tool_invocation.

## ValidationError
- User action: check pipeline/profile inputs.
- Developer action: tighten validators.
- Inspect: graph JSON, plan response.
