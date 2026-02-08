# Architecture

## Modules
- v1 public API
- internal handlers for wiring

## Handler responsibilities
- Translate request args into v1 API structs.
- Call planner/engine/report helpers in `bijux_dna_api::v1::api::*`.
- Marshal results into response payloads.
- Avoid owning schema versions or effectful execution.

## Data flow
- Accepts requests, delegates to planners/engine, returns results.
