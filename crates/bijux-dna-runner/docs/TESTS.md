# Tests

## What
Maps tests in this crate to their purpose and failure meaning.

## Why
Tests should explain the contract they enforce.

## Non-goals
- Full test implementation detail.

## Suite entrypoints
- `tests/boundaries.rs` loads architectural and backend guardrail coverage.
- `tests/contracts.rs` loads runner-facing behavioral contracts.
- `tests/determinism.rs` loads replay and run-id stability coverage.
- `tests/guardrails.rs`, `tests/schemas.rs`, `tests/semantics.rs`, and `tests/workspace_paths.rs` keep focused crate-level checks visible at the root.

## Boundaries suite (`tests/boundaries/*`)
- `tests/boundaries/architecture.rs` → source tree and dependency layout contract.
- `tests/boundaries/backend/backend_invariants.rs` → backend symmetry invariants.
- `tests/boundaries/backend/fixture_parity.rs` → backend fixture structure parity.
- `tests/boundaries/backend/invocation_hash.rs` → stable invocation identity rules.
- `tests/boundaries/backend/network_guardrail.rs` → network access remains outside runner execution.
- `tests/boundaries/backend/process_guardrail.rs` → process spawn stays confined to runner and environment tooling.

## Contracts suite (`tests/contracts/*`)
- `tests/contracts/backend.rs` → backend contract behavior visible to crate consumers.

## Determinism suite (`tests/determinism/*`)
- `tests/determinism/determinism.rs` → suite-level determinism coverage anchor.
- `tests/determinism/run_id_determinism.rs` → run id stability.
- `tests/determinism/replay/replay_contract.rs` → replay contract behavior.
- `tests/determinism/replay/replay_determinism.rs` → replay determinism.

## Failure modes
- Missing test documentation causes drift and confusion.
