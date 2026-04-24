# bijux-dna-environment-qa

## What this crate does
Effectful QA harness for image validation (non-production). Heavy deps allowed.
Runs Docker QA workflows and Apptainer smoke-contract QA.

## What it must not do (boundaries)
Must never be depended on by production crates. Enforced by
`crates/bijux-dna-policies/tests/boundaries/deps/budgets/qa_dependency_policy.rs`.

## Role in the stack
Upstream: QA workflows. Downstream: none in production.

## Public API / entrypoints
See `crates/bijux-dna-environment-qa/docs/INDEX.md`, `crates/bijux-dna-environment-qa/docs/RUNBOOK.md`, `crates/bijux-dna-environment-qa/docs/QA_MATRIX.md`, `crates/bijux-dna-environment-qa/docs/DATASETS.md`, `crates/bijux-dna-environment-qa/docs/APPTAINER_PLAN.md`, `crates/bijux-dna-environment-qa/docs/CHANGE_RULES.md`.

## Internal module layout
The image QA surface is organized by responsibility:
- `src/image_qa/contracts.rs` owns the shared QA stage and dataset contracts.
- `src/public_api.rs` exposes the stable environment facade consumed by QA workflows.
- `src/image_qa/datasets/`, `src/image_qa/records/`, and `src/image_qa/validation/` own discovery, persistence, and pass requirements.
- `src/image_qa/behavioral/` splits preprocessing stage checks from postprocessing/reporting checks.
- `src/image_qa/support/` owns diagnostics, output contracts, image resolution, Docker runtime helpers, and seqkit metrics.
- `src/image_qa/qa_docker_images/` owns Docker image planning, probing, runtime adapters, and reporting for catalog smoke checks.

## Key contracts it owns/consumes
QA manifests/reports and validation records.

## Effects & determinism guarantees
May run docker/network when explicitly invoked; default tests are offline. See `crates/bijux-dna-environment-qa/docs/EFFECTS.md` and `crates/bijux-dna-environment-qa/docs/OFFLINE_POLICY.md`.

## How to run
Typical QA runs are invoked via the runbook. Expect long runtimes and heavy IO.
See `crates/bijux-dna-environment-qa/docs/RUNBOOK.md` and `crates/bijux-dna-environment-qa/docs/QA_MATRIX.md`.

## Expected runtime
Minutes to hours depending on the scenario set and image catalog size.

## Artifacts / Contracts
- QA manifest and report JSON (see `crates/bijux-dna-environment-qa/docs/ARTIFACT_CONTRACT.md`).
- Logs and validation records under the QA output directory.

## How to run its tests
See `crates/bijux-dna-environment-qa/docs/TESTS.md`. High-signal targets: `tests/contracts/artifacts/qa_artifact_contract.rs`, `tests/contracts/qa_contracts.rs`, `tests/boundaries/architecture.rs`, `tests/boundaries/guardrails/guardrails.rs`, `tests/determinism/fixture_stability.rs`.

## Where the docs live
Start at `crates/bijux-dna-environment-qa/docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `crates/bijux-dna-environment-qa/docs/CHANGE_RULES.md`.

## Must not be depended on
This crate is QA-only and must not be depended on by production crates.
