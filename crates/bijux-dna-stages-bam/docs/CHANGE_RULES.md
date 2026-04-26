# Change Rules

These rules apply to public exports, stage contracts, observer outputs, fixture
snapshots, and dependency boundaries.

## Breaking Changes

- Removing or renaming a crate-root export.
- Removing, renaming, or changing the meaning of a BAM stage ID.
- Changing observer JSON shape, metric fields, metrics-envelope fields, or stage contract snapshots
  without an explicit versioned migration.
- Moving planning, tool choice, command construction, runtime scheduling, or environment ownership
  into this crate.
- Adding a normal dependency on a planner, runner, runtime, API, CLI, pipeline, or environment
  crate.

## Non-Breaking Changes

- Adding a parser for a documented BAM output while preserving existing parser behavior.
- Adding a supported output filename to metric discovery when existing filenames still work.
- Adding test fixtures and snapshots for already-supported metrics.
- Expanding docs without changing public behavior.

## Adding Or Changing An Observer

1. Add or update fixtures under `tests/fixtures/observer/default/`.
2. Add parser coverage under `tests/contracts/observer/`.
3. Add or update canonical snapshots under `tests/fixtures/observer_snapshots/default/`.
4. Update `docs/STAGE_CONTRACTS.md` with supported output names and fixture intent.
5. Run the contracts and determinism suites.

## Required Updates By Surface

- Public export changes: update `docs/PUBLIC_API.md`, `README.md`, and contract coverage.
- Stage registry changes: update `docs/STAGE_CONTRACTS.md`,
  `tests/contracts/registry_completeness.rs`, and snapshots.
- Dependency changes: update `docs/DEPENDENCIES.md` and dependency boundary tests.
- Layout changes: update `docs/ARCHITECTURE.md` and architecture boundary tests.
- Managed operation changes: update `docs/COMMANDS.md` and command inventory tests.

## Verification

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stages-bam --test boundaries --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stages-bam --test contracts --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stages-bam --test determinism --no-default-features
```
