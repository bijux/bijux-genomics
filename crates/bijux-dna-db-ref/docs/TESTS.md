# Tests

## Coverage

- `tests/guardrails.rs` checks policy guardrail registration.
- `tests/boundaries.rs` locks the documented crate tree, docs placement,
  command inventory, public API docs, and dependency graph.
- `tests/contracts.rs` exercises deterministic provider and resolution behavior.
- Unit tests under `src/resolution/` cover lock parsing, checksum validation,
  contig normalization, catalog file validation, and compatibility failures.

## How to run

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-db-ref --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-db-ref --test boundaries --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-db-ref --test contracts --no-default-features
```

## Test Tree

```text
tests/
├── boundaries.rs
├── boundaries/
│   └── architecture_tree.rs
├── contracts.rs
├── contracts/
│   └── runtime_provider.rs
└── guardrails.rs
```

## Notes

- Prefer `boundaries` when changing docs, source layout, dependency graph, or
  public surface shape.
- Prefer `contracts` when changing species, bundle, panel, or map resolution
  behavior.
