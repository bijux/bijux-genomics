# bijux-dna-domain-fastq Commands

`bijux-dna-domain-fastq` is a pure library crate. It owns FASTQ domain contracts, typed models,
catalogs, parsers, and validation helpers, but it owns no executable command surface.

## Managed command inventory

There are no crate-managed CLI commands, Cargo binary targets, process
entrypoints, background jobs, network clients, or repository mutation commands
in this crate.

## Forbidden Command Surfaces

- No Cargo binary targets or `src/bin` command modules.
- No `src/main.rs`.
- No CLI parser ownership.
- No process spawning or runtime command execution.

## Boundary

- This crate may expose typed domain APIs for planners, stages, benchmark tooling, and analyzers.
- Runtime execution belongs in runner, runtime, stage, planner, or developer-control-plane crates.
- Any future command surface must be rejected here unless the crate boundary changes explicitly.
- If a command is added after a reviewed boundary change, this document must list the binary,
  purpose, flags, effects, and owning module in the same change.

## Verification

Use `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-domain-fastq --no-default-features --test boundaries`
to verify the command-free boundary.
