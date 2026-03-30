# bijux-dna-domain-fastq

## What this crate does
FASTQ domain truth: IDs, params, metric semantics, and invariant verdicts.

## What it must not do (boundaries)
No selection or execution.

## Role in the stack
Upstream: core IDs. Downstream: planners/stages/analyze.

## Public API / entrypoints
See `crates/bijux-dna-domain-fastq/docs/INDEX.md`, `crates/bijux-dna-domain-fastq/docs/DOMAIN_MODEL.md`, `crates/bijux-dna-domain-fastq/docs/METRICS.md`, `crates/bijux-dna-domain-fastq/docs/PARAMS.md`, `crates/bijux-dna-domain-fastq/docs/STAGES.md`, `crates/bijux-dna-domain-fastq/docs/CHANGE_RULES.md`.

## Domain truth set (SSOT)
The FASTQ domain defines:
- IDs: stage IDs, tool IDs, and bank IDs (see `crates/bijux-dna-domain-fastq/docs/DOMAIN_MODEL.md`).
- Params: canonical JSON shapes and defaults (see `crates/bijux-dna-domain-fastq/docs/PARAMS.md`).
- Metric semantics: meaning + ordering rules (see `crates/bijux-dna-domain-fastq/docs/METRICS.md`).
- Invariants: verdict rules and thresholds (see `crates/bijux-dna-domain-fastq/docs/DOMAIN_MODEL.md`).

## Key contracts it owns/consumes
Owns FASTQ IDs, params, metrics semantics, and invariant verdicts.

## Artifacts / Contracts
See `crates/bijux-dna-domain-fastq/docs/DOMAIN_MODEL.md` and fixtures under `tests/fixtures/`.

## Retention semantics (must be explicit)
Retention is always a stage-boundary ratio with explicit numerator/denominator scope.
"Naked retention" (a bare percentage without scope) is forbidden.
See `tests/semantics/retention_truth_table.rs` and `crates/bijux-dna-domain-fastq/docs/DOMAIN_MODEL.md`.

## Banks
FASTQ banks live under `src/banks/*` and are SSOT for adapter/contaminant/polyX lists.
Selection rules live in `src/banks/selection.rs` and must not be overridden elsewhere.
Provenance + versioning rules are in `crates/bijux-dna-domain-fastq/docs/BANKS.md`.

## Effects & determinism guarantees
Pure data/validation. See `crates/bijux-dna-domain-fastq/docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `crates/bijux-dna-domain-fastq/docs/TESTS.md`. Golden tests: `tests/semantics/retention_semantics.rs`,
`tests/semantics/params_canonical.rs`, `tests/semantics/retention_truth_table.rs`,
`tests/purity/determinism.rs`.

## Where the docs live
Start at `crates/bijux-dna-domain-fastq/docs/INDEX.md` and follow the crate docs listed above.

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
Contract and behavior changes follow `crates/bijux-dna-domain-fastq/docs/CHANGE_RULES.md`.
