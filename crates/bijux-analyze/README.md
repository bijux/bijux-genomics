# bijux-analyze

## What this crate does
Loads runtime artifacts, scores decisions, and renders reports.

## What it must not do (boundaries)
No planning or execution.

## Role in the stack
Upstream: runtime artifacts. Downstream: benchmark and users.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/DATA_MODEL.md`, `docs/DECISIONS.md`, `docs/SCHEMA.md`, `docs/PERFORMANCE_BUDGET.md`, `docs/CHANGE_RULES.md`.

## Key contracts it owns/consumes
Report JSON and bundle outputs.

## Effects & determinism guarantees
Pure computation + report rendering; deterministic outputs. See `docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/report_contract.rs`, `tests/report_determinism.rs`, `tests/performance_budget.rs`, `tests/contract_handshake.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
