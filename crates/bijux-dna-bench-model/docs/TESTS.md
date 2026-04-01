# bijux-dna-bench-model Tests

## Coverage
- `tests/boundaries.rs` locks the documented source tree.
- `tests/contracts.rs` covers suite validation behavior.
- `tests/determinism.rs` covers repeatability and no-randomness guarantees.
- `tests/schemas.rs` covers public-surface and docs-linked schema checks.
- `tests/semantics.rs` covers explainability and metric semantics behavior.

## How to run
- `cargo test -p bijux-dna-bench-model`
- `cargo test -p bijux-dna-bench-model --test boundaries`
- `cargo test -p bijux-dna-bench-model --test contracts`

## Notes
- Prefer `boundaries` when changing module layout or public-surface shape.
- Prefer `contracts` when changing suite validation behavior.
- Prefer `schemas` when changing exported types or public API docs.
