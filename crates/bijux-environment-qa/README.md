# bijux-environment-qa

## What this crate does
Effectful QA harness for image validation (non-production). Heavy deps allowed.

## What it must not do (boundaries)
Must never be depended on by production crates. Enforced by
`crates/bijux-policies/tests/deps/qa_dependency_policy.rs`.

## Role in the stack
Upstream: QA workflows. Downstream: none in production.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/RUNBOOK.md`, `docs/QA_MATRIX.md`, `docs/DATASETS.md`, `docs/APPTAINER_PLAN.md`, `docs/CHANGE_RULES.md`.

## Key contracts it owns/consumes
QA manifests/reports and validation records.

## Effects & determinism guarantees
May run docker/network when explicitly invoked; default tests are offline. See `docs/EFFECTS.md` and `docs/OFFLINE_POLICY.md`.

## How to run
Typical QA runs are invoked via the runbook. Expect long runtimes and heavy IO.
See `docs/RUNBOOK.md` and `docs/QA_MATRIX.md`.

## Expected runtime
Minutes to hours depending on the scenario set and image catalog size.

## Artifacts / Contracts
- QA manifest and report JSON (see `docs/ARTIFACT_CONTRACT.md`).
- Logs and validation records under the QA output directory.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/artifacts/qa_artifact_contract.rs`, `tests/support/image_qa_support.rs`, `tests/guardrails/guardrails.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.

## Must not be depended on
This crate is QA-only and must not be depended on by production crates.
