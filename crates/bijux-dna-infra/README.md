# bijux-dna-infra

## What this crate does
Small deterministic utilities (logging, formats, paths) with zero domain semantics.

## What it must not do (boundaries)
No domain catalogs, SSOT ownership, or execution dependencies.
No duplicate canonicalization (enforced by `crates/bijux-dna-policies/tests/boundaries/surface/structure_guards/no_duplicate_canonicalizers.rs`).

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
See `crates/bijux-dna-infra/docs/INDEX.md`, `crates/bijux-dna-infra/docs/NO_DOMAIN.md`, `crates/bijux-dna-infra/docs/LOGGING.md`, `crates/bijux-dna-infra/docs/PATHS.md`, `crates/bijux-dna-infra/docs/WHY_YAML.md`, `crates/bijux-dna-infra/docs/CHANGE_RULES.md`.

## Key contracts it owns/consumes
None; utility-only.

## Artifacts / Contracts
See `crates/bijux-dna-infra/docs/LOGGING.md`, `crates/bijux-dna-infra/docs/PATHS.md`, and `crates/bijux-dna-infra/docs/WHY_YAML.md` for behavioral contracts.

## Effects & determinism guarantees
No process/network effects; deterministic helpers only. See `crates/bijux-dna-infra/docs/EFFECTS.md` and the golden tests below.

## No duplicate canonicalizers
Canonicalization lives in bijux-dna-core only. Infra must not re-implement it.
See `crates/bijux-dna-policies/tests/boundaries/surface/structure_guards/no_duplicate_canonicalizers.rs`.

## How to run its tests
See `crates/bijux-dna-infra/docs/TESTS.md`. Golden tests: `tests/determinism.rs`, `tests/guardrails.rs`,
`tests/guardrails/no_generic_helpers.rs`.

## Where the docs live
Start at `crates/bijux-dna-infra/docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `crates/bijux-dna-infra/docs/CHANGE_RULES.md`.
