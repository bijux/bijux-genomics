# Tests

## Intent
The test tree is organized by what each suite protects.

## Suite map
- `tests/boundaries.rs`: architecture tree and workspace guardrails
- `tests/contracts.rs`: public API, bench contracts, fixture/docs alignment, workspace paths, and
  ownership checks
- `tests/determinism.rs`: ordering, comparison stability, and realistic snapshots
- `tests/semantics.rs`: gate semantics and metric rejection behavior

## Important directories
- `tests/contracts/api/`: root surface and public API checks
- `tests/contracts/benching/`: benchmark contract, suite catalog, workspace paths, and ownership
- `tests/contracts/docs/`: docs-to-fixtures consistency checks
- `tests/fixtures/`: benchmark contract inputs
- `tests/snapshots/`: public API and compare snapshots

## Reserved coverage
- `tests/schemas/`: reserved for future standalone schema and public-surface snapshots that need a
  dedicated target

## Source-tree contract
- `tests/boundaries/architecture_tree.rs` locks the documented benchmark tree, including
  `repo/run_artifacts/`, `repo/sqlite/queries/run_index/`, `workflow/summary_{fairness,scope,statistics}.rs`,
  and the `public_api/stable_surface.rs` owner.
