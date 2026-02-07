# Tests

## What
Maps tests in this crate to their purpose and failure meaning.

## Why
Tests should explain the contract they enforce.

## Non-goals
- Full test implementation detail.

## Backend suite (`tests/backend/*`)
- `tests/backend/backend_invariants.rs` → backend symmetry invariants.
- `tests/backend/invocation_hash.rs` → stable invocation hash across backends.
- `tests/backend/process_guardrail.rs` → process spawn confined to runner/env tooling.

## Replay suite (`tests/replay/*`)
- `tests/replay/replay_contract.rs` → replay contract behavior.
- `tests/replay/replay_determinism.rs` → replay determinism.

## Determinism suite (`tests/determinism/*`)
- `tests/determinism/run_id_determinism.rs` → run id stability.

## Failure modes
- Missing test documentation causes drift and confusion.
