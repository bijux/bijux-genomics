# Tests

This file maps runtime tests to their contract purpose. Directory-local README files are intentionally not used; this document is the test taxonomy for the crate.

## Suite Entrypoints
- `tests/boundaries.rs` loads boundary and dependency guardrails.
- `tests/contracts.rs` loads runtime behavior and documentation contracts.
- `tests/determinism.rs` loads fixture stability checks.
- `tests/schemas.rs` loads schema snapshot checks.
- `tests/guardrails.rs` keeps the root guardrail smoke test visible.
- `tests/support/workspace_paths.rs` provides repository path helpers without creating a helper-only test binary.

## Boundaries Suite
- `tests/boundaries/command_inventory.rs` checks `docs/COMMANDS.md`.
- `tests/boundaries/dependency_graph.rs` checks dependency boundaries.
- `tests/boundaries/guardrails.rs` checks policy guardrails.

## Contracts Suite
- `tests/contracts/canonical_writer.rs` checks canonical JSON writer usage.
- `tests/contracts/docs_layout.rs` checks README, test docs, and public API docs.
- `tests/contracts/experimental_registry_alias.rs` checks governed registry aliases.
- `tests/contracts/manifest_integrity.rs` checks manifest integrity, artifact checksums, profile/lock manifests, and path safety.
- `tests/contracts/run_layout_contract.rs` checks run-layout invariants.
- `tests/contracts/stage_runner_contract.rs` checks runner support contracts.
- `tests/contracts/telemetry_contract.rs` checks telemetry schema, redaction, event taxonomy, and validation.
- `tests/contracts/telemetry_golden.rs` checks telemetry JSONL fixture stability.

## Reference Suite
- `tests/contracts/reference/reference_example.rs` → end-to-end reference story.
- `tests/contracts/reference/docs_reference_example.rs` → documentation coverage for the reference story.

## Determinism and Schema Suites
- `tests/determinism/fixture_stability.rs` checks stable fixture canonicalization.
- `tests/schemas/schema/runtime_schema_snapshots.rs` checks schema fixture snapshots.

## Main Command

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-runtime --no-default-features
```

## Failure modes
- Boundary failures mean ownership or dependency shape changed without updating contracts.
- Contract failures mean runtime writer, layout, telemetry, or manifest behavior drifted.
- Schema failures mean stable artifact shape changed and needs an explicit versioned review.
- Determinism failures mean fixture canonicalization changed.
