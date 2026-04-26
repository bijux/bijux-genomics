# Dependencies

`bijux-dna-testkit` should remain a small test-support crate with no required
dependencies on product workspace crates.

## Normal Dependencies

- `anyhow`: reserved for shared helper APIs that need workspace-style error
  compatibility.
- `rand`: seeded deterministic RNG helpers.
- `serde`: JSON value and serialization compatibility for callers.
- `serde_json`: fixture parsing and snapshot normalization.
- `tempfile`: isolated temporary directories.

## Dev Dependencies

- `bijux-dna-policies`: crate-local guardrail validation.

## Forbidden Edges

This crate must not take required dependencies on domain, stage, API, planner,
runtime, runner, environment, database, engine, benchmark, or application crates.
Other crates may use `bijux-dna-testkit` from dev-dependencies only.

## Validation

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-testkit --test boundaries --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo tree -p bijux-dna-testkit --no-default-features --edges normal,dev
```
