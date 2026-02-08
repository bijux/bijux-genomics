# Tests

## What
Maps tests in this crate to their purpose and failure meaning.

## Why
Tests should explain the contract they enforce.

## Non-goals
- Full test implementation detail.

## Contracts
- `tests/determinism/*` → deterministic plan ordering and stable hashes (docs/DETERMINISM.md).
- `tests/contracts/graph/*` → graph topology and stage wiring snapshots (docs/STAGE_MAPPING.md).
- `tests/contracts/plan/*` → plan JSON schema + configuration contracts (docs/PLANNER_MODEL.md).
- `tests/contracts/explain/*` → explain payload shape (docs/EXPLAIN_OUTPUT.md).

## Plan JSON stability
Plan JSON snapshots live under `tests/contracts/plan/snapshots/*` and enforce stable ordering.

## Mapping
- `tests/determinism/determinism.rs` → stable ordering/hashes.
- `tests/contracts/graph/graph_snapshots.rs` → graph snapshot contract.
- `tests/contracts/graph/docs_graph_snapshots.rs` → docs anchor coverage.
- `tests/contracts/plan/plan_json.rs` → plan JSON schema contract.
- `tests/contracts/plan/plan_snapshots.rs` → plan snapshot contract.
- `tests/contracts/plan/plan_integration.rs` → plan integration wiring.
- `tests/contracts/plan/pipeline_plan_snapshot.rs` → pipeline plan snapshot.
- `tests/contracts/plan/artifacts_contract.rs` → stage artifact contract.
- `tests/contracts/plan/params_complete.rs` → param completeness contract.
- `tests/contracts/plan/contract_handshake.rs` → plan handshake fixtures.
- `tests/contracts/plan/guardrails.rs` → policy guardrails.
- `tests/contracts/plan/no_parsing_execution.rs` → planner purity (no parsing/execution).
- `tests/contracts/explain/explainability.rs` → explain output contract.

## Failure modes
- Missing test documentation causes drift and confusion.

## Testkit patterns
See `crates/bijux-testkit/docs/USAGE.md` for shared fixture and snapshot helpers.
