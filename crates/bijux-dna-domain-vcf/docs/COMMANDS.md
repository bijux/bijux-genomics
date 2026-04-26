# bijux-dna-domain-vcf Commands

`bijux-dna-domain-vcf` is a pure library crate. It owns VCF domain contracts, typed params,
metrics, taxonomy, coverage reporting, and registry materialization helpers, but it owns no
executable command surface.

## Managed command inventory

There are no crate-managed CLI commands, process entrypoints, background jobs, network clients, or
repository mutation commands in this crate.

## Boundary

- This crate may expose typed library APIs and deterministic registry-rendering functions.
- Runtime execution belongs in runner, runtime, stage, planner, or developer-control-plane crates.
- Generated config writes belong to the caller that chooses an output path; this crate only returns
  deterministic TOML strings.
- Any future command surface must be rejected here unless the crate boundary changes explicitly.

## Verification

Use `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-domain-vcf --no-default-features --test boundaries`
to verify the command-free boundary.
