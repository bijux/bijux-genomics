# bijux-analyze

## What this crate does
Loads runtime artifacts, scores decisions, and renders reports.

## What it must not do (boundaries)
No planning or execution.

## Role in the stack
Upstream: runtime artifacts. Downstream: benchmark and users.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/REPORT_CONTRACT.md`, `docs/DECISIONS.md`, `docs/PERFORMANCE_BUDGET.md`, `docs/CHANGE_RULES.md`, `docs/DETERMINISM.md`.

## Key contracts it owns/consumes
Report JSON (`report.json`) and HTML bundle (`report_bundle/index.html`) outputs.

## Effects & determinism guarantees
Pure computation + report rendering; deterministic outputs. See `docs/EFFECTS.md` and `docs/DETERMINISM.md`.

## Report artifacts
- `report.json`
- `report_bundle/index.html`

## Performance budgets are enforced
See `docs/PERFORMANCE_BUDGET.md` and `tests/report/performance_budget.rs`.

## Interpretation guide
Start with `docs/INTERPRETATION.md`.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/report/report_contract.rs`, `tests/report/report_determinism.rs`, `tests/report/performance_budget.rs`, `tests/contracts/contract_handshake.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
