# bijux-infra

## What this crate does
Small deterministic utilities (logging, formats, paths) with zero domain semantics.

## What it must not do (boundaries)
No domain catalogs or execution dependencies.

## Role in the stack
Upstream: core/runtime/etc. Downstream: all crates as helpers.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/NO_DOMAIN.md`, `docs/LOGGING.md`, `docs/PATHS.md`, `docs/WHY_YAML.md`, `docs/CHANGE_RULES.md`.

## Key contracts it owns/consumes
None; utility-only.

## Effects & determinism guarantees
No process/network effects; deterministic helpers only. See `docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/determinism.rs`, `tests/docs_canonical_owner.rs`, `tests/guardrails.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
