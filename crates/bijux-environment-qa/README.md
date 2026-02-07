# bijux-environment-qa

## What this crate does
Effectful QA harness for image validation (non-production).

## What it must not do (boundaries)
Must never be depended on by production crates.

## Role in the stack
Upstream: QA workflows. Downstream: none in production.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/RUNBOOK.md`, `docs/QA_MATRIX.md`, `docs/DATASETS.md`, `docs/APPTAINER_PLAN.md`, `docs/CHANGE_RULES.md`.

## Key contracts it owns/consumes
QA manifests/reports and validation records.

## Effects & determinism guarantees
May run docker/network when explicitly invoked; default tests are offline. See `docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/qa_artifact_contract.rs`, `tests/image_qa_support.rs`, `tests/guardrails.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
