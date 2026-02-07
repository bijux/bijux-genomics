# Tests

## What
Maps tests in this crate to their purpose and failure meaning.

## Why
Tests should explain the contract they enforce.

## Non-goals
- Full test implementation detail.

## Contracts
- Each test file should be referenced here.

## Contracts suite (`tests/contracts/*`)
- `tests/contracts/architecture.rs` → dependency boundary assertions.
- `tests/contracts/effect_boundary.rs` → effect boundaries (no process spawn).
- `tests/contracts/params_hash.rs` → canonical params hashing stability.
- `tests/contracts/runner_tests.rs` → plan execution behavior.
- `tests/contracts/support_naming.rs` → support helper naming rules.

## Recording suite (`tests/recording/*`)
- `tests/recording/recording_completeness.rs` → truth-set emission completeness.
- `tests/recording/docs_recording_truth_set.rs` → docs reference for truth set.
- `tests/recording/run_manifest.rs` → run manifest includes telemetry/facts.

## Determinism suite (`tests/determinism/*`)
- `tests/determinism/replay_determinism.rs` → replay produces identical records + tree.
- `tests/determinism/manifest_layout_snapshot.rs` → same manifest hash + layout tree snapshot.

## Failure modes
- Missing test documentation causes drift and confusion.

## Testkit patterns
See `crates/bijux-testkit/docs/USAGE.md` for shared fixture and snapshot helpers.
