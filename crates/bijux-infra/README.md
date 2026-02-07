# bijux-infra

## What this crate does
Provides small, deterministic utilities (logging, formats, paths). It must remain free of domain semantics.

## What it must not do (boundaries)
Must not depend on domain/stage/planner crates or define catalogs. See `docs/NO_DOMAIN.md`.

## Public API / entrypoints
Utilities are documented in `docs/LOGGING.md` and `docs/PATHS.md`.

## Key contracts it owns/consumes
Consumes core canonicalization; does not define its own hashing rules. See `docs/WHY_YAML.md` for format boundaries.

## Effects & determinism guarantees
No process or network effects. Determinism is enforced by `tests/determinism.rs`.

## How to run its tests
See `docs/TESTS.md`. Key tests: `tests/determinism.rs`, `tests/docs_canonical_owner.rs`, `tests/guardrails.rs`.

## Where the docs live
Start at `docs/INDEX.md`, then read `docs/NO_DOMAIN.md`, `docs/LOGGING.md`, and `docs/PATHS.md`.

## Artifacts / Contracts
No runtime artifacts; utility-only.

## Failure modes
Violations are caught by policy tests and determinism checks.

## Stability
Public APIs are small and stable; changes follow `docs/CHANGE_RULES.md`.
