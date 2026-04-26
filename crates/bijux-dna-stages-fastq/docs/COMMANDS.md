# Commands

This file is the SSOT for callable operations managed by
`bijux-dna-stages-fastq`. The crate owns Rust operations, not CLI commands.
CLI parsing and command routing belong outside this crate.

## Managed Stage Operations

| Operation | Rust entrypoint | Purpose |
| --- | --- | --- |
| `list-fastq-contract-stages` | `contract_stage_ids` | Return the full FASTQ contract registry. |
| `list-fastq-implemented-stages` | `implemented_stages` | Return the closed execution surface implemented by this crate. |
| `list-fastq-observer-stages` | `observer_specialized_stage_ids` / `observer_stage_ids` | Return the observer-specialized stage subset. |
| `classify-fastq-runtime-interpretation` | `runtime_interpretation_for_stage` / `runtime_interpretation_for_stage_tool` | Classify FASTQ stages or stage-tool pairs as observer-specialized or generic-envelope. |
| `check-fastq-stage-support` | `FastqStagePlugin::handles_stage` | Accept only registered FASTQ stage IDs. |
| `materialize-fastq-stage` | `FastqStagePlugin::materialize` | Validate a planned FASTQ stage and return its planned invocation without choosing tools. |
| `parse-fastq-stage-outputs` | `FastqStagePlugin::parse_outputs` | Parse existing output artifacts into a plugin output envelope. |
| `build-fastq-metrics-envelope` | `metrics` / `metrics::stage_metrics` | Build deterministic metrics payloads and provenance for planned FASTQ outputs. |
| `parse-fastq-observer-output` | `observer::*` | Parse supported FASTQ tool output formats. |
| `write-fastq-observer-artifact` | `observer::artifacts::*` | Write crate-owned observer artifact JSON under `run_artifacts/`. |

## CLI Commands

None. This crate owns no binaries, subcommands, shell entrypoints, process
execution surfaces, or environment commands.

## Commands Owned Elsewhere

- User-facing CLI commands belong in command/API crates.
- Pipeline composition belongs in planner and pipeline crates.
- Tool execution belongs in runner/runtime crates.
- Environment and container commands belong in environment crates.

## Validation

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stages-fastq --test contracts --no-default-features
```
