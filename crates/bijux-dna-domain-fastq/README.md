# bijux-dna-domain-fastq

## What this crate does
FASTQ domain truth: IDs, params, metric semantics, and invariant verdicts.

## What it must not do (boundaries)
No selection or execution.

## Role in the stack
Upstream: core IDs. Downstream: planners/stages/analyze.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/DOMAIN_MODEL.md`, and `docs/PUBLIC_API.md`.
Command inventory is authoritative in `docs/COMMANDS.md`.

## Domain truth set (SSOT)
The FASTQ domain defines:
- IDs: stage IDs, tool IDs, and bank IDs (see `docs/DOMAIN_MODEL.md`).
- Params: canonical JSON shapes and defaults (see `docs/DOMAIN_MODEL.md`).
- Metric semantics: meaning + ordering rules (see `docs/DOMAIN_MODEL.md`).
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
Provenance + versioning rules are in `docs/DOMAIN_MODEL.md`.

## Effects & determinism guarantees
Pure data/validation. See `docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/semantics/retention_semantics.rs`,
`tests/semantics/params_canonical.rs`, `tests/semantics/retention_truth_table.rs`,
`tests/purity/determinism.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Start here in code
`src/stages/ids.rs` → `src/params/` → `src/metrics/` → `src/invariants/`.

## Module layout
- `src/artifacts/` owns governed stage report and manifest types.
- `src/observer/` owns parser behavior and observer specialization contracts.
- `src/pipeline_contract/` owns pipeline ordering and graph assembly.
- `src/execution_support/` and `src/stage_tool_governance/` own manifest-backed readiness catalogs.
- `src/bench/` owns benchmark query metadata and repository interfaces.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CONTRACTS.md`.
