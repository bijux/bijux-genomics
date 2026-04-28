# bijux-dna-infra

## What this crate does
Deterministic infrastructure helpers for filesystem IO, path construction, retry behavior, logging bootstrap, file locking, and config-compatible formats.

## What it must not do (boundaries)
No domain catalogs, SSOT ownership, or execution dependencies.
No duplicate canonicalization (enforced by `crates/bijux-dna-policies/tests/boundaries/surface/structure_guards/no_duplicate_canonicalizers.rs`).

## Allowed responsibilities
Allowed utilities (and only these):
- formats (JSON/TOML/YAML for config compatibility)
- logging (stable field conventions only)
- paths (deterministic path helpers)
- io (filesystem writes/reads/removals with explicit error taxonomy)
- retry (policy, clock abstraction, and backoff orchestration)
- run layout support (contracts plus run publish/lock helpers)
- temp and locking helpers

Explicitly forbidden:
- any domain semantics
- contract schema ownership

## Role in the stack
Upstream: core/runtime/etc. Downstream: all crates as helpers.

## Public API / entrypoints
See `docs/PUBLIC_API.md`, `docs/COMMANDS.md`, and `docs/INDEX.md`.

This crate owns no Cargo binaries or host commands.

## Key contracts it owns/consumes
None; utility-only.

## Artifacts / Contracts
See `docs/EFFECTS.md`, `docs/PATHS.md`, and `docs/FORMATS.md` for behavioral contracts.

## Effects & determinism guarantees
No process/network effects; deterministic helpers only. See `crates/bijux-dna-infra/docs/EFFECTS.md` and the golden tests below.

## No duplicate canonicalizers
Canonicalization lives in bijux-dna-core only. Infra must not re-implement it.
See `crates/bijux-dna-policies/tests/boundaries/surface/structure_guards/no_duplicate_canonicalizers.rs`.

## How to run its tests
See `crates/bijux-dna-infra/docs/TESTS.md`. Key tests: `tests/contracts/io.rs`,
`tests/contracts/run_layout.rs`, `tests/determinism/hash.rs`,
`tests/determinism/retry.rs`, and `tests/schemas/public_surface.rs`.

## Where the docs live
Root docs are limited to this `README.md`. All other crate docs live in `docs/`; start at
`docs/INDEX.md`.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/PUBLIC_API.md`, `docs/BOUNDARY.md`, and
`docs/DEPENDENCIES.md`.

## Workspace Policy
Workspace work on this crate is governed by `/Users/bijan/bijux/bijux-genomics/README.md`,
`/Users/bijan/bijux/README.md`, and `/Users/bijan/bijux/CODEX.md`; re-read
those files before editing this child repository or making commits.
