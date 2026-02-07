# bijux-stage-contract

## What this crate does
Defines this crate's core responsibilities and wiring.

## What it must not do (boundaries)
Must only depend on approved crates; must not reach into execution or domain logic unless explicitly allowed in docs.

## Public API / entrypoints
See `docs/INDEX.md` for stable entrypoints and re-exports.

## Key contracts it owns/consumes
See `docs/INDEX.md` for contract ownership and consumption details.

## Effects & determinism guarantees
See `docs/EFFECTS.md` for allowed effects and determinism guarantees.

## Artifacts / Contracts
None by default unless documented in `docs/ARCHITECTURE.md`.

## Failure modes
See crate logs/tests; start with `docs/TESTS.md` for debugging paths.

## How to run its tests
See `docs/TESTS.md`.

## Where the docs live
- `docs/INDEX.md`
- `docs/SCOPE.md`
- `docs/ARCHITECTURE.md`
- `docs/EFFECTS.md`
- `docs/CHANGE_RULES.md`
