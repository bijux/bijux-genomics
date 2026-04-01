# bijux-dna-pipelines

## What this crate does
Defines canonical pipeline profiles, defaults ledgers, and registry lookups for FASTQ, BAM, VCF, and cross-domain handoffs.

Pipeline IDs:
- fastq-only: `fastq-to-fastq__default__v1`, `fastq-to-fastq__minimal__v1`, `fastq-to-fastq__adna__v1`
- fastq → bam: `fastq-to-bam__default__v1`, `fastq-to-bam__adna_shotgun__v1`
- bam-only: `bam-to-bam__default__v1`, `bam-to-bam__adna_shotgun__v1`, `bam-to-bam__adna_capture__v1`

## Allowed dependencies
Pipelines may depend on domain + planner contracts, but must not depend on engine/runner execution machinery.

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
Pipeline IDs are validated through `src/registry/pipeline_id.rs`, and registry collections stay snapshot-locked through contract tests.

## What it must not do (boundaries)
No execution or tool selection.

## Role in the stack
Upstream: domain contracts. Downstream: planners/analyze.

## Public API / entrypoints
See `crates/bijux-dna-pipelines/docs/INDEX.md`, `crates/bijux-dna-pipelines/docs/PIPELINES.md`, `crates/bijux-dna-pipelines/docs/PIPELINE_MODEL.md`, `crates/bijux-dna-pipelines/docs/DEFAULTS_LEDGER.md`,
`crates/bijux-dna-pipelines/docs/PIPELINE_VERSIONING.md`, `crates/bijux-dna-pipelines/docs/CHANGE_RULES.md`.

## Key contracts it owns/consumes
Defaults ledger and profile snapshots.

## Artifacts / Contracts
See `crates/bijux-dna-pipelines/docs/DEFAULTS_LEDGER.md`, registry snapshots in `tests/snapshots/`, and `crates/bijux-dna-pipelines/docs/PIPELINE_MODEL.md`.

## Effects & determinism guarantees
Pure data only; deterministic ordering. See `crates/bijux-dna-pipelines/docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `crates/bijux-dna-pipelines/docs/TESTS.md`. Entry points:
- `tests/boundaries.rs`
- `tests/contracts.rs`
- `tests/guardrails.rs`
- `tests/invariant_fast.rs`

## Where the docs live
Start at `crates/bijux-dna-pipelines/docs/INDEX.md` and follow the crate docs listed above.

## Start here in code
- `src/public_api/` for the curated stable surface.
- `src/contract/` for pipeline profile and invariant contracts.
- `src/defaults/` for defaults ledgers, parameter envelopes, and override merging.
- `src/registry/` for pipeline id validation, collections, and lookups.
- `src/cross/fastq_to_bam/` for the cross-domain handoff profiles.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `crates/bijux-dna-pipelines/docs/CHANGE_RULES.md`.
