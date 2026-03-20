# bijux-dna-stages-fastq

## What this crate does
FASTQ stage contracts plus observer-side parsing/metrics helpers.

## What it must not do (boundaries)
No command assembly or tool selection.

## Role in the stack
Upstream: domain contracts. Downstream: planners/analyze.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/STAGE_LIST.md`, `docs/STAGE_CONTRACTS.md`, `docs/OBSERVERS.md`, `docs/TOOL_ROSTER.md`, `docs/CHANGE_RULES.md`.

## Observer boundaries
Observers parse known tool outputs into metrics.
- Parsed: fixtures and documented output formats per tool.
- Ignored: extra/unknown fields that are not part of the contract.
- Required: the fields listed in `docs/STAGE_CONTRACTS.md`.

## Stages and observers
`contract_stage_ids()` publishes the full FASTQ contract surface.
`implemented_stages()` is the narrower set with observer-specialized runtime interpretation in this
crate.
`observer_stage_ids()` is the narrower set with parser-specialized observer coverage documented under
`docs/OBSERVERS.md`.

| Stage | Observer Inputs → Outputs |
| --- | --- |
| fastq.validate_reads | FASTQ validator output → validation metrics |
| fastq.profile_read_lengths | seqkit/fastp/prinseq output → length metrics |
| fastq.detect_adapters | FastQC output → adapter evidence metrics |
| fastq.profile_overrepresented_sequences | FastQC/seqkit output → sequence evidence metrics |
| fastq.profile_reads | seqkit stats output → read/base metrics |
| fastq.report_qc | MultiQC output → QC aggregation metrics |

## Key contracts it owns/consumes
Stage report/metrics shape snapshots.

## Artifacts / Contracts
See `docs/STAGE_CONTRACTS.md`, `docs/OBSERVERS.md`, and snapshots under `tests/snapshots/`.

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
