# Example Plan (Annotated)

See `EXAMPLE_PLAN.json` for the raw example.

## Field notes
- `contract_version`: version of the contract (breaking change = major bump).
- `schema_version`: schema name for the serialized plan.
- `plan_id`: unique ID for this plan instance.
- `pipeline_id`: registry ID owned by `bijux-pipelines`.
- `planner_id`: planner that produced the plan.
- `steps`: ordered list of planned stage steps.
- `step_id`: unique step identifier (often equal to `stage_id`).
- `stage_id`: canonical stage identifier from domain contracts.
- `tool_id`/`tool_version`: selected tool metadata (planning only).
- `inputs`/`outputs`: artifact IDs + relative paths for each step.
- `params`: explicit parameters for the step (no hidden defaults).
