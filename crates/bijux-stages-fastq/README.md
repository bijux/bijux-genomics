# bijux-stages-fastq

## What this crate does
FASTQ stage specs + observers only (parsing/metrics).

## What it must not do (boundaries)
No command assembly or tool selection.

## Role in the stack
Upstream: domain contracts. Downstream: planners/analyze.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/STAGE_LIST.md`, `docs/STAGE_CONTRACTS.md`, `docs/OBSERVERS.md`, `docs/TOOL_ROSTER.md`, `docs/CHANGE_RULES.md`.

## Stages and observers
Stage list is authoritative in `docs/STAGE_LIST.md`. Observers map input artifacts → outputs
documented under `docs/OBSERVERS.md`.

| Stage | Observer Inputs → Outputs |
| --- | --- |
| validate_pre | FASTQ → report.json |
| trim | FASTQ → trimmed FASTQ |
| merge | paired FASTQ → merged FASTQ |
| filter | FASTQ → filtered FASTQ |
| screen | FASTQ → screened FASTQ |
| qc_post | FASTQ → qc report |
| stats_neutral | FASTQ → stats report |
| correct | FASTQ → corrected FASTQ |
| umi | FASTQ → umi FASTQ |

## Key contracts it owns/consumes
Stage report/metrics shape snapshots.

## Effects & determinism guarantees
Pure parsing; deterministic snapshots. See `docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/contracts/contract_snapshots.rs`, `tests/observer/observer_determinism.rs`, `tests/contracts/symmetry.rs`, `tests/contracts/registry_completeness.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Start here in code
`src/plugin.rs`, then `src/stage_specs.rs`, then `src/observer/parse.rs`.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
