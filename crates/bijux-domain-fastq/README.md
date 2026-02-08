# bijux-domain-fastq

## What this crate does
FASTQ domain truth: IDs, params, metric semantics, and invariant verdicts.

## What it must not do (boundaries)
No selection or execution.

## Role in the stack
Upstream: core IDs. Downstream: planners/stages/analyze.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/DOMAIN_MODEL.md`, `docs/METRICS.md`, `docs/PARAMS.md`, `docs/STAGES.md`, `docs/CHANGE_RULES.md`.

## Domain truth set (SSOT)
The FASTQ domain defines:
- IDs: stage IDs, tool IDs, and bank IDs (see `docs/DOMAIN_MODEL.md`).
- Params: canonical JSON shapes and defaults (see `docs/PARAMS.md`).
- Metric semantics: meaning + ordering rules (see `docs/METRICS.md`).
- Invariants: verdict rules and thresholds (see `docs/DOMAIN_MODEL.md`).

## Key contracts it owns/consumes
Owns FASTQ IDs, params, metrics semantics, and invariant verdicts.

## Artifacts / Contracts
See `docs/DOMAIN_MODEL.md` and fixtures under `tests/fixtures/`.

## Retention semantics (must be explicit)
Retention is always a stage-boundary ratio with explicit numerator/denominator scope.
"Naked retention" (a bare percentage without scope) is forbidden.
See `tests/semantics/retention_truth_table.rs` and `docs/DOMAIN_MODEL.md`.

## Banks
FASTQ banks live under `src/banks/*` and are SSOT for adapter/contaminant/polyX lists.
Selection rules live in `src/banks/selection.rs` and must not be overridden elsewhere.
Provenance + versioning rules are in `docs/BANKS.md`.

## Effects & determinism guarantees
Pure data/validation. See `docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/semantics/retention_semantics.rs`,
`tests/semantics/params_canonical.rs`, `tests/semantics/retention_truth_table.rs`,
`tests/purity/determinism.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Start here in code
`src/stages/ids.rs` → `src/params/*` → `src/metrics/*` → `src/invariants/*`.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
