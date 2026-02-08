# bijux-infra

## What this crate does
Small deterministic utilities (logging, formats, paths) with zero domain semantics.

## What it must not do (boundaries)
No domain catalogs, SSOT ownership, or execution dependencies.
No duplicate canonicalization (enforced by `crates/bijux-policies/tests/surface/no_duplicate_canonicalizers.rs`).

## Allowed responsibilities
Allowed utilities (and only these):
- formats (JSON/TOML/YAML for config compatibility)
- logging (stable field conventions only)
- paths (deterministic path helpers)

Explicitly forbidden:
- any domain semantics
- contract schema ownership

## Role in the stack
Upstream: core/runtime/etc. Downstream: all crates as helpers.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/NO_DOMAIN.md`, `docs/LOGGING.md`, `docs/PATHS.md`, `docs/WHY_YAML.md`, `docs/CHANGE_RULES.md`.

## Key contracts it owns/consumes
None; utility-only.

## Effects & determinism guarantees
No process/network effects; deterministic helpers only. See `docs/EFFECTS.md` and the golden tests below.

## No duplicate canonicalizers
Canonicalization lives in bijux-core only. Infra must not re-implement it.
See `crates/bijux-policies/tests/surface/no_duplicate_canonicalizers.rs`.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/determinism.rs`, `tests/guardrails.rs`,
`tests/guardrails/no_generic_helpers.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
