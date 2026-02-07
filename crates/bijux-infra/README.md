# bijux-infra

## What this crate does
Small deterministic utilities (logging, formats, paths) with zero domain semantics.

## What it must not do (boundaries)
No domain catalogs, SSOT ownership, or execution dependencies.
No duplicate canonicalization (see bijux-core `contract::canonical`).

## Allowed responsibilities
- Deterministic IO helpers (atomic writes, bounded reads, retries).
- Logging initialization with stable field conventions.
- Non-contract JSON/YAML parsing for config compatibility.
- Path helpers for run layout construction only.

## Role in the stack
Upstream: core/runtime/etc. Downstream: all crates as helpers.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/NO_DOMAIN.md`, `docs/LOGGING.md`, `docs/PATHS.md`, `docs/WHY_YAML.md`, `docs/CHANGE_RULES.md`.

## Key contracts it owns/consumes
None; utility-only.

## Effects & determinism guarantees
No process/network effects; deterministic helpers only. See `docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/determinism.rs`, `tests/guardrails.rs`,
`tests/guardrails/no_generic_helpers.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
