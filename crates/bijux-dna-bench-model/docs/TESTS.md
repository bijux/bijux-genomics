# bijux-dna-bench-model Tests

The crate keeps one root `README.md` and this single test guide under `docs/`.
Test subdirectories intentionally do not carry their own README files; the
workspace policy suite checks repository source and documentation boundaries.

## Test Layout

- Workspace boundary checks run in `bijux-dna-policies`, where dependency and source ownership rules are enforced.
- `tests/contracts.rs` aggregates suite validation contract tests.
- `tests/determinism.rs` aggregates seeded-randomness and no-hidden-randomness
  checks.
- `tests/schemas.rs` aggregates public API docs and snapshot checks.
- `tests/semantics.rs` aggregates decision explainability and metric semantics
  checks.
- `tests/snapshots/` stores governed public surface snapshots.
- `tests/fixtures/` is reserved for stable fixtures used by integration tests.

## Surface Ownership

- Source or docs tree changes: run the policy crate's `boundaries` target.
- Suite validation changes: run `contracts`.
- Public exports, schemas, or public docs: run `schemas`.
- Gate policy records or rationale trace contracts: run `semantics`.
- Scientific gate evaluation or metric semantics: run the
  `bijux-dna-science` `benchmark_gate` test target.
- Bootstrap, randomness, or ordering changes: run `determinism`.

## Commands

Run from the `bijux-genomics` repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-policies --test boundaries --no-default-features
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

Shared guardrail parser regressions are owned and executed by the policy crate. The empty boundary aggregator is retired; existing contract suites retain model schema and architecture checks. Constructor-only guardrail wrappers provide no source-check coverage and are retired.

`tests/contracts/campaign_confidentiality.rs` covers semantic TOML confidentiality
through the public raw-source validator, including equivalent typed Slurm input,
escaped and nested sensitive keys, array members, reference controls, private-key
markers and malformed-source diagnostics. Repository campaign acquisition and
its pre-effect ordering remain covered by the benchmark owner's campaign tests.
