# Tests

## Unit coverage

- `client` tests cover filereport URL construction, header validation, TSV row
  decoding, numeric validation, sample filtering, and pre-request query
  validation.
- `download` tests cover deterministic task planning, output path
  deduplication, config validation, and dry-run reporting.
- `model` tests cover selector normalization, selector validation, URL
  normalization, and source-selection helpers.

## Integration coverage

- `tests/guardrails.rs` validates workspace guardrail conventions for this
  crate.
- `tests/boundaries.rs` aggregates source-tree, docs placement, command
  inventory, and dependency graph checks.
- `tests/boundaries/architecture.rs` locks the expected crate tree.

## Test Tree

```text
tests/
├── boundaries.rs
├── boundaries/
│   └── architecture.rs
└── guardrails.rs
```

Add new integration test intent directories only when they contain tracked test
files and are documented here.
