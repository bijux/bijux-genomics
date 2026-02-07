# bijux-environment-qa

## What this crate does
Effectful QA harness for container images and environment specs. It validates real-world behavior and emits QA artifacts.

## What it must not do (boundaries)
Must never be depended on by production crates. It is heavy, effectful, and isolated to QA workflows.

## Public API / entrypoints
QA entrypoints are the binaries under `src/bin/` and the library in `src/lib/`. See `docs/RUNBOOK.md`.

## Key contracts it owns/consumes
Consumes environment specs and produces QA artifacts that mirror runtime manifests. See `docs/QA_MATRIX.md` and `docs/DATASETS.md`.

## Effects & determinism guarantees
May run docker and touch the network when explicitly invoked. Default tests are offline and deterministic.

## How to run its tests
See `docs/TESTS.md`. Key tests: `tests/qa_artifact_contract.rs`, `tests/image_qa_support.rs`, `tests/guardrails.rs`.

## Where the docs live
Start at `docs/INDEX.md`, then read `docs/RUNBOOK.md`, `docs/QA_MATRIX.md`, and `docs/APPTAINER_PLAN.md`.

## Artifacts / Contracts
Produces QA manifests and reports for image validation; see `tests/fixtures/qa_artifacts/`.

## Failure modes
Failures include missing images, behavioral mismatches, or artifact contract violations.

## Stability
QA behavior is allowed to evolve; changes must be reflected in docs and tests per `docs/CHANGE_RULES.md`.
