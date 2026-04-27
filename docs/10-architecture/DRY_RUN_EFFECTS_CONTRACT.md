# Dry-Run Effects Contract

Owner: Architecture
Scope: Dry-run side effects and artifact boundaries
Last reviewed: 2026-04-26
Contract version: v1

## Purpose
Guarantee that dry-run paths prove planning and manifest shape without executing product tools.

## Allowed inputs
- CLI/API request parameters.
- Repository configs and domain contracts.
- Fixture files used by contract tests.

## Forbidden dependencies
- Dry-run code must not require runner backends, container engines, or network services.

## Forbidden effects
- No external tool execution.
- No network access.
- No writes outside declared run/artifact output paths.
- No mutation of source configs, fixtures, or snapshots.

## Validation command
- `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna --test contracts dry_run --no-default-features`
- The governed dry-run test anchor lives in
  [../../crates/bijux-dna/tests/contracts/dry_run.rs](../../crates/bijux-dna/tests/contracts/dry_run.rs).
- Declared run and artifact output paths live in [../30-operations/RUN_ARTIFACTS.md](../30-operations/RUN_ARTIFACTS.md).

## Failure modes
- A dry-run that executes tools can make reports look reproducible without proving runtime isolation.
- A dry-run that writes undeclared paths makes tests and HPC runs unsafe to repeat.
