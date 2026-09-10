# bijux-dna-containers-qa Tests

## Commands

Use artifact-rooted target and test temp directories:

```sh
TEST_TMP_DIR=artifacts/test-tmp CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-containers-qa --no-default-features
TEST_TMP_DIR=artifacts/test-tmp CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-containers-qa --no-default-features --test boundaries
TEST_TMP_DIR=artifacts/test-tmp CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-containers-qa --no-default-features --test contracts
```

## Boundaries Suite

- `tests/boundaries/architecture.rs`: crate root, docs, source, and test layout.
- `tests/boundaries/guardrails/guardrails.rs`: repository policy guardrails.
- `tests/boundaries/guardrails/offline_guardrails.rs`: offline-by-default documentation.
- Command, dependency, and public API boundaries should fail when contracts drift.

## Contracts Suite

- `tests/contracts/qualification.rs`: source admission, required outputs, content identity, metrics and failure classes.
- `tests/contracts/qa_contracts.rs`: shared QA support behavior with `verification-tools` enabled.
- `tests/contracts/apptainer_smoke.rs` and `runtime_verification.rs`: verification tool behavior with the default feature enabled.

Run the default suite to retain every verification capability:

```sh
TEST_TMP_DIR=artifacts/test-tmp CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-containers-qa
```

The commands above with `--no-default-features` qualify the independently usable qualification library. Inputs are created inside owned test sandboxes for the behavior being exercised.

## Failure Meaning

- Boundary failures mean layout, docs, commands, dependencies, or public exports drifted.
- Contract failures mean QA evidence shape or support behavior changed.

The retained compiler fixture also checks that pure binding projection survives absent files while the full compiler rejects absent or changed input bytes. Input/output contract mismatches remain projection failures. The original source identities, staged-image rejection and public qualification cases remain covered; illustrative BAM/image fixture bytes do not certify tool execution.
