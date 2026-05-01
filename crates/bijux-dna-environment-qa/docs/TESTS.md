# bijux-dna-environment-qa Tests

## Commands

Use artifact-rooted target and test temp directories:

```sh
TEST_TMP_DIR=artifacts/test-tmp CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-environment-qa --no-default-features
TEST_TMP_DIR=artifacts/test-tmp CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-environment-qa --no-default-features --test boundaries
TEST_TMP_DIR=artifacts/test-tmp CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-environment-qa --no-default-features --test contracts
```

## Boundaries Suite

- `tests/boundaries/architecture.rs`: crate root, docs, source, and test layout.
- `tests/boundaries/guardrails/guardrails.rs`: repository policy guardrails.
- `tests/boundaries/guardrails/offline_guardrails.rs`: offline-by-default documentation.
- Command, dependency, and public API boundaries should fail when contracts drift.

## Contracts Suite

- `tests/contracts/artifacts/qa_artifact_contract.rs`: fixture manifest/report shape.
- `tests/contracts/qa_contracts.rs`: shared QA support contract coverage.

## Determinism Suite

- `tests/determinism/fixture_stability.rs`: stable JSON fixture ordering.

## Support Fixtures

- `tests/support/image_qa_support.rs`: helper contract tests included by `tests/contracts.rs`.
- `tests/fixtures/qa_artifacts/default/`: minimal manifest/report fixtures.

## Failure Meaning

- Boundary failures mean layout, docs, commands, dependencies, or public exports drifted.
- Contract failures mean QA evidence shape or support behavior changed.
- Determinism failures mean fixture output is no longer stable.
