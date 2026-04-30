# bijux-dna-stages-bam

`bijux-dna-stages-bam` owns BAM stage contract exports, observer parser exports,
and deterministic metrics-envelope materialization for already-planned BAM
stages.

This crate follows repository governance documentation. `README.md` and
`README.md`; re-read those files before editing this child
repository and before committing.

## What this crate does

This crate owns BAM stage contract exports, observer parser exports, and
deterministic metrics-envelope materialization for already-planned BAM stages.

## Boundary

This crate does not plan workflows, choose tools, assemble shell commands,
execute processes, manage environments, or expose CLI commands. Planners and
runtime crates call into this crate; this crate must not call back into those
layers.

## Public Surface

- `BamStagePlugin`: stage plugin implementation for registered BAM stage IDs.
- `StagePlanJson`: public JSON stage-plan shape from `bijux-dna-stage-contract`.
- `implemented_stages()`: ordered BAM stage registry mirrored from
  `bijux-dna-domain-bam`.
- `metrics`: deterministic BAM metric discovery from existing output files.
- `observer`: parser re-exports for supported BAM tool output formats.
- `stage_specs`: planner-facing BAM domain vocabulary re-exports.

`docs/COMMANDS.md` is the SSOT for callable operations managed by this crate.

## Release Example

Run the release-surface example from the repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo run -q -p bijux-dna-stages-bam --example bam_release_surface
```

The example prints the governed BAM implemented-stage set and asserts presence
of alignment, mapping-summary, and coverage stages used in release evidence.

Managed operations:

- `list-bam-stages`
- `check-bam-stage-support`
- `materialize-bam-stage`
- `parse-bam-stage-outputs`
- `collect-bam-metrics`
- `parse-bam-observer-output`

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
CARGO_TARGET_DIR=artifacts/cargo-target cargo check -p bijux-dna-stages-bam --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stages-bam --no-default-features
```
