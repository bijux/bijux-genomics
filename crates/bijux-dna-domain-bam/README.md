# bijux-dna-domain-bam

## What this crate does
BAM domain truth: ordered stage model, params, metric semantics, and invariants.

## What it must not do (boundaries)
No selection or execution. This crate contains no runner/env/tooling logic.
Purity is enforced by `tests/boundaries/purity.rs`.

## Role in the stack
Upstream: core IDs. Downstream: planners/stages/analyze.

## Public API / entrypoints
See `crates/bijux-dna-domain-bam/docs/INDEX.md`, `crates/bijux-dna-domain-bam/docs/PHASES.md`, `crates/bijux-dna-domain-bam/docs/METRICS.md`, `crates/bijux-dna-domain-bam/docs/METRICS_GLOSSARY.md`,
`crates/bijux-dna-domain-bam/docs/PARAMS.md`, `crates/bijux-dna-domain-bam/docs/DOMAIN_MODEL.md`, `crates/bijux-dna-domain-bam/docs/CHANGE_RULES.md`.

## Most important docs
- `crates/bijux-dna-domain-bam/docs/PHASES.md`
- `crates/bijux-dna-domain-bam/docs/METRICS.md`
- `crates/bijux-dna-domain-bam/docs/METRICS_GLOSSARY.md`
- `crates/bijux-dna-domain-bam/docs/INTERPRETATION.md`

## v1 scope
Includes pre/core/downstream parameter groups and the BAM metrics surfaced in `crates/bijux-dna-domain-bam/docs/METRICS.md`.

## Key contracts it owns/consumes
Domain JSON shapes and fixtures.

## Artifacts / Contracts
See `crates/bijux-dna-domain-bam/docs/DOMAIN_MODEL.md`, `src/stage_specs/`, and the snapshot fixtures under `tests/snapshots/`.

## Effects & determinism guarantees
Pure data/validation. See `crates/bijux-dna-domain-bam/docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `crates/bijux-dna-domain-bam/docs/TESTS.md`. Golden tests: `tests/semantics/invariants/phase_semantics.rs`,
`tests/contracts/metrics_contract.rs`, `tests/contracts/canonical_serialization.rs`,
`tests/contracts/reference_suite/reference_suite.rs`.

## Where the docs live
Start at `crates/bijux-dna-domain-bam/docs/INDEX.md` and follow the crate docs listed above.

## Start here in code
`src/pipeline_contract.rs` → `src/stage_specs/*` → `src/metrics/*` → `src/invariants/*`.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `crates/bijux-dna-domain-bam/docs/CHANGE_RULES.md`.
