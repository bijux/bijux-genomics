# bijux-pipelines

## What this crate does
Scientific pipeline presets and profiles with defaults ledger.

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
Pipeline IDs are defined in `src/registry/*` and must be unique. Uniqueness is enforced by
`tests/profiles/pipeline_ids_unique.rs` and registry snapshots.

## What it must not do (boundaries)
No execution or tool selection.

## Role in the stack
Upstream: domain contracts. Downstream: planners/analyze.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/PIPELINES.md`, `docs/PIPELINE_MODEL.md`, `docs/DEFAULTS_LEDGER.md`,
`docs/PIPELINE_VERSIONING.md`, `docs/CHANGE_RULES.md`.

## Key contracts it owns/consumes
Defaults ledger and profile snapshots.

## Artifacts / Contracts
See `docs/DEFAULTS_LEDGER.md`, registry snapshots in `tests/snapshots/`, and `docs/PIPELINE_MODEL.md`.

## Effects & determinism guarantees
Pure data only; deterministic ordering. See `docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/registry.rs`, `tests/defaults.rs`,
`tests/profiles.rs`, `tests/guardrails.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Start here in code
`src/registry/id.rs` → `src/registry/mod.rs` → `src/fastq/profiles.rs`

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
