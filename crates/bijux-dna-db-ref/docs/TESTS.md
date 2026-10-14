# Tests

## Coverage

- `tests/boundaries.rs` locks the documented crate tree, docs placement,
  command inventory, public API docs, and dependency graph.
- `tests/contracts.rs` exercises deterministic provider and resolution behavior.
- `tests/contracts/runtime_root.rs` exercises private catalog and lock identities,
  conflicting repository roots, invalid bindings, missing files, and unreadable
  file shapes through independent child processes. Tests bind their writable
  runtime directories under repository artifacts.
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

FASTA index contracts cover LF, CRLF, final lines without terminators and distinct record wrap widths. `fixtures/fasta_index/native_cases.json` records byte-exact FAI output produced by bcftools/HTSlib 1.20; JSON escapes preserve CRLF bytes across Git checkout policies. Negative cases reject ambiguous identity and non-indexable layouts.
