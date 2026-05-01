# Commands

This file is the single source of truth for commands owned by `bijux-dna-analyze`.

## Runtime Command Surface
This crate owns the library entrypoint `analyze_run(input: &AnalyzeInput)` and the
`bijux-dna-verify` utility binary for external bundle verification and release-facing
evidence materialization.

Library-mode entrypoint variants:

- `Report`: load facts, validate inputs, compute derived analysis, build a report model, and render
  configured report outputs.
- `Summary`: load facts, validate inputs, compute summaries, and render summary outputs.
- `Compare`: compare two completed run directories and write `compare.json`.
- `Rank`: compute ranking data from loaded facts and render ranking outputs when configured.

`bijux-dna-verify` commands:

- `verify-evidence <evidence_bundle.json>`
- `verify-profile <profile_bundle.json>`
- `write-methods <run_dir> [facts.jsonl]`
- `write-profile <run_dir> [profile] [facts.jsonl]`
- `challenge-submit <run_dir> <artifact_id> <evidence_path> <report_field> <caveat> <question> <requested_by>`
- `challenge-list <run_dir>`

Other CLI crates should route user-facing analysis commands into these modes/surfaces
instead of duplicating analysis policy.

## Required Local Checks
Use the repository root as the working directory for every command.

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo check -p bijux-dna-analyze --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-analyze --test boundaries --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-analyze --test contracts --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-analyze --test determinism --no-default-features
```

## Feature Checks
SQLite coverage is feature-gated because the crate keeps database readers optional.

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-analyze --test schemas --features sqlite
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-analyze --test semantics --features sqlite
```

Parquet coverage is feature-gated because `facts.parquet` support is optional.

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo check -p bijux-dna-analyze --features parquet
```

## Snapshot Commands
Only bless snapshots after reviewing the diff.

```sh
INSTA_UPDATE=always CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-analyze --test contracts --no-default-features
INSTA_UPDATE=always CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-analyze --test semantics --features sqlite
```

## Final Package Check
Run this before a pull request when time allows:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-analyze --all-features
```
