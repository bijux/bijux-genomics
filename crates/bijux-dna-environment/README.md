# bijux-dna-environment

## What this crate does
Deterministic resolution of tool images and environment specs.

## What it must not do (boundaries)
No tool execution or container runs.

## Role in the stack
Upstream: API/planners. Downstream: runner.

## Public API / entrypoints
See `crates/bijux-dna-environment/docs/INDEX.md`, `crates/bijux-dna-environment/docs/ENV_REFERENCE.md`, `crates/bijux-dna-environment/docs/ENV_MATRIX.md`, `crates/bijux-dna-environment/docs/SCHEMAS.md`, `crates/bijux-dna-environment/docs/BOUNDARY.md`, `crates/bijux-dna-environment/docs/CHANGE_RULES.md`.

## Resolution precedence (inputs → spec → digest → cache)
Inputs flow in a strict order:
1. Inputs
2. Spec normalization
3. Digest resolution
4. Cache (resolved result reused)

Authoritative rules live in `crates/bijux-dna-environment/docs/ENV_REFERENCE.md`.

Example (tag pinned to digest, then cached):
```
tool_image_spec.json -> resolve to digest -> cached resolved spec
```

## Key contracts it owns/consumes
Resolved environment specs and digests only.

## Artifacts / Contracts
See schema fixtures under `tests/fixtures/env_schema/` and `crates/bijux-dna-environment/docs/SCHEMAS.md`.

## Effects & determinism guarantees
Pure resolution; no network execution. Stable digest for the same inputs; see `crates/bijux-dna-environment/docs/THREAT_MODEL.md` for stability breakers.

## What's deliberately NOT supported yet
- Network pulls or remote registry probing.
- HPC scheduler integration.

## No execution
This crate must not depend on `bijux-dna-runner` or execute tools. See `crates/bijux-dna-environment/docs/BOUNDARY.md` and `tests/boundaries/guardrails/no_runner_usage.rs`.

## Common failures
- Bad platform spec (schema mismatch): see `tests/schemas/schema/schema_snapshots.rs`.
- Missing tool image spec: see `tests/contracts/matrix/reference_matrix.rs`.

## How to run its tests
See `crates/bijux-dna-environment/docs/TESTS.md`. High-signal targets: `tests/contracts/matrix/reference_matrix.rs`, `tests/contracts/resolve_runtime.rs`, `tests/schemas/schema/schema_snapshots.rs`, `tests/boundaries/guardrails/guardrails_runtime.rs`, `tests/contracts/matrix/docs_reference_matrix.rs`.

## Where the docs live
Start at `crates/bijux-dna-environment/docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `crates/bijux-dna-environment/docs/CHANGE_RULES.md`.

## Where to start
- `src/runtime_spec/mod.rs`
- `src/resolve/mod.rs`
- `src/build/mod.rs`
