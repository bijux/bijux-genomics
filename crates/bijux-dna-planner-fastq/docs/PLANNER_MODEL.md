# PLANNER_MODEL

## Planner owns
- tool selection
- execution graph construction
- explain payload generation

## Planner must not
- parse tool outputs
- implement policy scans

## Selection args
Selection inputs are modeled in `src/selection/args.rs` (e.g. `BenchFastqPreprocessArgs`).
These types are the single source of truth for selection arguments; avoid duplicating them in docs.
