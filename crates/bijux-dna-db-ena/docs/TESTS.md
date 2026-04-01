# TESTS

## Unit coverage
- `client` tests cover filereport URL construction and TSV decoding
- `download` tests cover deterministic task planning
- `model` tests cover normalization and source-selection helpers

## Integration coverage
- `tests/guardrails.rs` validates workspace guardrail conventions for this crate
- `tests/boundaries.rs` aggregates source-tree architecture checks
- `tests/boundaries/architecture.rs` locks the expected crate tree

## Reserved suites
- `tests/contracts/`: future user-visible ENA contract tests
- `tests/determinism/`: future reproducibility and stable-output assertions
- `tests/schemas/`: future persisted-schema or public-surface snapshots
