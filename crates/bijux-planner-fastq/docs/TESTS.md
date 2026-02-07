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
- `tests/explain/*` → explain payload shape + documentation anchors.

## Mapping
- `tests/determinism/determinism.rs` → stable ordering/hashes.
- `tests/graph/graph_snapshots.rs` → graph snapshot contract.
- `tests/plan/plan_json.rs` → plan JSON schema contract.
- `tests/plan/plan_snapshots.rs` → plan snapshot contract.
- `tests/plan/contract_handshake.rs` → plan handshake fixtures.
- `tests/plan/trim_params.rs` → trim param resolution contract.
- `tests/plan/trim_plan.rs` → trim stage plan contract.
- `tests/plan/guardrails.rs` → policy guardrails.
- `tests/plan/no_parsing.rs` → planner purity (no observer parsing APIs).
- `tests/explain/explainability.rs` → explain output contract.
- `tests/explain/docs_explainability.rs` → docs anchor coverage.

## Failure modes
- Missing test documentation causes drift and confusion.

## Testkit patterns
See `crates/bijux-testkit/docs/USAGE.md` for shared fixture and snapshot helpers.
