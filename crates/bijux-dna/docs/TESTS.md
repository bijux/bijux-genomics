# bijux-dna Tests

## Coverage
- `tests/boundaries.rs` is the integration-test entrypoint for boundary coverage.
- `tests/boundaries/architecture_tree.rs` enforces the documented crate tree.
- `tests/boundaries/guardrails/` checks dependency, policy, process-spawn, and public-surface boundaries.
- `tests/contracts/` checks CLI behavior, dry-run contracts, bank contracts, and HPC layout behavior.
- `tests/snapshots/help/` locks help output contracts.

## How to run
- `cargo test -p bijux-dna`
- `cargo test -p bijux-dna --test boundaries`
- `cargo test -p bijux-dna --test contracts`

## Notes
- Prefer `boundaries` when changing module layout, public surface, or dependency rules.
- Prefer `contracts` when changing command behavior or CLI-visible outputs.
- Re-run snapshot coverage after changing help text or dry-run formatting.
