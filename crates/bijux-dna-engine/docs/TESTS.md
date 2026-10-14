# Tests

## Intent
The test tree is organized by the contract each suite protects. Boundary tests
guard dependencies and effects, contract tests guard observable
engine behavior, and determinism tests guard replayable output.

## Suite map

- `tests/boundaries.rs`: dependency declarations and effect-boundary enforcement.
- `tests/contracts.rs`: public behavior for orchestration, runner retries, cancellation, params
  hashing, recording truth-set contracts, and helper naming.
- `tests/determinism.rs`: replay determinism and manifest layout stability.
- `tests/guardrails.rs`: shared source policy checks run against this crate.

## Important directories

- `tests/boundaries/`: dependency and side-effect boundary assertions.
- `tests/contracts/`: engine-facing behavior and documentation contracts.
- `tests/contracts/recording.rs`: execution truth-set documentation and recording completeness checks.
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

- Boundary failures mean dependency declarations or the effect boundary changed.
- Contract failures mean public execution behavior or recorded artifacts changed.
- Determinism failures mean stable ordering or repeatable output changed.

Shared guardrail parser regressions are owned and executed by the policy crate.
Engine boundary cases retain dependency and effect checks, while
`tests/guardrails.rs` calls the shared source policy implementation against the
engine source tree.

The engine suite does not enforce an exact source or documentation tree or a
managed-operation inventory. Keep the documented owner boundaries and operation
list accurate when changing them.
