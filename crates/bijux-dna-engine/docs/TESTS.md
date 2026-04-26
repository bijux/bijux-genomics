# Tests

## Intent
The test tree is organized by the contract each suite protects. Boundary tests
guard the crate shape and dependency graph, contract tests guard observable
engine behavior, and determinism tests guard replayable output.

## Suite map
- `tests/boundaries.rs`: architecture tree, command inventory, documentation placement,
  dependency graph, and effect-boundary enforcement.
- `tests/contracts.rs`: public behavior for orchestration, runner retries, cancellation, params
  hashing, recording truth-set contracts, and helper naming.
- `tests/determinism.rs`: replay determinism and manifest layout stability.
- `tests/guardrails.rs`: repository policy guardrails shared with generated policy tests.

## Important directories
- `tests/boundaries/`: crate-tree, dependency, and side-effect boundary assertions.
- `tests/boundaries/command_inventory.rs`: `docs/COMMANDS.md` operation inventory contract.
- `tests/contracts/`: engine-facing behavior and documentation contracts.
- `tests/contracts/recording/`: execution truth-set documentation and recording completeness checks
- `tests/determinism/`: replay and run-manifest stability assertions.
- `tests/support/`: shared engine fixtures, graph builders, and runner stubs.

The crate intentionally has no `tests/README.md` files and no reserved
placeholder directories. New tests should live under the suite that owns the
contract they enforce.

## Regenerating affected coverage
Run from the `bijux-genomics` repository root with generated output under
`artifacts/`:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-engine --test boundaries --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-engine --test contracts --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-engine --test determinism --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-engine --no-default-features
```

## Failure interpretation
- Boundary failures mean the source tree, docs layout, dependency graph, or effect boundary
  changed.
- Contract failures mean public execution behavior or recorded artifacts changed.
- Determinism failures mean stable ordering or repeatable output changed.
