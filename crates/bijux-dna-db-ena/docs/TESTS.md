# TESTS

## Unit tests
- `client` URL/query and TSV parsing tests.
- `download` plan materialization tests.
- `model` normalization tests.

## Guardrails
- `tests/guardrails.rs` validates workspace guardrail conventions for this crate.
- `tests/boundaries.rs` aggregates boundary policies for the crate test surface.
- `tests/boundaries/architecture.rs` locks the expected crate tree and top-level ownership.
