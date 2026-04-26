# Tests

This file is the single test documentation entrypoint for `bijux-dna-bench`.
README files are intentionally not allowed under `tests/`.

## Suite Entrypoints

- `tests/boundaries.rs` loads source, docs, root-layout, and guardrail tests.
- `tests/contracts.rs` loads public API, benchmark contract, suite catalog,
  docs/fixture, workspace path, and ownership tests.
- `tests/determinism.rs` loads stable ordering, compare, and realistic snapshot
  tests.
- `tests/semantics.rs` loads gate semantic tests.
- `tests/guardrails.rs` runs the shared crate guardrail policy from the root.
- `tests/workspace_paths.rs` provides repository path helpers.

## Boundary Tests

- `tests/boundaries/architecture_tree.rs` protects the root layout, 10-docs
  allowance, Markdown location rules, source layout, and test layout.
- `tests/boundaries/guardrails.rs` runs shared guardrails for the crate.

## Contract Tests

- `tests/contracts/api/` protects the public API snapshot, owner comments, no
  public API panics, and no raw JSON in public surfaces.
- `tests/contracts/benching/` protects benchmark contract shape, suite catalog
  rules, canonical ids, workspace paths, and dependency boundaries.
- `tests/contracts/docs/` protects docs-to-fixture alignment.
- `tests/contracts/contracts.rs` is the contract target aggregator.

## Determinism Tests

- `tests/determinism/stable_ordering.rs` protects deterministic summary ordering.
- `tests/determinism/compare.rs` protects compare output snapshots.
- `tests/determinism/bench_realistic_snapshot.rs` protects a realistic suite
  snapshot.

## Semantic Tests

- `tests/semantics/gate/gate.rs` protects gate pass/fail behavior.
- `tests/semantics/gate/rejects_unknown_metric.rs` protects unknown metric
  rejection.

## Fixtures And Snapshots

- `tests/fixtures/` contains governed benchmark input fixtures.
- `tests/snapshots/` contains public API and compare snapshots.

## Commands

Run from the `bijux-genomics` repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench --test boundaries --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench --test contracts --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench --test determinism --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench --test semantics --no-default-features
```
