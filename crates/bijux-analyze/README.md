# bijux-analyze

## What this crate does
Loads run artifacts, scores decisions, and renders reports (load → decide → report).

## What it must not do (boundaries)
Must not plan graphs or execute tools. It consumes runtime artifacts only.

## Public API / entrypoints
Report and decision contracts documented in `docs/DECISIONS.md` and `docs/SCHEMA.md`.

## Key contracts it owns/consumes
Consumes run manifests/records; produces report bundles and summaries.

## Effects & determinism guarantees
Deterministic report rendering and schema stability enforced by snapshots.

## How to run its tests
See `docs/TESTS.md`. Key tests: `tests/report_contract.rs`, `tests/report_determinism.rs`, `tests/performance_budget.rs`.

## Where the docs live
Start at `docs/INDEX.md`, then read `docs/DATA_MODEL.md`, `docs/DECISIONS.md`, and `docs/SCHEMA.md`.

## Artifacts / Contracts
Produces `report.json`, `report_bundle/`, and summaries; fixtures in `tests/fixtures/`.

## Failure modes
Missing fields or unstable ordering fail report completeness and determinism tests.

## Stability
Schema and report bundles are snapshot-tested; see `docs/CHANGE_RULES.md`.
