# Engine Error Taxonomy

## ContractError
Returned when declared contract requirements are violated. Guarantees:
- Includes the `step_id`
- Includes the `artifact_id` (if applicable)
- Includes the path of the missing/invalid artifact

## ToolError
Returned when a tool exits non-zero.

## PlanError
Returned when the plan or execution graph is invalid.

## InfraError
Returned for filesystem or runtime infrastructure failures.

## ParseError
Returned when required metrics or reports cannot be parsed.
