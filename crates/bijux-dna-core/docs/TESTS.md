# Tests

This file maps the stable test entrypoints and intent directories for
`bijux-dna-core`.

Core is a dependency anchor, so source-tree drift, boundary drift, and
public-surface drift need explicit guardrails.

## Entry points
- `tests/boundaries.rs` — boundary, layering, guardrail, and source-tree contract coverage.
- `tests/contracts.rs` — contract behavior, identity, execution, and surface contracts.
- `tests/schemas.rs` — docs and public-surface locks.
- `tests/semantics.rs` — identifier catalog, validator, metrics, and input-assessment semantics.
- `tests/guardrails.rs` — crate-local guardrail smoke coverage.

## Intent directories
- `tests/boundaries/` — dependency boundaries, command inventory, and layout contracts.
- `tests/contracts/` — execution, identity, and surface behavior contracts.
- `tests/fixtures/` — stable shared inputs for contract and schema coverage.
- `tests/schemas/` — public API and docs locks.
- `tests/semantics/` — semantic behavior checks for IDs, metrics, and input assessment.

## Layout ownership

- `tests/boundaries/architecture_tree.rs` locks the crate root, docs allowance,
  source tree, public module tree, and named test-suite layout.
- `tests/boundaries/command_inventory.rs` locks `docs/COMMANDS.md` as the
  complete managed core operation inventory.
- `tests/boundaries/dependency_graph.rs` locks normal dependencies to generic
  library crates and rejects workspace crates in `[dependencies]`.
- `tests/boundaries/layering.rs` locks source layering between foundation and
  contract modules.
- `tests/contracts/execution/` owns execution graph, execution output, and
  canonical execution-plan behavior.
- `tests/contracts/identity/` owns hashing, reproducibility identity, run index,
  run metadata, and prelude export contracts.
- `tests/contracts/surface/` owns canonicalization, ID, metric, stage-selection,
  and public contract behavior.
- `tests/schemas/` owns docs-to-code locks and public surface snapshots.
- `tests/semantics/ids/` owns catalog and typed-id conversion semantics.
- `tests/semantics/metrics/` owns metrics registry semantics.
- `tests/semantics/input_assessment.rs` owns FASTQ discovery, assessment, and
  persistence behavior.

## Source-tree contract

- `tests/boundaries/architecture_tree.rs` locks the documented `core` namespace
  layout, including the `id_catalog/{pipeline,stage,tool}` and
  `ids/{parsing,typed}` partitions.
- New test intent directories must be backed by tracked tests or fixtures before
  they are added to this contract.
