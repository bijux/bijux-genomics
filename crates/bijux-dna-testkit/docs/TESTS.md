# Tests

## Standard Verification

Run from the `bijux-genomics` repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-testkit --no-default-features
```

## Focused Checks

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-testkit --test boundaries --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-testkit --test contracts --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-testkit --test determinism --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-testkit --test schemas --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-testkit --test guardrails --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo check -p bijux-dna-testkit --no-default-features
```

## Test Layout

- `tests/boundaries.rs` locks source layout, docs placement, dependency shape,
  dev-dependency usage, and effect boundaries.
- `tests/contracts.rs` locks helper contracts such as fixture error reporting.
- `tests/determinism.rs` locks deterministic clock, RNG, ordering, and timestamp
  behavior.
- `tests/schemas.rs` locks public API and snapshot normalization behavior.
- `tests/guardrails.rs` runs the workspace guardrail policy.
- `tests/snapshots/` stores public surface snapshots.
