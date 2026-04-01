# bijux-dna-db-ref Tests

## Coverage
- `tests/guardrails.rs` checks policy guardrail registration.
- `tests/boundaries.rs` locks the documented crate tree.
- `tests/contracts.rs` exercises deterministic provider and resolution behavior.

## How to run
- `cargo test -p bijux-dna-db-ref`
- `cargo test -p bijux-dna-db-ref --test boundaries`
- `cargo test -p bijux-dna-db-ref --test contracts`

## Notes
- Prefer `boundaries` when changing source layout or public surface shape.
- Prefer `contracts` when changing species, bundle, panel, or map resolution behavior.
