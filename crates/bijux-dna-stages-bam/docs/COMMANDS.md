# Commands

This file is the SSOT for callable operations managed by `bijux-dna-stages-bam`.
The crate owns Rust operations, not CLI commands. CLI parsing and command
routing belong outside this crate.

## Managed Stage Operations

| Operation | Rust entrypoint | Purpose |
| --- | --- | --- |
| `list-bam-stages` | `implemented_stages` | Return the BAM stage registry exposed by this crate. |
| `check-bam-stage-support` | `BamStagePlugin::handles_stage` | Accept only registered BAM stage IDs. |
| `materialize-bam-stage` | `BamStagePlugin::materialize` | Validate the planned BAM stage and nonblank command template, then return a stage invocation without choosing tools. |
| `parse-bam-stage-outputs` | `BamStagePlugin::parse_outputs` | Parse existing output artifacts into a plugin output envelope. |
| `collect-bam-metrics` | `metrics::bam_metrics_from_dir` | Discover supported BAM output files and aggregate `BamMetricsV1`. |
| `parse-bam-observer-output` | `observer::*` | Parse supported BAM tool output formats through domain-owned parsers. |

## CLI Commands

None. This crate owns no binaries, subcommands, shell entrypoints, process
execution surfaces, or environment commands.

## Operation Preconditions

- `materialize-bam-stage` accepts only registered BAM stage IDs.
- `materialize-bam-stage` requires a planner-provided command template with at
  least one nonblank argument.
- `parse-bam-stage-outputs` reads existing output files only; it does not create
  artifacts or execute tools.
- `collect-bam-metrics` must be deterministic for equivalent output artifact
  sets regardless of artifact ordering.

## Commands Owned Elsewhere

- User-facing CLI commands belong in command/API crates.
- Pipeline composition belongs in planner and pipeline crates.
- Tool execution belongs in runner/runtime crates.
- Environment and container commands belong in environment crates.

## Validation
Run:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stages-bam --test boundaries --no-default-features
```
