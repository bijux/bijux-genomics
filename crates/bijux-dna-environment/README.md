# bijux-dna-environment

## What this crate does
Deterministic resolution of tool images and environment specs.

## What it must not do (boundaries)
No tool execution or container runs.

## Role in the stack
Upstream: API/planners. Downstream: runner.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/ENV_REFERENCE.md`, `docs/ENV_MATRIX.md`, `docs/SCHEMAS.md`, `docs/BOUNDARY.md`, `docs/CHANGE_RULES.md`.

## Resolution precedence (inputs → spec → digest → cache)
Inputs flow in a strict order:
1. Inputs
2. Spec normalization
3. Digest resolution
4. Cache (resolved result reused)

Authoritative rules live in `docs/ENV_REFERENCE.md`.

Example (tag pinned to digest, then cached):
```
tool_image_spec.json -> resolve to digest -> cached resolved spec
```

## Key contracts it owns/consumes
Resolved environment specs and digests only.

## Artifacts / Contracts
See schema fixtures under `tests/fixtures/env_schema/` and `docs/SCHEMAS.md`.

## Effects & determinism guarantees
Pure resolution; no network execution. Stable digest for the same inputs; see `docs/THREAT_MODEL.md` for stability breakers.

## What's deliberately NOT supported yet
- Network pulls or remote registry probing.
- HPC scheduler integration.

## No execution
This crate must not depend on `bijux-dna-runner` or execute tools. See `docs/BOUNDARY.md` and `tests/guardrails/no_runner_usage.rs`.

## Common failures
- Bad platform spec (schema mismatch): see `tests/schema/schema_snapshots.rs`.
- Missing tool image spec: see `tests/matrix/reference_matrix.rs`.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/matrix/reference_matrix.rs`, `tests/schema/schema_snapshots.rs`, `tests/guardrails/guardrails_runtime.rs`, `tests/matrix/docs_reference_matrix.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.

## Where to start
- `src/runtime_spec.rs`
- `src/resolve.rs`
- `src/build.rs`
