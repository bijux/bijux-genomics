# Commands

`bijux-dna-stages-bam` owns no CLI commands, binaries, subcommands, shell
entrypoints, or process execution surfaces.

## Commands This Crate Manages
None.

## Surfaces This Crate Provides Instead
- BAM stage IDs and planner-facing stage metadata through
  `implemented_stages`.
- BAM stage plugin materialization through `BamStagePlugin`.
- BAM observer parsers and metric discovery through `observer` and `metrics`.

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
