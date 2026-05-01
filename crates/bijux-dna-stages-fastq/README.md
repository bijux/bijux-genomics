# bijux-dna-stages-fastq

`bijux-dna-stages-fastq` owns FASTQ stage contract exports, observer-side
parsing, stage metrics normalization, and stage-plugin output envelopes for
already-planned FASTQ stages.

This crate follows repository governance documentation. `README.md` and
`README.md`; re-read those files before editing this child
repository and before committing.

## What this crate does

This crate owns FASTQ stage contract exports, observer-side parsing, stage
metrics normalization, and stage-plugin output envelopes for already-planned
FASTQ stages.

## Boundary

This crate does not plan workflows, choose tools, assemble shell commands,
execute processes, manage environments, or expose CLI commands. Planners and
runtime-facing callers may consume this crate; this crate must not call back
into planner, runner, engine, API, CLI, or environment layers.

## Public Surface

- `FastqStagePlugin`
- `StagePlanJson`
- `contract_stage_ids`
- `closed_execution_stage_ids`
- `implemented_stages`
- `observer_specialized_stage_ids`
- `observer_stage_ids`
- `observer_stage_tool_bindings`
- `runtime_interpretation_for_stage`
- `runtime_interpretation_for_stage_tool`
- `runtime_interpretation_stage_ids`
- `RuntimeInterpretationLevel`
- `contracts`
- `metrics`
- `observer`
- `stage_specs`

`docs/COMMANDS.md` is the SSOT for callable operations managed by this crate.

## Release Example

Run the release-surface example from the repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo run -q -p bijux-dna-stages-fastq --example fastq_release_surface
```

The example prints the governed FASTQ implemented-stage set and observer tool
bindings used in essential QC release flows.

Managed operations:

- `list-fastq-contract-stages`
- `list-fastq-implemented-stages`
- `list-fastq-observer-stages`
- `classify-fastq-runtime-interpretation`
- `check-fastq-stage-support`
- `materialize-fastq-stage`
- `parse-fastq-stage-outputs`
- `build-fastq-metrics-envelope`
- `parse-fastq-observer-output`
- `write-fastq-observer-artifact`

## Documentation

The crate root intentionally has only this `README.md`. All other docs live
under `docs/`, with a 10-document allowance:

- `docs/ARCHITECTURE.md`
- `docs/BOUNDARY.md`
- `docs/CHANGE_RULES.md`
- `docs/COMMANDS.md`
- `docs/DEPENDENCIES.md`
- `docs/EFFECTS.md`
- `docs/INDEX.md`
- `docs/PUBLIC_API.md`
- `docs/STAGE_CONTRACTS.md`
- `docs/TESTS.md`

## Tests

Run from the `bijux-genomics` repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo check -p bijux-dna-stages-fastq --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stages-fastq --no-default-features
```
