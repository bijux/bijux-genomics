# Developer Workflow

## Add a stage
1. Add stage spec + contract in the stage crate.
2. Add observer parsing + fixtures.
3. Wire tool selection in the planner.
4. Update pipeline profiles if necessary.

## Add a tool
1. Add adapter in planner.
2. Add tool to roster docs with rationale.
3. Ensure stage contract lists applicability.

## Add a metric
1. Define metric in domain crate.
2. Update observer parser.
3. Update report schema.

## Avoid SSOT and purity violations
- IDs live only in `bijux-core`.
- Planners select tools; stages parse outputs only.
- Execution happens only in runner backends.
