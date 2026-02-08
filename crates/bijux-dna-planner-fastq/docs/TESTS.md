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
- `tests/contracts/explain/*` → explain payload shape + documentation anchors (docs/EXPLAIN_OUTPUT.md).

## Mapping
- `tests/determinism/determinism.rs` → stable ordering/hashes.
- `tests/contracts/graph/graph_snapshots.rs` → graph snapshot contract.
- `tests/contracts/plan/plan_json.rs` → plan JSON schema contract.
- `tests/contracts/plan/plan_snapshots.rs` → plan snapshot contract.
- `tests/contracts/plan/contract_handshake.rs` → plan handshake fixtures.
- `tests/contracts/plan/trim_params.rs` → trim param resolution contract.
- `tests/contracts/plan/trim_plan.rs` → trim stage plan contract.
- `tests/contracts/plan/guardrails.rs` → policy guardrails.
- `tests/contracts/plan/no_parsing.rs` → planner purity (no observer parsing APIs).
- `tests/contracts/explain/explainability.rs` → explain output contract.
- `tests/contracts/explain/docs_explainability.rs` → docs anchor coverage.

## Failure modes
- Missing test documentation causes drift and confusion.

## Testkit patterns
See `crates/bijux-dna-testkit/docs/USAGE.md` for shared fixture and snapshot helpers.
