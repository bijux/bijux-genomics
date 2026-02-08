# bijux-domain-bam

## What this crate does
BAM domain truth: phase model, params, metric semantics, and invariants.

## What it must not do (boundaries)
No selection or execution. This crate contains no runner/env/tooling logic.
Purity is enforced by `tests/contracts/purity.rs`.

## Role in the stack
Upstream: core IDs. Downstream: planners/stages/analyze.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/PHASES.md`, `docs/METRICS.md`, `docs/METRICS_GLOSSARY.md`,
`docs/PARAMS.md`, `docs/DOMAIN_MODEL.md`, `docs/CHANGE_RULES.md`.

## Most important docs
- `docs/PHASES.md`
- `docs/METRICS.md`
- `docs/METRICS_GLOSSARY.md`
- `docs/INTERPRETATION.md`

## v1 scope
Includes pre/core/downstream phase params and the BAM metrics surfaced in `docs/METRICS.md`.

## Key contracts it owns/consumes
Domain JSON shapes and fixtures.

## Artifacts / Contracts
See `docs/DOMAIN_MODEL.md` and the snapshot fixtures under `tests/fixtures/`.

## Effects & determinism guarantees
Pure data/validation. See `docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/invariants/phase_semantics.rs`,
`tests/contracts/metrics_contract.rs`, `tests/contracts/canonical_serialization.rs`,
`tests/reference_suite/reference_suite.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Start here in code
`src/pipeline_contract.rs` → `src/stage_specs/*` → `src/metrics/*` → `src/invariants/*`.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
