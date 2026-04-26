# Tests

## Standard Verification

Run from the `bijux-genomics` repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stages-vcf --no-default-features
```

## Focused Checks

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stages-vcf --test boundaries --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stages-vcf --test contracts --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stages-vcf --test guardrails --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo check -p bijux-dna-stages-vcf --no-default-features
```

## Test Layout

- `tests/boundaries.rs` locks source layout, documentation placement, dependency
  shape, and command inventory.
- `tests/contracts.rs` includes the focused `tests/contracts/` files for pipeline,
  stage, IO, invariant, imputation, panel, phasing, population, and real-tool
  behavior.
- `tests/guardrails.rs` loads the repository policy guardrail for this crate.

## External Tool Tests

Some VCF IO tests skip external-tool paths unless `BIJUX_E2E=1` is set. The
default crate verification still exercises deterministic fallback behavior.
