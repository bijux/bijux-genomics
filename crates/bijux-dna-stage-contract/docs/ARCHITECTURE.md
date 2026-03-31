# Architecture

## Modules
- `stage_plan/` owns stage-plan models, JSON projections, decision reasons, and execution-step projections.
- `execution_plan/` owns plan models, edges, canonicalization helpers, and validation.
- `executor_registry/` owns executor vocabulary, catalog entries, and lookup helpers.
- `plan_run/` owns run-plan assembly, artifact schema mapping, and planner-contract views.
- `stage_plugin.rs` remains the focused plugin contract surface.

## Data flow
- Planner → stage plan types.
