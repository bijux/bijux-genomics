# bijux-api

## What this crate does
Defines the public API surface for planning, executing, and reporting pipelines. It is the orchestration entrypoint for consumers.

## What it must not do (boundaries)
Must not implement tool selection logic or execution effects directly. It wires planners, engine, and runtime only.

## Public API / entrypoints
The v1 API is documented in `docs/API.md` and `docs/ENDPOINT_GUIDES.md`.

## Key contracts it owns/consumes
Consumes planner outputs and runtime manifests; produces API responses and explainability payloads.

## Effects & determinism guarantees
No direct process spawn. Determinism is enforced by schema snapshot tests; see `docs/API_STABILITY.md`.

## How to run its tests
See `docs/TESTS.md`. Key tests: `tests/api_stability.rs`, `tests/explain_roundtrip.rs`, `tests/contract_spine.rs`.

## Where the docs live
Start at `docs/INDEX.md`, then read `docs/API.md`, `docs/ENDPOINT_GUIDES.md`, and `docs/API_STABILITY.md`.

## Artifacts / Contracts
Produces plan/execute/dry-run responses and explain payloads; see schema snapshots in `tests/snapshots/`.

## Failure modes
Validation and schema mismatches are caught by stability tests; see `docs/API_STABILITY.md`.

## Stability
API schemas are snapshot-tested; breaking changes require updates per `docs/CHANGE_RULES.md`.
