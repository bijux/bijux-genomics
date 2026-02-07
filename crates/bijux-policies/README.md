# bijux-policies

## What this crate does
Defines and enforces repository policies: dependency boundaries, SSOT ownership, effect limits, docs spine, and tree contracts. This is the policy constitution and the only place policy logic lives.

## What it must not do (boundaries)
Must not implement product logic or mutate contracts. It only reads files/metadata and reports failures.

## Public API / entrypoints
All policies are executed via tests under `tests/`. The policy registry is documented in `docs/INDEX.md` and `docs/POLICY_MATRIX.md`.

## Key contracts it owns/consumes
Owns policy rules and diagnostics. Consumes crate docs and workspace structure; see `docs/EFFECTS.md` and `docs/EXCEPTIONS.md`.

## Effects & determinism guarantees
Reads the filesystem and cargo metadata; no network or process execution. Deterministic results are enforced via snapshot tests.

## How to run its tests
See `docs/TESTS.md`. Key tests: `tests/deps/dependency_graph.rs`, `tests/surface/docs_spine.rs`, `tests/surface/no_helpers_policy.rs`, `tests/surface/ssot_catalog_authority.rs`.

## Where the docs live
Start at `docs/INDEX.md`. See `docs/POLICY_DIAGNOSTICS.md`, `docs/EXCEPTIONS.md`, and `docs/EVOLUTION.md`.

## Artifacts / Contracts
Produces policy failure diagnostics; no runtime artifacts.

## Failure modes
Policy failures point to the violating file and the remediation steps in `docs/POLICY_DIAGNOSTICS.md`.

## Stability
Policy names are stable; semantic changes require updates to `docs/CHANGE_RULES.md` and snapshots.
