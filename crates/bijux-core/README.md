# bijux-core

## What this crate does
Owns the stable contract surface for Bijux: execution graph/manifest contracts, IDs/newtypes, and the metrics registry. It is the single source of truth (SSOT) for contract serialization and hashing rules.

## What it must not do (boundaries)
Must not select tools, assemble commands, spawn processes, or touch runtime/runner effects. This crate defines contracts only; execution belongs elsewhere.

## Public API / entrypoints
Exports the contract types and prelude described in `docs/PUBLIC_API.md` and the contract atlas in `docs/CONTRACTS.md`.

## Key contracts it owns/consumes
Owns: `ExecutionGraph`, `RunManifest`, IDs, metrics registry. Consumes: none (foundational). See `docs/SSOT.md` and `docs/SERIALIZATION.md`.

## Effects & determinism guarantees
No effects allowed beyond pure serialization. Canonical JSON and hashing are deterministic. See `docs/EFFECTS.md`.

## How to run its tests
See `docs/TESTS.md`. Key tests: `tests/execution_graph_validate.rs`, `tests/canonicalization.rs`, `tests/public_api_lock.rs`, `tests/public_module_tree.rs`.

## Where the docs live
Start at `docs/INDEX.md` and read `docs/CONTRACTS.md`, `docs/SERIALIZATION.md`, `docs/INVARIANTS.md`, and `docs/CHANGE_RULES.md`.

## Artifacts / Contracts
Produces canonical JSON for contracts; no runtime artifacts. Contract examples live in `tests/fixtures/`.

## Failure modes
Violations show up as contract validation errors or snapshot failures; inspect the test referenced above and `docs/INVARIANTS.md`.

## Stability
This is a stable SSOT crate. Breaking changes require contract versioning and snapshot updates per `docs/CHANGE_RULES.md`.
