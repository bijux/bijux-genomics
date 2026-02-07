# bijux-api

## What this crate does
Public API surface for plan/dry-run/execute/report/explain endpoints.

## What it must not do (boundaries)
No tool selection or execution effects directly.

## Role in the stack
Upstream: CLI/external clients. Downstream: planners + engine + runtime.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/API.md`, `docs/ENDPOINT_GUIDES.md`, `docs/API_STABILITY.md`, `docs/BOUNDARIES.md`, `docs/CHANGE_RULES.md`.

## Key contracts it owns/consumes
API schemas and responses; snapshots in tests.

## Effects & determinism guarantees
Coordinates orchestrator; no direct process spawn. See `docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/api_stability.rs`, `tests/explain_roundtrip.rs`, `tests/contract_spine.rs`, `tests/public_surface.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
