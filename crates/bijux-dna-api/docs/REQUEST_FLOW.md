# Request Flow

## Goal
Make handler wiring explicit and prevent handler sprawl.

## Flow (v1)
1. Parse request into v1 request types.
2. Map request to planner inputs.
3. Call planners to build execution plans.
4. Hand plan to engine for validation + execution.
5. Persist runtime artifacts (manifest, record, provenance).
6. Return v1 response schema.

## Example: `plan`
Input:
- `PlanRequest` (pipeline id + inputs)

Mapping:
- Load tool registry
- Build an execution graph via planners

Artifacts:
- `ExecutionGraph` (plan)
- `PlanResponse` (stable schema)

## Boundaries
- API owns request/response types and schema stability.
- Engine owns execution, runtime owns artifacts.
- `src/internal/*` is wiring only and may change at any time.
