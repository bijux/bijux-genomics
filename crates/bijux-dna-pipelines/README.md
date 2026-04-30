# bijux-dna-pipelines

Repository policy: apply `README.md` and `README.md` before changing this crate.

## What this crate does
Defines canonical pipeline profiles, defaults ledgers, manifests, and registry lookups for FASTQ, BAM, VCF, and cross-domain handoffs.

Pipeline IDs:
- fastq-only: `fastq-to-fastq__adna__v1`, `fastq-to-fastq__amplicon_standard__v1`, `fastq-to-fastq__amplicon_umi__v1`, `fastq-to-fastq__contaminant_depletion__v1`, `fastq-to-fastq__default__v1`, `fastq-to-fastq__edna_metabarcoding__v1`, `fastq-to-fastq__host_depletion__v1`, `fastq-to-fastq__minimal__v1`, `fastq-to-fastq__qc_only__v1`, `fastq-to-fastq__reference_adna__v1`, `fastq-to-fastq__rrna_depletion__v1`, `fastq-to-fastq__trim_qc__v1`, `fastq-to-fastq__umi__v1`
- fastq → bam: `fastq-to-bam__default__v1`, `fastq-to-bam__adna_shotgun__v1`
- bam-only: `bam-to-bam__adna_capture__v1`, `bam-to-bam__adna_shotgun__v1`, `bam-to-bam__default__v1`, `bam-to-bam__reference_adna__v1`
- vcf-only: `vcf-to-vcf__minimal__v1`, `vcf-to-vcf__reference_basic__v1`

## Allowed dependencies
Pipelines may depend on core and domain contracts. Planner, engine, runner, command, stage implementation, database, environment, analysis application, and science orchestration crates stay downstream.

## Profiles
A profile selects a pipeline ID + defaults and may override specific values. Overrides are explicit and
precedence is: profile > pipeline > global.

Example precedence:
- pipeline defaults set `trim_min_len = 25`
- profile overrides to `trim_min_len = 30`
- effective value is `30`

## Defaults ledger
The defaults ledger records effective defaults, tool selections, and provenance for each pipeline.
Changes are guarded by snapshot tests; update only when the contract changes intentionally.

## Registry authority
Pipeline IDs are validated through `src/registry/pipeline_id.rs`, profile families are assembled under `src/registry/families/`, and stable query behavior lives under `src/registry/catalog/`.

## What it must not do (boundaries)
No command routing, execution, process spawning, network access, or runtime tool discovery. This crate declares deterministic contract data only.

## Role in the stack
Upstream: core and domain contracts. Downstream: planners, analysis, API, CLI, engine, runner, and runtime consumers.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/PUBLIC_API.md`, `docs/PIPELINES.md`, `docs/DEFAULTS_LEDGER.md`, `docs/DEPENDENCIES.md`, and `docs/COMMANDS.md`.

## Key contracts it owns/consumes
Owns profile contracts, defaults ledgers, manifests, deterministic registry ordering, and stable public reexports. Consumes core IDs and domain vocabulary.

## Artifacts / Contracts
See `docs/DEFAULTS_LEDGER.md`, `docs/PIPELINES.md`, registry snapshots in `tests/snapshots/`, and source-tree guards in `tests/boundaries/`.

## Effects & determinism guarantees
Pure data only; deterministic ordering and deterministic hashing. See `docs/EFFECTS.md` and the contract tests below.

## How to run its tests
See `docs/TESTS.md`. Standard command:

```bash
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-pipelines --no-default-features
```

Entry points:
- `tests/boundaries.rs`
- `tests/contracts.rs`
- `tests/guardrails.rs`
- `tests/invariant_fast.rs`

## Where the docs live
Start at `docs/INDEX.md`. The crate root intentionally keeps only this `README.md`; all other Markdown docs belong under `docs/`.

## Start here in code
- `src/public_api/` for the curated stable surface.
- `src/contract/` for pipeline profile and invariant contracts.
- `src/defaults/` for defaults ledgers, parameter envelopes, and override merging.
- `src/fastq/` for FASTQ defaults, profiles, and invariants.
- `src/bam/` for BAM profile families and invariants.
- `src/vcf/` for VCF profile families and invariants.
- `src/registry/` for pipeline id validation, profile families, registry catalog assembly, and query behavior.
- `src/cross/fastq_to_bam/` for cross-domain handoff profile families.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/PIPELINES.md` and `docs/DEFAULTS_LEDGER.md`.
