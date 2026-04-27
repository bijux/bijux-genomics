# Architecture Contract

Owner: Architecture
Scope: Repository root architecture map and boundary authority
Last reviewed: 2026-04-26
Contract version: v1
Applies to: repository root, workspace crates, generated configs, policy tests

## Purpose
Define the machine-checkable contract for repository architecture authority.

## Allowed inputs
- Workspace membership from [Cargo.toml](../../Cargo.toml).
- Dependency edges from crate [Cargo.toml](../../Cargo.toml) files.
- Boundary authority from [docs/10-architecture/BOUNDARY_MAP.md](BOUNDARY_MAP.md).
- Responsibility authority from [docs/10-architecture/CRATE_AUTHORITY_MAP.md](CRATE_AUTHORITY_MAP.md).
- Contract artifact authority from [docs/10-architecture/CONTRACT_SPINE.md](CONTRACT_SPINE.md).

## Forbidden dependencies
- Crate dependency edges not listed in the executable `boundaries` block in
  [docs/10-architecture/BOUNDARY_MAP.md](BOUNDARY_MAP.md).
- Domain, stage, planner, runtime, runner, API, CLI, and analyzer concerns owned by a different
  crate family in [docs/10-architecture/CRATE_AUTHORITY_MAP.md](CRATE_AUTHORITY_MAP.md).
- Parallel contract authorities outside the files listed in
  [docs/10-architecture/CONTRACT_INDEX.md](CONTRACT_INDEX.md).

## Forbidden effects
- Architecture validation must not run product pipelines, spawn external tools, use network access,
  mutate generated configs, or rewrite snapshots.
- Architecture validation may read repository files and emit test diagnostics only.

## Validation commands
- `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-policies --test boundaries`
- `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-policies --test contracts`

## Failure modes
- Missing or stale authority documents make policy failures ambiguous.
- Undeclared dependency edges allow hidden reverse coupling.
- Multiple contract sources for the same artifact make generated configs and snapshots drift.
