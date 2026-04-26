# bijux-dna-bench-model Tests

The crate keeps one root `README.md` and this single test guide under `docs/`.
Test subdirectories intentionally do not carry their own README files; the
boundary suite enforces that documentation layout.

## Test Layout

- `tests/boundaries.rs` aggregates `tests/boundaries/*` and locks crate root,
  docs allowance, source tree, test tree, and structural guardrails.
- `tests/contracts.rs` aggregates suite validation contract tests.
- `tests/determinism.rs` aggregates seeded-randomness and no-hidden-randomness
  checks.
- `tests/schemas.rs` aggregates public API docs and snapshot checks.
- `tests/semantics.rs` aggregates decision explainability and metric semantics
  checks.
- `tests/guardrails.rs` checks policy configuration wiring for this crate.
- `tests/snapshots/` stores governed public surface snapshots.
- `tests/fixtures/` is reserved for stable fixtures used by integration tests.

## Surface Ownership

- Source or docs tree changes: run `boundaries`.
- Suite validation changes: run `contracts`.
- Public exports, schemas, or public docs: run `schemas`.
- Gate policy, metric semantics, or rationale trace changes: run `semantics`.
- Bootstrap, randomness, or ordering changes: run `determinism`.

## Commands

Run from the `bijux-genomics` repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --test boundaries --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --test contracts --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --test determinism --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --test schemas --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --test semantics --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --all-features
```

## Expectations

- Add or update the narrowest integration test with every behavior change.
- Keep snapshots reviewable and tied to explicit public API changes.
- Keep fixtures deterministic, small, and committed only when they represent a
  durable contract.
- Do not add Markdown files under `tests/`; update this guide instead.
