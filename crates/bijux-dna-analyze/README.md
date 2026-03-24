# bijux-dna-analyze

## What this crate does
Loads runtime artifacts, scores decisions, and renders report bundles.

## What it must not do (boundaries)
No planning or execution.

## Role in the stack
Upstream: runtime artifacts. Downstream: benchmark and users.

## Public API / entrypoints
See `crates/bijux-dna-analyze/docs/INDEX.md`, `crates/bijux-dna-analyze/docs/REPORT_CONTRACT.md`, `crates/bijux-dna-analyze/docs/DECISIONS.md`, `crates/bijux-dna-analyze/docs/PERFORMANCE_BUDGET.md`,
`crates/bijux-dna-analyze/docs/CHANGE_RULES.md`, `crates/bijux-dna-analyze/docs/DETERMINISM.md`.

## What you get (inputs → outputs)
Inputs:
- Runtime artifacts (manifest, provenance, record, layout).
- Stage metrics and tool outputs referenced by the run manifest.

Outputs:
- `report.json` (canonical JSON report).
- `report_bundle/` (HTML bundle + bundled assets).

## Report structure walkthrough (one page)
1. Load runtime artifacts and validate schema compatibility.
2. Resolve pipeline defaults + effective configs.
3. Aggregate stage reports and metrics into report sections.
4. Compute verdicts, deltas, and failure hints.
5. Emit `report.json` and an HTML bundle that mirrors the JSON structure.

## Key contracts it owns/consumes
Report JSON (`report.json`) and HTML bundle (`report_bundle/index.html`) outputs.

## Effects & determinism guarantees
Pure computation + report rendering; deterministic outputs. See `crates/bijux-dna-analyze/docs/EFFECTS.md` and `crates/bijux-dna-analyze/docs/DETERMINISM.md`.

## Report artifacts
- `report.json`
- `report_bundle/index.html`

## Artifacts / Contracts
- `report.json` contract: `crates/bijux-dna-analyze/docs/REPORT_CONTRACT.md` + `tests/contracts/report_contract.rs`.
- Report bundle structure: `crates/bijux-dna-analyze/docs/REPORT_CONTRACT.md` + `tests/report/report_contract.rs`.
- Failure hints contract: `crates/bijux-dna-analyze/docs/FAILURE_TAXONOMY.md` + `tests/contracts/failure_hints.rs`.

## Performance budgets are enforced
See `crates/bijux-dna-analyze/docs/PERFORMANCE_BUDGET.md` and `tests/report/performance_budget.rs`.
Budget expectations are: size caps and runtime ceilings; violations fail tests.

## Interpretation guide
Start with `crates/bijux-dna-analyze/docs/INTERPRETATION.md`.

## Failure taxonomy
See `crates/bijux-dna-analyze/docs/FAILURE_TAXONOMY.md` and `tests/contracts/failure_hints.rs`.

## How to run its tests
See `crates/bijux-dna-analyze/docs/TESTS.md`. Golden tests: `tests/report/report_contract.rs`,
`tests/report/report_determinism.rs`, `tests/report/performance_budget.rs`,
`tests/contracts/contract_handshake.rs`.

## Where the docs live
Start at `crates/bijux-dna-analyze/docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `crates/bijux-dna-analyze/docs/CHANGE_RULES.md`.
