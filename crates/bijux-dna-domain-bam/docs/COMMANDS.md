# bijux-dna-domain-bam Commands

`bijux-dna-domain-bam` is a pure library crate and owns no executable command surface.

## Managed command inventory

There are no crate-managed CLI commands, process entrypoints, background jobs, network clients, or repository mutation commands in this crate.

## Boundary

- This crate may expose typed BAM domain data, stage specs, params, metrics, invariants, and deterministic parsers.
- Runtime execution belongs in runner, runtime, stage, planner, or developer-control-plane crates.
- Any future command surface must be rejected here unless the crate boundary changes explicitly.

## Verification

Use `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-domain-bam --no-default-features --test boundaries` to verify the command-free boundary.
