# bijux-policies

## What this crate does
Policy constitution: enforces boundaries, SSOT ownership, docs spine, and tree contracts.

## What it must not do (boundaries)
No product logic; only diagnostics and policy assertions.

## Role in the stack
Upstream: CI/dev workflows. Downstream: entire workspace (policy enforcement).

## Public API / entrypoints
See `docs/INDEX.md`, `docs/POLICY_DIAGNOSTICS.md`, `docs/EXCEPTIONS.md`, `docs/EVOLUTION.md`, `docs/POLICY_MATRIX.md`, `docs/CHANGE_RULES.md`.

## Key contracts it owns/consumes
Policy diagnostics and snapshots only.

## Effects & determinism guarantees
Reads repo state and cargo metadata only; no network or process execution. See `docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/deps/dependency_graph.rs`, `tests/surface/docs_spine.rs`, `tests/surface/no_helpers_policy.rs`, `tests/surface/ssot_catalog_authority.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
