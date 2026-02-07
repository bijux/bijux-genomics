# Tests

## What
Maps tests in this crate to their purpose and failure meaning.

## Why
Tests should explain the contract they enforce.

## Non-goals
- Full test implementation detail.

## Contracts
- `tests/determinism/*` → deterministic plan ordering and stable hashes.
- `tests/graph/*` → graph topology and stage wiring snapshots.
- `tests/plan/*` → plan JSON schema + configuration contracts.
- `tests/explain/*` → explain payload shape.

## Mapping
- `tests/determinism/determinism.rs` → stable ordering/hashes.
- `tests/graph/graph_snapshots.rs` → graph snapshot contract.
- `tests/graph/docs_graph_snapshots.rs` → docs anchor coverage.
- `tests/plan/plan_json.rs` → plan JSON schema contract.
- `tests/plan/plan_snapshots.rs` → plan snapshot contract.
- `tests/plan/plan_integration.rs` → plan integration wiring.
- `tests/plan/pipeline_plan_snapshot.rs` → pipeline plan snapshot.
- `tests/plan/artifacts_contract.rs` → stage artifact contract.
- `tests/plan/params_complete.rs` → param completeness contract.
- `tests/plan/contract_handshake.rs` → plan handshake fixtures.
- `tests/plan/guardrails.rs` → policy guardrails.
- `tests/plan/no_parsing_execution.rs` → planner purity (no parsing/execution).
- `tests/explain/explainability.rs` → explain output contract.

## Failure modes
- Missing test documentation causes drift and confusion.

## Testkit patterns
See `crates/bijux-testkit/docs/USAGE.md` for shared fixture and snapshot helpers.
